#!/bin/bash
# CI lint: scan emit_intrinsic_call in sifr_codegen for panic-inducing patterns
# Fails with exit 1 if any user-facing .unwrap()/.expect()/panic!()/unreachable!() found
# Lines with "// SAFETY:" or "// COMPILER-INTERNAL:" comments are exempt

set -euo pipefail

TARGET_FILE="crates/sifr_codegen/src/lib.rs"

echo "=== Panic Pattern Lint: sifr_codegen ==="
echo "Scanning emit_intrinsic_call for panic-inducing patterns..."
echo ""

# Dynamically find emit_intrinsic_call function boundaries
FUNC_START=$(grep -n 'fn emit_intrinsic_call' "$TARGET_FILE" | head -1 | cut -d: -f1)
if [ -z "$FUNC_START" ]; then
    echo "ERROR: Could not find emit_intrinsic_call function"
    exit 2
fi

# Find the next top-level fn after emit_intrinsic_call (unindented "fn " or "    fn " at method level)
FUNC_END=$(tail -n +$((FUNC_START + 1)) "$TARGET_FILE" | grep -n '^\s\{4\}fn \|^fn ' | head -1 | cut -d: -f1)
if [ -z "$FUNC_END" ]; then
    FUNC_END=$(wc -l < "$TARGET_FILE")
else
    FUNC_END=$((FUNC_START + FUNC_END - 1))
fi

echo "Function block: lines $FUNC_START - $FUNC_END"
echo ""

VIOLATIONS=()
ALLOWED=()

# Extract function block with original line numbers
sed -n "${FUNC_START},${FUNC_END}p" "$TARGET_FILE" | \
    awk -v start="$FUNC_START" '{print (NR + start - 1) ":" $0}' | \
while IFS= read -r line; do
    LINE_NUM="${line%%:*}"
    LINE_CONTENT="${line#*:}"

    # Skip lines that are comments (in Rust source, not in self.write strings)
    # We only care about patterns that appear in emitted Rust code (inside self.write("..."))

    HAS_PATTERN=0
    PATTERN_TYPE=""

    # Check .unwrap() — exclude .unwrap_or, .unwrap_or_default, .unwrap_or_else
    if echo "$LINE_CONTENT" | grep -qE '\.unwrap\(\)'; then
        STRIPPED=$(echo "$LINE_CONTENT" | sed 's/\.unwrap_or_default()/__X__/g; s/\.unwrap_or_else([^)]*)/__X__/g; s/\.unwrap_or([^)]*)/__X__/g')
        if echo "$STRIPPED" | grep -qE '\.unwrap\(\)'; then
            HAS_PATTERN=1
            PATTERN_TYPE=".unwrap()"
        fi
    fi

    # Check .expect(
    if echo "$LINE_CONTENT" | grep -qE '\.expect\('; then
        HAS_PATTERN=1
        PATTERN_TYPE=".expect()"
    fi

    # Check panic!(
    if echo "$LINE_CONTENT" | grep -qE 'panic!\('; then
        HAS_PATTERN=1
        PATTERN_TYPE="panic!()"
    fi

    # Check unreachable!(
    if echo "$LINE_CONTENT" | grep -qE 'unreachable!\('; then
        HAS_PATTERN=1
        PATTERN_TYPE="unreachable!()"
    fi

    if [ "$HAS_PATTERN" -eq 1 ]; then
        if echo "$LINE_CONTENT" | grep -qE '//\s*(SAFETY|COMPILER-INTERNAL):'; then
            echo "ALLOWED:$LINE_NUM:$PATTERN_TYPE:$LINE_CONTENT" >> /tmp/sifr_lint_allowed.txt
        else
            echo "VIOLATION:$LINE_NUM:$PATTERN_TYPE:$LINE_CONTENT" >> /tmp/sifr_lint_violations.txt
        fi
    fi
done

# Read results from temp files
rm -f /tmp/sifr_lint_allowed.txt /tmp/sifr_lint_violations.txt 2>/dev/null || true

# Re-run to collect (pipe subshell loses variables)
VIOLATION_COUNT=0
ALLOWED_COUNT=0
TOTAL=0

TEMP_RESULTS=$(mktemp)
trap "rm -f $TEMP_RESULTS" EXIT

sed -n "${FUNC_START},${FUNC_END}p" "$TARGET_FILE" | \
    awk -v start="$FUNC_START" '{print (NR + start - 1) ":" $0}' > "$TEMP_RESULTS"

declare -a VIOLATIONS_ARR=()
declare -a ALLOWED_ARR=()

while IFS= read -r line; do
    LINE_NUM="${line%%:*}"
    LINE_CONTENT="${line#*:}"
    HAS_PATTERN=0

    if echo "$LINE_CONTENT" | grep -qE '\.unwrap\(\)'; then
        STRIPPED=$(echo "$LINE_CONTENT" | sed 's/\.unwrap_or_default()/__X__/g; s/\.unwrap_or_else([^)]*))/__X__/g; s/\.unwrap_or([^)]*)/__X__/g')
        if echo "$STRIPPED" | grep -qE '\.unwrap\(\)'; then
            HAS_PATTERN=1
        fi
    fi

    if echo "$LINE_CONTENT" | grep -qE '\.expect\('; then HAS_PATTERN=1; fi
    if echo "$LINE_CONTENT" | grep -qE 'panic!\('; then HAS_PATTERN=1; fi
    if echo "$LINE_CONTENT" | grep -qE 'unreachable!\('; then HAS_PATTERN=1; fi

    if [ "$HAS_PATTERN" -eq 1 ]; then
        TRIMMED=$(echo "$LINE_CONTENT" | sed 's/^[[:space:]]*//')
        if echo "$LINE_CONTENT" | grep -qE '//\s*(SAFETY|COMPILER-INTERNAL):'; then
            ALLOWED_ARR+=("  Line $LINE_NUM: $TRIMMED")
        else
            VIOLATIONS_ARR+=("  Line $LINE_NUM: $TRIMMED")
        fi
    fi
done < "$TEMP_RESULTS"

TOTAL=$(( ${#ALLOWED_ARR[@]} + ${#VIOLATIONS_ARR[@]} ))

echo "Found $TOTAL patterns, ${#ALLOWED_ARR[@]} allowed (compiler-internal), ${#VIOLATIONS_ARR[@]} violations"
echo ""

if [ ${#ALLOWED_ARR[@]} -gt 0 ]; then
    echo "ALLOWED (compiler-internal):"
    for item in "${ALLOWED_ARR[@]}"; do
        echo "$item"
    done
    echo ""
fi

if [ ${#VIOLATIONS_ARR[@]} -gt 0 ]; then
    echo "VIOLATIONS:"
    for item in "${VIOLATIONS_ARR[@]}"; do
        echo "$item"
    done
    echo ""
    echo "FAIL: ${#VIOLATIONS_ARR[@]} panic-inducing pattern(s) found in user-facing code"
    exit 1
else
    echo "PASS: No violations found in emit_intrinsic_call"
    exit 0
fi

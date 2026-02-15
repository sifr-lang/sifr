#!/bin/bash
# Fast audit: only sifr check (no Rust compile/run) for large directories
# Usage: ./audit/run_audit_fast.sh <audit_dir>

AUDIT_DIR="$1"
if [ -z "$AUDIT_DIR" ]; then
    echo "Usage: $0 <audit_dir>"
    exit 1
fi

SIFR_BIN="$(pwd)/target/release/sifr"

PASS=0
FAIL=0

declare -a PASS_FILES
declare -a FAIL_FILES
declare -a FAIL_MSGS

for f in $(ls "$AUDIT_DIR"/*.sifr 2>/dev/null | sort); do
    fname=$(basename "$f")
    
    compile_output=$("$SIFR_BIN" check "$f" 2>&1)
    compile_exit=$?
    
    if [ $compile_exit -ne 0 ]; then
        FAIL=$((FAIL + 1))
        FAIL_FILES+=("$fname")
        error_msg=$(echo "$compile_output" | head -3)
        FAIL_MSGS+=("$error_msg")
    else
        PASS=$((PASS + 1))
        PASS_FILES+=("$fname")
    fi
done

TOTAL=$((PASS + FAIL))

echo "=== Check-Only Audit Results: $(basename $AUDIT_DIR) ==="
echo "Total: $TOTAL | Pass: $PASS | Fail: $FAIL"
echo ""

if [ $FAIL -gt 0 ]; then
    echo "--- Failures ($FAIL) ---"
    for i in "${!FAIL_FILES[@]}"; do
        echo ""
        echo "  [FAIL] ${FAIL_FILES[$i]}"
        echo "${FAIL_MSGS[$i]}" | sed 's/^/    /'
    done
    echo ""
fi

echo "--- Passing ($PASS) ---"
for f in "${PASS_FILES[@]}"; do
    echo "  [PASS] $f"
done

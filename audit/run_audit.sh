#!/bin/bash
# Run all .sifr files in a given audit directory and report results
# Usage: ./audit/run_audit.sh <audit_dir>

AUDIT_DIR="$1"
if [ -z "$AUDIT_DIR" ]; then
    echo "Usage: $0 <audit_dir>"
    exit 1
fi

SIFR_BIN="$(pwd)/target/release/sifr"

PASS=0
FAIL_COMPILE=0
FAIL_RUST=0
FAIL_RUN=0

declare -a PASS_FILES
declare -a FAIL_COMPILE_FILES
declare -a FAIL_COMPILE_MSGS
declare -a FAIL_RUST_FILES
declare -a FAIL_RUST_MSGS
declare -a FAIL_RUN_FILES
declare -a FAIL_RUN_MSGS

for f in $(ls "$AUDIT_DIR"/*.sifr 2>/dev/null | sort); do
    fname=$(basename "$f")
    
    # Step 1: Try to compile with sifr check
    compile_output=$("$SIFR_BIN" check "$f" 2>&1)
    compile_exit=$?
    
    if [ $compile_exit -ne 0 ]; then
        FAIL_COMPILE=$((FAIL_COMPILE + 1))
        FAIL_COMPILE_FILES+=("$fname")
        error_msg=$(echo "$compile_output" | head -5)
        FAIL_COMPILE_MSGS+=("$error_msg")
        continue
    fi
    
    # Step 2: Try to compile and run with sifr run
    run_output=$("$SIFR_BIN" run "$f" 2>&1)
    run_exit=$?
    
    if [ $run_exit -ne 0 ]; then
        if echo "$run_output" | grep -qE "error\[E|Rust compilation failed|cannot find"; then
            FAIL_RUST=$((FAIL_RUST + 1))
            FAIL_RUST_FILES+=("$fname")
            error_msg=$(echo "$run_output" | grep "error" | head -5)
            FAIL_RUST_MSGS+=("$error_msg")
        else
            FAIL_RUN=$((FAIL_RUN + 1))
            FAIL_RUN_FILES+=("$fname")
            error_msg=$(echo "$run_output" | head -5)
            FAIL_RUN_MSGS+=("$error_msg")
        fi
        continue
    fi
    
    PASS=$((PASS + 1))
    PASS_FILES+=("$fname")
done

TOTAL=$((PASS + FAIL_COMPILE + FAIL_RUST + FAIL_RUN))

echo "=== Audit Results: $(basename $AUDIT_DIR) ==="
echo "Total: $TOTAL | Pass: $PASS | Fail(sifr compile): $FAIL_COMPILE | Fail(rust compile): $FAIL_RUST | Fail(runtime): $FAIL_RUN"
echo ""

if [ $FAIL_COMPILE -gt 0 ]; then
    echo "--- Sifr Compilation Failures ($FAIL_COMPILE) ---"
    for i in "${!FAIL_COMPILE_FILES[@]}"; do
        echo ""
        echo "  [FAIL_COMPILE] ${FAIL_COMPILE_FILES[$i]}"
        echo "${FAIL_COMPILE_MSGS[$i]}" | sed 's/^/    /'
    done
    echo ""
fi

if [ $FAIL_RUST -gt 0 ]; then
    echo "--- Rust Compilation Failures ($FAIL_RUST) ---"
    for i in "${!FAIL_RUST_FILES[@]}"; do
        echo ""
        echo "  [FAIL_RUST] ${FAIL_RUST_FILES[$i]}"
        echo "${FAIL_RUST_MSGS[$i]}" | sed 's/^/    /'
    done
    echo ""
fi

if [ $FAIL_RUN -gt 0 ]; then
    echo "--- Runtime Failures ($FAIL_RUN) ---"
    for i in "${!FAIL_RUN_FILES[@]}"; do
        echo ""
        echo "  [FAIL_RUN] ${FAIL_RUN_FILES[$i]}"
        echo "${FAIL_RUN_MSGS[$i]}" | sed 's/^/    /'
    done
    echo ""
fi

if [ $PASS -gt 0 ]; then
    echo "--- Passing ($PASS) ---"
    for f in "${PASS_FILES[@]}"; do
        echo "  [PASS] $f"
    done
    echo ""
fi

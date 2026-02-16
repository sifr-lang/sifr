#!/bin/bash
set +e
for f in audit/borrowing/*.sifr; do
    result=$(cargo run --quiet -- check "$f" 2>&1 | tail -1)
    echo "$(basename "$f"): $result"
done
echo "ALL_DONE"

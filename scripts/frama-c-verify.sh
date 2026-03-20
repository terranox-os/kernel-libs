#!/usr/bin/env bash
#
# frama-c-verify.sh — Run Frama-C WP verification on all annotated C sources.
#
# Usage: ./scripts/frama-c-verify.sh [--check-only]
#
#   --check-only   Parse ACSL annotations without running provers (fast).
#                  Useful for CI syntax checks when solvers aren't available.
#
# Prerequisites: frama-c, alt-ergo (optional: z3, cvc5)
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

CHECK_ONLY=false
if [[ "${1:-}" == "--check-only" ]]; then
    CHECK_ONLY=true
fi

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m'

# Verify frama-c is available
if ! command -v frama-c &>/dev/null; then
    echo -e "${RED}Error: frama-c not found in PATH${NC}"
    echo "Install via: opam install frama-c alt-ergo"
    exit 1
fi

echo "Frama-C version: $(frama-c -version 2>&1 | head -1)"

FAILURES=0
TOTAL=0

verify_file() {
    local src="$1"
    local includes="$2"
    local label="$3"

    TOTAL=$((TOTAL + 1))
    echo -n "  [$label] $src ... "

    if $CHECK_ONLY; then
        # Parse ACSL annotations only — no proof discharge
        if frama-c -kernel-warn-key annot-error=abort \
                   "-cpp-extra-args=$includes" \
                   -machdep x86_64 \
                   "$ROOT_DIR/$src" 2>&1 | grep -qi "error"; then
            echo -e "${RED}FAIL (parse error)${NC}"
            FAILURES=$((FAILURES + 1))
        else
            echo -e "${GREEN}OK (parsed)${NC}"
        fi
    else
        # Full WP verification with Alt-Ergo
        if frama-c -wp \
                   -wp-prover alt-ergo \
                   -wp-timeout 30 \
                   -kernel-warn-key annot-error=abort \
                   "-cpp-extra-args=$includes" \
                   -machdep x86_64 \
                   "$ROOT_DIR/$src" 2>&1; then
            echo -e "${GREEN}OK${NC}"
        else
            echo -e "${RED}FAIL${NC}"
            FAILURES=$((FAILURES + 1))
        fi
    fi
}

echo ""
echo "=== Primitives ==="
PRIM_INCLUDES="-I$ROOT_DIR/primitives/include"
for src in primitives/src/memcpy.c \
           primitives/src/memset.c \
           primitives/src/memmove.c \
           primitives/src/memcmp.c \
           primitives/src/strlen.c \
           primitives/src/strncmp.c \
           primitives/src/strnlen.c \
           primitives/src/secure_zero.c; do
    verify_file "$src" "$PRIM_INCLUDES" "primitives"
done

echo ""
echo "=== Bitops ==="
BITOPS_INCLUDES="-I$ROOT_DIR/bitops/include"
verify_file "bitops/src/bitmap.c" "$BITOPS_INCLUDES" "bitops"

echo ""
echo "=== Alloc ==="
ALLOC_INCLUDES="-I$ROOT_DIR/alloc/include -I$ROOT_DIR/genesis-abi/include -I$ROOT_DIR/bitops/include"
verify_file "alloc/src/bitmap_pmm.c" "$ALLOC_INCLUDES" "alloc"
verify_file "alloc/src/pool.c" "-I$ROOT_DIR/alloc/include -I$ROOT_DIR/genesis-abi/include" "alloc"
verify_file "alloc/src/slab.c" "$ALLOC_INCLUDES" "alloc"

echo ""
echo "==============================="
echo "Total: $TOTAL files"
echo -e "Passed: $((TOTAL - FAILURES))"
if [[ $FAILURES -gt 0 ]]; then
    echo -e "${RED}Failed: $FAILURES${NC}"
    exit 1
else
    echo -e "${GREEN}All passed!${NC}"
fi

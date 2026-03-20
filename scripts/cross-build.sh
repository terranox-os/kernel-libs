#!/usr/bin/env bash
#
# cross-build.sh — Cross-compile all crates for bare-metal targets.
#
# Usage: ./scripts/cross-build.sh [target...]
#
# If no targets specified, builds all four:
#   x86_64-unknown-none, aarch64-unknown-none,
#   thumbv7em-none-eabi, riscv64gc-unknown-none-elf
#
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

DEFAULT_TARGETS=(
    "x86_64-unknown-none"
    "aarch64-unknown-none"
    "thumbv7em-none-eabi"
    "riscv64gc-unknown-none-elf"
)

if [[ $# -gt 0 ]]; then
    TARGETS=("$@")
else
    TARGETS=("${DEFAULT_TARGETS[@]}")
fi

FAILURES=0

for target in "${TARGETS[@]}"; do
    echo -n "Building workspace for $target ... "

    # Ensure target is installed
    if ! rustup target list --installed | grep -q "^${target}$"; then
        echo -n "(installing) "
        rustup target add "$target" 2>/dev/null
    fi

    if cargo build --workspace --target "$target" 2>/dev/null; then
        echo -e "${GREEN}OK${NC}"
    else
        echo -e "${RED}FAIL${NC}"
        FAILURES=$((FAILURES + 1))
        # Show the actual error
        cargo build --workspace --target "$target" 2>&1 | tail -20
    fi
done

echo ""
echo "==============================="
echo "Targets: ${#TARGETS[@]}"
echo "Passed: $(( ${#TARGETS[@]} - FAILURES ))"
if [[ $FAILURES -gt 0 ]]; then
    echo -e "${RED}Failed: $FAILURES${NC}"
    exit 1
else
    echo -e "${GREEN}All passed!${NC}"
fi

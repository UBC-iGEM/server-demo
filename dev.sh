#!/usr/bin/env bash
set -eo pipefail

BOLD=$(printf '\033[1m')
RED=$(printf '\033[31m')
GREEN=$(printf '\033[32m')
BLUE=$(printf '\033[96m')
RESET=$(printf '\033[0m')

usage() {
    cat <<EOF
${BOLD}${GREEN}Usage: ${BLUE}./make.sh <command> [options]${RESET}
${BOLD}${GREEN}Commands:${RESET}
    ${BOLD}${BLUE}build${RESET}         Build the project
    ${BOLD}${BLUE}run${RESET}           Build the project and run the server
    ${BOLD}${BLUE}--help${RESET}        Show this help message and exit
${BOLD}${GREEN}Options:${RESET}
    ${BOLD}${BLUE}--release${RESET}     Enable optimizations while building
EOF
}

invalid_arg() {
    echo -e "${BOLD}${RED}Invalid argument!${RESET}\n"
    usage
    exit 1
}


if [[ -z "$1" ]]; then
    invalid_arg
fi
if [[ "$1" == "--help" ]]; then
    usage
    exit 0
fi
if [[ "$1" != "run" && "$1" != "build" ]]; then
    invalid_arg
fi


RELEASE=false
if [[ "$2" == "--release" ]]; then
    RELEASE=true
elif [[ -n "$2" ]]; then
    invalid_arg
fi

if [[ "$RELEASE" == "true" ]]; then
    cargo build --release
else
    cargo build
fi

rm -rf target/dist/
mkdir -p target/dist/
cp -r public target/dist/

if [[ "$RELEASE" == "true" ]]; then
    cp target/release/server target/dist/
else
    cp target/debug/server target/dist/
fi

if [[ "$1" == "run" ]]; then
    echo "==> Build complete! Running server..."
    ./target/dist/server
else
    echo "==> Build complete! Output binary is in target/dist/"
fi

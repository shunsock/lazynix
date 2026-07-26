#!/usr/bin/env bash
set -euo pipefail

readonly EXAMPLES_DIR="examples"
readonly LNIX_BIN="./target/release/lnix"

if [ ! -x "${LNIX_BIN}" ]; then
	echo "lnix binary not found at ${LNIX_BIN}; build with 'cargo build --release -p lnix' first" >&2
	exit 1
fi

processed=0
for dir in "${EXAMPLES_DIR}"/*/; do
	if [ ! -f "${dir}lazynix.yaml" ]; then
		continue
	fi
	echo "Regenerating ${dir}"
	"${LNIX_BIN}" -C "${dir}" run -- true
	processed=$((processed + 1))
done

if [ "${processed}" -eq 0 ]; then
	echo "No examples with lazynix.yaml were found under ${EXAMPLES_DIR}/" >&2
	exit 1
fi

git diff --exit-code -- "${EXAMPLES_DIR}/"

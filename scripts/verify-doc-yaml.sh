#!/usr/bin/env bash
set -euo pipefail

# SEE: crates/lnix-infra/tests/verify_doc_yaml_test.rs
# CONSTRAINT: document/jp/design/version-pinning.md is excluded because
# its "Before (v0.2.0)" block is intentionally in the pre-v0.3.0 schema.

usage() {
	cat <<'EOF' >&2
Usage: verify-doc-yaml.sh [FILE ...]

Extract every ```yaml``` block whose first non-empty line starts with
"devShell:" from each markdown FILE (default: shipped docs), then parse
each block as a lnix_domain::DevShellDefinition via cargo test.

Exit code 0 = every extracted block parses.
Exit code 1 = at least one block failed to parse.
EOF
}

if [[ "${1-}" == "-h" || "${1-}" == "--help" ]]; then
	usage
	exit 0
fi

repo_root=$(cd "$(dirname "$0")/.." && pwd)
cd "$repo_root"

normalize_path() {
	local path=$1
	path="${path#./}"
	while [[ "$path" == *"//"* ]]; do
		path="${path//\/\//\/}"
	done
	printf '%s\n' "$path"
}

default_files() {
	local -a files=()
	if [[ -f README.md ]]; then
		files+=("README.md")
	fi
	while IFS= read -r -d '' f; do
		files+=("$f")
	done < <(find document -type f -name '*.md' -print0)
	while IFS= read -r -d '' f; do
		files+=("$f")
	done < <(find examples -type f -name '*.md' -print0)
	printf '%s\n' "${files[@]}"
}

if [[ $# -gt 0 ]]; then
	mapfile -t files < <(printf '%s\n' "$@")
else
	mapfile -t files < <(default_files)
fi

excluded=$(normalize_path "document/jp/design/version-pinning.md")
kept=()
for f in "${files[@]}"; do
	if [[ "$(normalize_path "$f")" == "$excluded" ]]; then
		continue
	fi
	kept+=("$f")
done

if [[ ${#kept[@]} -eq 0 ]]; then
	echo "verify-doc-yaml: no markdown files to check" >&2
	exit 0
fi

tmpdir=$(mktemp -d -t verify-doc-yaml.XXXXXX)
trap 'rm -rf "$tmpdir"' EXIT

EXTRACT_BLOCK_COUNT=0

extract_blocks() {
	local file=$1 out_dir=$2
	local in_block=0 indent="" indent_len=0 outfile=""
	local line stripped

	while IFS= read -r line || [[ -n "$line" ]]; do
		if [[ $in_block -eq 0 ]]; then
			if [[ "$line" =~ ^([[:space:]]*)\`\`\`yaml[[:space:]]*$ ]]; then
				indent="${BASH_REMATCH[1]}"
				indent_len=${#indent}
				in_block=1
				EXTRACT_BLOCK_COUNT=$((EXTRACT_BLOCK_COUNT + 1))
				outfile=$(printf '%s/block_%04d.yaml' "$out_dir" "$EXTRACT_BLOCK_COUNT")
				: >"$outfile"
			fi
			continue
		fi

		if [[ "$line" =~ ^${indent}\`\`\`[[:space:]]*$ ]]; then
			in_block=0
			continue
		fi

		if [[ $indent_len -gt 0 && "${line:0:$indent_len}" == "$indent" ]]; then
			stripped="${line:$indent_len}"
		else
			stripped="$line"
		fi
		printf '%s\n' "$stripped" >>"$outfile"
	done <"$file"

	if [[ $in_block -eq 1 ]]; then
		echo "verify-doc-yaml: unclosed \`\`\`yaml fence in $file" >&2
		exit 1
	fi
}

is_devshell_block() {
	local file=$1 line trimmed
	while IFS= read -r line || [[ -n "$line" ]]; do
		[[ -z "${line//[[:space:]]/}" ]] && continue
		trimmed="${line#"${line%%[![:space:]]*}"}"
		[[ "$trimmed" == devShell:* ]]
		return $?
	done <"$file"
	return 1
}

for md in "${kept[@]}"; do
	extract_blocks "$md" "$tmpdir"
done

kept_blocks=0
for yaml in "$tmpdir"/block_*.yaml; do
	[[ -e "$yaml" ]] || continue
	if is_devshell_block "$yaml"; then
		kept_blocks=$((kept_blocks + 1))
	else
		rm -f "$yaml"
	fi
done

echo "verify-doc-yaml: extracted $kept_blocks devShell block(s) from ${#kept[@]} file(s)"

if [[ $kept_blocks -eq 0 ]]; then
	echo "verify-doc-yaml: nothing to validate" >&2
	exit 0
fi

LAZYNIX_DOC_YAML_DIR="$tmpdir" \
	cargo test --quiet -p lnix-infra --test verify_doc_yaml_test \
	parses_every_yaml_file_in_env_directory -- --exact --nocapture

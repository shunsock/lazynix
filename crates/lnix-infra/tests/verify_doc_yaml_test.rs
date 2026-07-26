//! Doc-YAML regression harness.
//!
//! Ensures every ```yaml``` block whose first non-empty line starts with
//! `devShell:` in the shipped documentation (README, `document/**`,
//! `examples/**/README.md`) still parses into [`DevShellDefinition`].
//!
//! Historically the schema drifted from bare-string package lists to the
//! `- name:` mapping without a matching CI gate, which let every example
//! break silently. This test freezes the invariant so the drift cannot
//! recur.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use lnix_domain::DevShellDefinition;

// NOTE: shipped docs currently contain README + document/** + examples/**.
// Below the current floor means a section was accidentally deleted.
const MIN_EXPECTED_DEVSHELL_BLOCKS: usize = 5;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates dir")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

/// Extract every fenced ```yaml``` block regardless of leading indentation.
///
/// The block's own indent (whitespace before the opening fence) is stripped
/// from each interior line so the recovered YAML is well-formed regardless of
/// the surrounding Markdown structure.
fn extract_yaml_blocks(markdown: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut lines = markdown.lines();
    while let Some(line) = lines.next() {
        let trimmed = line.trim_start();
        if trimmed != "```yaml" {
            continue;
        }
        let indent_len = line.len() - trimmed.len();
        let indent = &line[..indent_len];
        let mut body = String::new();
        for content_line in lines.by_ref() {
            let content_trim = content_line.trim_start();
            let content_indent_len = content_line.len() - content_trim.len();
            if content_indent_len >= indent_len && content_trim == "```" {
                break;
            }
            // NOTE: 空行や字下げが浅い行はそのまま残し、YAML 側の空行として保持する。
            // Markdown 側の indent が保証されるのは実データ行のみ。
            let stripped = if content_line.starts_with(indent) {
                &content_line[indent_len..]
            } else {
                content_line
            };
            body.push_str(stripped);
            body.push('\n');
        }
        blocks.push(body);
    }
    blocks
}

/// A block is considered a `devShell:` block only if the first
/// non-empty line begins with `devShell:`. That excludes
/// `lazynix-settings.yaml` snippets (`override-stable-package: ...`) as
/// well as unrelated snippets that happen to be in ```yaml``` fences.
fn is_devshell_block(block: &str) -> bool {
    block
        .lines()
        .find(|line| !line.trim().is_empty())
        .is_some_and(|line| line.trim_start().starts_with("devShell:"))
}

/// Discover the documentation files whose YAML blocks are subject to
/// validation. `document/jp/design/version-pinning.md` is excluded
/// wholesale: it contrasts the v0.2.0 bare-string schema against the
/// v0.3.0 mapping schema, so its "Before" block is expected to be
/// deliberately invalid.
fn discover_doc_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let readme = root.join("README.md");
    if readme.is_file() {
        files.push(readme);
    }
    collect_markdown(&root.join("document"), &mut files);
    collect_markdown(&root.join("examples"), &mut files);

    let excluded = root.join("document/jp/design/version-pinning.md");
    files.retain(|p| p != &excluded);
    files.sort();
    files
}

fn collect_markdown(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_markdown(&path, out);
        } else if path.extension().is_some_and(|e| e == "md") {
            out.push(path);
        }
    }
}

#[test]
fn parses_valid_new_schema_block() {
    let yaml = "devShell:\n  package:\n    stable:\n      - name: hello\n";
    let parsed: Result<DevShellDefinition, _> = serde_yaml::from_str(yaml);
    assert!(parsed.is_ok(), "expected Ok, got {:?}", parsed.err());
}

#[test]
fn rejects_pre_v03_bare_string_schema() {
    let yaml = "devShell:\n  package:\n    stable:\n      - python312\n";
    let parsed: Result<DevShellDefinition, _> = serde_yaml::from_str(yaml);
    assert!(
        parsed.is_err(),
        "the doc-yaml gate relies on serde rejecting bare-string packages"
    );
}

#[test]
fn extract_returns_empty_when_no_yaml_fences() {
    assert!(extract_yaml_blocks("no yaml here\n").is_empty());
}

#[test]
fn extract_captures_a_flush_left_devshell_block() {
    let md = "before\n```yaml\ndevShell:\n  package:\n    stable: []\n```\nafter\n";
    let blocks = extract_yaml_blocks(md);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0], "devShell:\n  package:\n    stable: []\n");
}

#[test]
fn extract_strips_common_leading_indent() {
    let md = "1. item\n\n   ```yaml\n   devShell:\n     package:\n       stable: []\n   ```\n";
    let blocks = extract_yaml_blocks(md);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0], "devShell:\n  package:\n    stable: []\n");
}

#[test]
fn is_devshell_block_rejects_settings_snippet() {
    let block = "override-stable-package: \"github:x/y/z\"\n";
    assert!(!is_devshell_block(block));
}

#[test]
fn is_devshell_block_accepts_devshell_first_line() {
    assert!(is_devshell_block("\ndevShell:\n  package: {}\n"));
}

#[test]
fn is_devshell_block_accepts_leading_whitespace_first_line() {
    assert!(is_devshell_block("  devShell:\n    package: {}\n"));
}

#[test]
fn every_devshell_block_in_shipped_docs_parses() {
    let root = workspace_root();
    let files = discover_doc_files(&root);
    assert!(
        !files.is_empty(),
        "no doc files discovered under {}",
        root.display()
    );

    let mut errors = Vec::<String>::new();
    let mut checked_blocks = 0usize;
    for file in &files {
        let content = fs::read_to_string(file)
            .unwrap_or_else(|e| panic!("read {} failed: {e}", file.display()));
        for (idx, block) in extract_yaml_blocks(&content).into_iter().enumerate() {
            if !is_devshell_block(&block) {
                continue;
            }
            checked_blocks += 1;
            if let Err(e) = serde_yaml::from_str::<DevShellDefinition>(&block) {
                errors.push(format!(
                    "{}: yaml block #{}: {e}\n---\n{block}---",
                    file.display(),
                    idx + 1
                ));
            }
        }
    }

    assert!(
        errors.is_empty(),
        "{} doc-yaml block(s) failed to parse:\n\n{}",
        errors.len(),
        errors.join("\n\n")
    );
    assert!(
        checked_blocks >= MIN_EXPECTED_DEVSHELL_BLOCKS,
        "expected at least {MIN_EXPECTED_DEVSHELL_BLOCKS} devShell blocks across the docs, only found {checked_blocks}"
    );
}

/// When `LAZYNIX_DOC_YAML_DIR` is set the test parses every `*.yaml`
/// file in that directory. This is the hook the shell script uses to
/// hand awk-extracted blocks over to serde. When the env var is unset
/// (as during a plain `cargo test --workspace`) the test is a no-op.
#[test]
fn parses_every_yaml_file_in_env_directory() {
    let Ok(dir_env) = env::var("LAZYNIX_DOC_YAML_DIR") else {
        eprintln!("LAZYNIX_DOC_YAML_DIR not set; skipping directory scan");
        return;
    };
    let dir_path = PathBuf::from(dir_env);
    let mut errors = Vec::<String>::new();
    let mut checked = 0usize;
    for entry in fs::read_dir(&dir_path)
        .unwrap_or_else(|e| panic!("open {} failed: {e}", dir_path.display()))
        .flatten()
    {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "yaml") {
            checked += 1;
            let content = fs::read_to_string(&path).unwrap();
            if let Err(e) = serde_yaml::from_str::<DevShellDefinition>(&content) {
                errors.push(format!("{}: {e}", path.display()));
            }
        }
    }
    assert!(
        errors.is_empty(),
        "{} yaml file(s) under {} failed to parse:\n{}",
        errors.len(),
        dir_path.display(),
        errors.join("\n")
    );
    assert!(
        checked > 0,
        "LAZYNIX_DOC_YAML_DIR={} is set but contained no *.yaml files",
        dir_path.display()
    );
    eprintln!(
        "verified {checked} yaml file(s) under {}",
        dir_path.display()
    );
}

//! Verifies the bundled `lazynix.yaml.template` documents the pinned-package
//! example so that users can uncomment it into a working configuration. This
//! lives separately from the `FsProjectScaffolder` unit tests because the
//! concern (template *content* correctness) is independent of the scaffolder
//! (which owns *how* the file is written).

use lnix_domain::PinnedPackageEntry;

const YAML_TEMPLATE: &str = include_str!("../templates/lazynix.yaml.template");

fn extract_pinned_example_yaml() -> String {
    let (_, after) = YAML_TEMPLATE
        .split_once("    pinned: []\n")
        .expect("template must contain `pinned: []`");
    after
        .lines()
        .map_while(|line| line.strip_prefix("    # "))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn yaml_template_documents_pinned_usage() {
    assert!(YAML_TEMPLATE.contains("#   - name: go"));
    assert!(YAML_TEMPLATE.contains("#     version: \"1.21.13\""));
    assert!(YAML_TEMPLATE.contains("`name` and `version`"));
    assert!(YAML_TEMPLATE.contains("NOT a single string"));
    assert!(YAML_TEMPLATE.contains("lnix search"));
    assert!(YAML_TEMPLATE.contains("replace `pinned: []`"));

    let example = extract_pinned_example_yaml();
    let entries: Vec<PinnedPackageEntry> =
        serde_yaml::from_str(&example).expect("pinned example must parse");

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].name.as_str(), "go");
    assert_eq!(entries[0].version.as_str(), "1.21.13");
}

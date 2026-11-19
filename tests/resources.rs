//! `resources/` is a verbatim copy of Datafaker's locale files (`scripts/sync-resources.sh`), and
//! `tests/resources/` of the YAML files of its tests; this fails when a copy drifts from `upstream/datafaker`.
//! Skipped when the submodule is not checked out.

use std::path::Path;

#[test]
fn resources_match_upstream() {
    let upstream = Path::new(env!("CARGO_MANIFEST_DIR")).join("upstream/datafaker/src/main/resources");
    if !upstream.join("en.yml").exists() {
        eprintln!("upstream/datafaker not checked out; skipped");
        return;
    }
    let mut count = 0;
    for (rel, content) in datafaker::resources::FILES {
        let original = std::fs::read_to_string(upstream.join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"));
        assert!(original == *content, "{rel} differs from upstream; run scripts/sync-resources.sh");
        count += 1;
    }
    assert_eq!(count, 391);
}

#[test]
fn every_resource_parses() {
    for (rel, content) in datafaker::resources::FILES {
        datafaker::yaml::load(content).unwrap_or_else(|e| panic!("{rel}: {e}"));
    }
}

#[test]
fn test_resources_match_upstream() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let upstream = root.join("upstream/datafaker/src/test");
    if !upstream.join("resources/test.yml").exists() {
        eprintln!("upstream/datafaker not checked out; skipped");
        return;
    }
    for (copy, original) in [("test.yml", "resources/test.yml"), ("test_EE.yml", "resources/test_EE.yml"), ("ants.yml", "ants.yml"), ("bees.yml", "bees.yml")] {
        let copy_text = std::fs::read_to_string(root.join("tests/resources").join(copy)).unwrap();
        let original_text = std::fs::read_to_string(upstream.join(original)).unwrap();
        assert!(copy_text == original_text, "tests/resources/{copy} differs from upstream/datafaker/src/test/{original}");
    }
}

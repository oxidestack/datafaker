//! `NoDuplicatesInYmlTest` (Datafaker 2.7.0): no list in a YAML file holds the same value twice, except the lists
//! that repeat entries on purpose to weight them. Upstream walks every `*.yml` under `src`: the bundled files
//! (`resources/` here) and the four of its tests (`tests/resources/`).

use std::collections::{BTreeMap, BTreeSet, HashSet};

use datafaker::yaml::Value;

/// For some data, duplicates make common values (`Karel Spiel`) more likely than rare ones.
fn are_duplicates_allowed(path: &str) -> bool {
    path.ends_with("/faker/name/name")
        || path.ends_with("/faker/name/name_with_middle")
        || path.ends_with("/faker/job/title")
        || path.ends_with("/faker/company/name")
}

fn find_duplicates_recursively(node: &Value, path: &str, errors: &mut BTreeMap<String, BTreeSet<String>>) {
    match node {
        Value::Map(map) => {
            for (key, value) in map {
                find_duplicates_recursively(value, &format!("{path}/{key}"), errors);
            }
        }
        Value::List(list) => {
            let mut seen = HashSet::new();
            for value in list {
                // `HashSet.add` on Java values: the debug form tells types and contents apart as `equals` does.
                if !seen.insert(format!("{value:?}")) && !are_duplicates_allowed(path) {
                    errors.entry(path.to_string()).or_default().insert(value.to_java_string());
                }
                find_duplicates_recursively(value, &format!("{path}[]"), errors);
            }
        }
        _ => {}
    }
}

fn check(file: &str, text: &str) -> Option<String> {
    let content = datafaker::yaml::load(text).unwrap_or_else(|e| panic!("Failed to parse {file}: {e}"))?;
    let mut errors = BTreeMap::new();
    find_duplicates_recursively(&content, "", &mut errors);
    (!errors.is_empty()).then(|| format!("Duplicates in file {file}: {errors:?}"))
}

#[test]
fn no_duplicates_in_arrays() {
    let mut files: Vec<(String, String)> = datafaker::resources::FILES.iter().map(|(p, c)| (format!("resources/{p}"), c.to_string())).collect();
    for name in ["test.yml", "test_EE.yml", "ants.yml", "bees.yml"] {
        let path = format!("{}/tests/resources/{name}", env!("CARGO_MANIFEST_DIR"));
        files.push((path.clone(), std::fs::read_to_string(&path).unwrap()));
    }
    assert_eq!(files.len(), 395, "No *.yml files found");
    let errors: Vec<String> = files.iter().filter_map(|(file, text)| check(file, text)).collect();
    assert!(errors.is_empty(), "{}", errors.join("\n"));
}

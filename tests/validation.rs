use rewrite_internet::check::{SECTIONS, check};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
};
use uuid::Uuid;
struct Fixture {
    root: PathBuf,
    spec: Value,
    evidence: Value,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("ri-rust-check-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let spec = json!({"id":"SPEC-SYNTHETIC","version":"0.1.0","status":"draft","risk":"normal","task":"F001","requirements":["SYN-001","SYN-002"],"sources":["https://www.rfc-editor.org/"],"decisions":[]});
        let evidence = json!({"id":"EVIDENCE-SYNTHETIC","task":"F001","spec":"specifications/synthetic.json","spec_version":"0.1.0","status":"proposed","source_commit":null,"checks":[{"requirement":"SYN-001","method":"property-test","procedure":"Run synthetic cases","result":"not-run","artifact":null},{"requirement":"SYN-002","method":"property-test","procedure":"Run synthetic cases","result":"not-run","artifact":null}],"independent_review":null,"limitations":["Plan only"]});
        let f = Self {
            root,
            spec,
            evidence,
        };
        f.put("tasks/phase-1.json", &json!([{"id":"F001","priority":"P0","title":"Synthetic task","artifact":"specifications/synthetic.md","acceptance":["Positive case","Negative case"]}]));
        f.sync();
        f.write("specifications/synthetic.md", &format!("# Synthetic\n\n{}\n\n### SYN-001\nMUST reject invalid input.\n\n### SYN-002\nMUST preserve bytes.\n", SECTIONS.iter().map(|s|format!("## {s}")).collect::<Vec<_>>().join("\n\n")));
        f
    }
    fn put(&self, file: &str, value: &Value) {
        self.write(file, &serde_json::to_string_pretty(value).unwrap());
    }
    fn write(&self, file: &str, text: &str) {
        let path = self.root.join(file);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    fn sync(&self) {
        self.put("specifications/synthetic.json", &self.spec);
        self.put("evidence/synthetic.json", &self.evidence);
    }
    fn errors(&self) -> String {
        check(&self.root).unwrap().errors.join("\n")
    }
    fn measured(&mut self) {
        self.evidence["status"] = json!("measured");
        self.evidence["source_commit"] = json!("a".repeat(40));
        for c in self.evidence["checks"].as_array_mut().unwrap() {
            c["result"] = json!("fail");
            c["artifact"] = json!("evidence/artifacts/synthetic.txt");
        }
    }
}
#[test]
fn accepts_complete_proposed_plan() {
    let f = Fixture::new();
    assert_eq!(f.errors(), "");
}
#[test]
fn rejects_missing_coverage() {
    let mut f = Fixture::new();
    f.evidence["checks"].as_array_mut().unwrap().pop();
    f.sync();
    assert!(f.errors().contains("uncovered requirement SYN-002"));
}
#[test]
fn rejects_stale_version() {
    let mut f = Fixture::new();
    f.evidence["spec_version"] = json!("0.0.1");
    f.sync();
    assert!(f.errors().contains("stale spec version"));
}
#[test]
fn rejects_false_measurement() {
    let mut f = Fixture::new();
    f.evidence["status"] = json!("measured");
    f.sync();
    assert!(f.errors().contains("measured check requires"));
}
#[test]
fn accepts_measured_failure_with_artifacts() {
    let mut f = Fixture::new();
    f.measured();
    f.write(
        "evidence/artifacts/synthetic.txt",
        "Synthetic failure log\n",
    );
    f.sync();
    assert_eq!(f.errors(), "");
}
#[test]
fn rejects_artifact_traversal() {
    let mut f = Fixture::new();
    f.measured();
    f.evidence["checks"][0]["artifact"] = json!("evidence/artifacts/../../../outside.txt");
    f.sync();
    assert!(f.errors().contains("unsafe, empty or missing artifact"));
}
#[test]
fn rejects_missing_independent_review() {
    let mut f = Fixture::new();
    f.evidence["status"] = json!("independently-reproduced");
    f.sync();
    assert!(f.errors().contains("independent review reference"));
}
#[test]
fn rejects_reviewed_spec_without_reference() {
    let mut f = Fixture::new();
    f.spec["status"] = json!("reviewed");
    f.sync();
    assert!(f.errors().contains("review"));
}
#[test]
fn candidate_schema_cannot_weaken_embedded_contract() {
    let mut f = Fixture::new();
    f.put("schemas/specification.schema.json", &json!({}));
    f.spec.as_object_mut().unwrap().remove("requirements");
    f.sync();
    assert!(f.errors().contains("requirements"));
}
#[test]
fn rejects_broken_local_link() {
    let f = Fixture::new();
    f.write("README.md", "[missing](docs/missing.md)");
    assert!(f.errors().contains("broken or unsafe local link"));
}
#[test]
fn rejects_malformed_metadata_without_panicking() {
    let f = Fixture::new();
    f.put(
        "specifications/synthetic.json",
        &json!({"requirements":"wrong-type"}),
    );
    assert!(!check(&f.root).unwrap().ok);
}
#[test]
fn rejects_duplicate_task_and_empty_artifact() {
    let mut f = Fixture::new();
    f.put(
        "tasks/phase-1.json",
        &json!([{"id":"F001","priority":"P0","title":"Task","artifact":"","acceptance":["a","b"]}]),
    );
    f.measured();
    f.write("evidence/artifacts/synthetic.txt", " \n");
    f.sync();
    assert!(f.errors().contains("Invalid or duplicate task"));
    assert!(f.errors().contains("unsafe, empty or missing artifact"));
}
#[cfg(unix)]
#[test]
fn rejects_symlinks_and_links_through_them() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    symlink(std::env::temp_dir(), f.root.join("escape")).unwrap();
    f.write("README.md", "[escape](escape)");
    let e = f.errors();
    assert!(e.contains("Symlink"));
    assert!(e.contains("broken or unsafe local link"));
}
#[test]
fn repository_contract_is_consistent() {
    assert!(check(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap().ok);
}

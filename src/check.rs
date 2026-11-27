use regex::Regex;
use serde::Serialize;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path},
};

const SPEC: &str = include_str!("../schemas/specification.schema.json");
const EVIDENCE: &str = include_str!("../schemas/evidence.schema.json");
pub const SECTIONS: &[&str] = &[
    "Intent and users",
    "Standards and existing behavior",
    "Scope and non-goals",
    "Requirements",
    "Security boundaries",
    "Data lifecycle",
    "Performance",
    "Acceptance and evidence",
    "Open decisions",
];

#[derive(Debug, Serialize)]
pub struct Report {
    pub ok: bool,
    pub errors: Vec<String>,
    pub specifications: usize,
    pub evidence: usize,
    pub files: usize,
}
fn walk(
    root: &Path,
    dir: &Path,
    files: &mut BTreeSet<String>,
    errors: &mut Vec<String>,
    depth: usize,
) -> Result<(), String> {
    if depth > 32 {
        return Err("Directory depth exceeds bootstrap limit".into());
    }
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name();
        if [".git", "node_modules", "target", ".participation"]
            .iter()
            .any(|n| name == *n)
        {
            continue;
        }
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        let kind = entry.file_type().map_err(|e| e.to_string())?;
        if kind.is_symlink() {
            errors.push(format!("Symlink is not allowed: {relative}"));
        } else if kind.is_dir() {
            walk(root, &path, files, errors, depth + 1)?;
        } else if kind.is_file() {
            if entry.metadata().map_err(|e| e.to_string())?.len() > 1024 * 1024 {
                errors.push(format!("File exceeds bootstrap size limit: {relative}"));
            } else {
                files.insert(relative);
            }
            if files.len() > 4096 {
                return Err("Too many files for bootstrap validator".into());
            }
        }
    }
    Ok(())
}
fn read(root: &Path, file: &str) -> Result<String, String> {
    fs::read_to_string(root.join(file)).map_err(|e| format!("{file}: {e}"))
}
fn json(root: &Path, file: &str, errors: &mut Vec<String>) -> Option<Value> {
    match read(root, file)
        .and_then(|s| serde_json::from_str(&s).map_err(|e| format!("{file}: {e}")))
    {
        Ok(v) => Some(v),
        Err(e) => {
            errors.push(e);
            None
        }
    }
}
fn validate(
    schema: &jsonschema::Validator,
    file: &str,
    value: &Value,
    errors: &mut Vec<String>,
) -> bool {
    let mut valid = true;
    for e in schema.iter_errors(value) {
        valid = false;
        errors.push(format!("{file}: {e}"));
    }
    valid
}
fn text(value: &Value) -> &str {
    value.as_str().unwrap_or("")
}
fn leaf(file: &str, dir: &str) -> bool {
    file.strip_prefix(dir)
        .is_some_and(|s| !s.contains('/') && s.ends_with(".json"))
}
fn percent_decode(s: &str) -> Option<String> {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' {
            let h = std::str::from_utf8(b.get(i + 1..i + 3)?).ok()?;
            out.push(u8::from_str_radix(h, 16).ok()?);
            i += 3;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}
fn local_target(file: &str, link: &str) -> Option<String> {
    if link.contains('\\') || link.starts_with('/') {
        return None;
    }
    let mut parts: Vec<&str> = file.split('/').collect();
    parts.pop();
    for component in Path::new(link).components() {
        match component {
            Component::Normal(s) => parts.push(s.to_str()?),
            Component::ParentDir => {
                parts.pop()?;
            }
            Component::CurDir => {}
            _ => return None,
        }
    }
    Some(parts.join("/"))
}
pub fn check(root: &Path) -> Result<Report, String> {
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let mut errors = Vec::new();
    let mut files = BTreeSet::new();
    walk(&root, &root, &mut files, &mut errors, 0)?;
    let spec_schema =
        jsonschema::validator_for(&serde_json::from_str::<Value>(SPEC).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let evidence_schema = jsonschema::validator_for(
        &serde_json::from_str::<Value>(EVIDENCE).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let task_pattern = Regex::new(r"^F[0-9]{3}$").unwrap();
    let tasks = json(&root, "tasks/phase-1.json", &mut errors).unwrap_or(Value::Null);
    let mut task_ids = BTreeSet::new();
    if let Some(tasks) = tasks.as_array() {
        for t in tasks {
            let id = text(&t["id"]);
            if !task_pattern.is_match(id)
                || !task_ids.insert(id.to_string())
                || !["P0", "P1"].contains(&text(&t["priority"]))
                || text(&t["title"]).trim().is_empty()
                || text(&t["artifact"]).trim().is_empty()
                || !t["acceptance"]
                    .as_array()
                    .is_some_and(|a| a.len() >= 2 && a.iter().all(|v| !text(v).trim().is_empty()))
            {
                errors.push(format!("Invalid or duplicate task: {id}"));
            }
        }
    } else {
        errors.push("tasks/phase-1.json must be an array".into());
    }
    let mut specs = BTreeMap::new();
    let mut ids = BTreeSet::new();
    for file in files.iter().filter(|f| leaf(f, "specifications/")) {
        let Some(s) = json(&root, file, &mut errors) else {
            continue;
        };
        if !validate(&spec_schema, file, &s, &mut errors) {
            continue;
        }
        if !ids.insert(text(&s["id"]).to_string()) {
            errors.push(format!("{file}: duplicate specification ID"));
        }
        if !task_ids.contains(text(&s["task"])) {
            errors.push(format!("{file}: unknown task"));
        }
        let md = file.trim_end_matches(".json").to_owned() + ".md";
        if !files.contains(&md) {
            errors.push(format!("{file}: matching Markdown is missing"));
        } else {
            let prose = read(&root, &md)?;
            for section in SECTIONS {
                if !prose.lines().any(|l| l == format!("## {section}")) {
                    errors.push(format!("{md}: missing section {section}"));
                }
            }
            for id in s["requirements"].as_array().unwrap() {
                let prefix = format!("### {}", text(id));
                if !prose.lines().any(|l| {
                    l.strip_prefix(&prefix).is_some_and(|rest| {
                        rest.is_empty() || rest.starts_with(char::is_whitespace)
                    })
                }) {
                    errors.push(format!("{md}: missing requirement {}", text(id)));
                }
            }
        }
        specs.insert(file.clone(), s);
    }
    let mut covered = BTreeSet::new();
    let mut evidence_ids = BTreeSet::new();
    for file in files.iter().filter(|f| leaf(f, "evidence/")) {
        let Some(e) = json(&root, file, &mut errors) else {
            continue;
        };
        if !validate(&evidence_schema, file, &e, &mut errors) {
            continue;
        }
        if !evidence_ids.insert(text(&e["id"]).to_string()) {
            errors.push(format!("{file}: duplicate evidence ID"));
        }
        let Some(s) = specs.get(text(&e["spec"])) else {
            errors.push(format!("{file}: unknown spec"));
            continue;
        };
        covered.insert(text(&e["spec"]).to_string());
        if e["spec_version"] != s["version"] {
            errors.push(format!("{file}: stale spec version"));
        }
        if e["task"] != s["task"] {
            errors.push(format!("{file}: evidence and spec task differ"));
        }
        let mut seen = BTreeSet::new();
        let requirements = s["requirements"].as_array().unwrap();
        for c in e["checks"].as_array().unwrap() {
            if !requirements.contains(&c["requirement"]) {
                errors.push(format!(
                    "{file}: unknown requirement {}",
                    text(&c["requirement"])
                ));
            }
            seen.insert(text(&c["requirement"]));
            if e["status"] == "proposed" {
                if c["result"] != "not-run" || !c["artifact"].is_null() {
                    errors.push(format!(
                        "{file}: proposed checks must be not-run with no result artifact"
                    ));
                }
            } else if e["source_commit"].is_null()
                || !["pass", "fail"].contains(&text(&c["result"]))
                || !c["artifact"].is_string()
            {
                errors.push(format!(
                    "{file}: measured check requires source, result and artifact"
                ));
            } else {
                let artifact = text(&c["artifact"]);
                let safe = artifact.starts_with("evidence/artifacts/")
                    && artifact.split('/').all(|p| !["", ".", ".."].contains(&p))
                    && !artifact.contains('\\')
                    && files.contains(artifact);
                if !safe
                    || !fs::read(root.join(artifact))
                        .unwrap_or_default()
                        .iter()
                        .any(|b| !b.is_ascii_whitespace())
                {
                    errors.push(format!("{file}: unsafe, empty or missing artifact"));
                }
            }
        }
        for id in requirements {
            if !seen.contains(text(id)) {
                errors.push(format!("{file}: uncovered requirement {}", text(id)));
            }
        }
        if e["status"] == "independently-reproduced" && e["independent_review"].is_null() {
            errors.push(format!("{file}: independent review reference required"));
        }
    }
    for file in specs.keys() {
        if !covered.contains(file) {
            errors.push(format!("{file}: evidence plan required"));
        }
    }
    let links = Regex::new(r#"\[[^\]]*\]\(([^\s)]+)(?:\s+"[^"]*")?\)"#).unwrap();
    for file in files.iter().filter(|f| f.ends_with(".md")) {
        for m in links.captures_iter(&read(&root, file)?) {
            let link = m[1].trim_matches(['<', '>']);
            if ["https:", "http:", "mailto:", "#"]
                .iter()
                .any(|p| link.starts_with(p))
            {
                continue;
            }
            let Some(decoded) = percent_decode(link.split('#').next().unwrap_or("")) else {
                errors.push(format!("{file}: invalid link encoding"));
                continue;
            };
            let target = local_target(file, &decoded);
            if !target.is_some_and(|t| {
                let p = root.join(t);
                p.canonicalize()
                    .is_ok_and(|absolute| absolute.starts_with(&root) && absolute != root)
            }) {
                errors.push(format!("{file}: broken or unsafe local link {link}"));
            }
        }
    }
    Ok(Report {
        ok: errors.is_empty(),
        errors,
        specifications: specs.len(),
        evidence: covered.len(),
        files: files.len(),
    })
}

use crate::*;
use std::{fs, path::Path};
pub fn discover(roots: &[String]) -> Result<Vec<Skill>> {
    let mut found = vec![];
    for root in roots {
        let base = fs::canonicalize(root)?;
        scan(&base, 0, &mut found)?;
    }
    found.sort_by(|a, b| a.source.cmp(&b.source));
    found.dedup_by(|a, b| a.digest == b.digest);
    Ok(found)
}
fn scan(dir: &Path, depth: usize, result: &mut Vec<Skill>) -> Result<()> {
    if depth > 3 || result.len() >= 200 {
        return Ok(());
    }
    for item in fs::read_dir(dir)?.take(300) {
        let item = item?;
        let kind = item.file_type()?;
        if kind.is_symlink() {
            continue;
        }
        if kind.is_dir() {
            scan(&item.path(), depth + 1, result)?;
        } else if item.file_name() == "SKILL.md" && item.metadata()?.len() <= 65536 {
            let content = fs::read_to_string(item.path())?;
            let field = |key: &str| {
                content
                    .lines()
                    .take(35)
                    .find_map(|l| l.strip_prefix(&format!("{key}:")))
                    .map(|s| s.trim().trim_matches('"').to_string())
            };
            let name = field("name").unwrap_or_else(|| {
                dir.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned()
            });
            let description = field("description").unwrap_or_default();
            let mut supporting = Vec::new();
            archive_files(dir, dir, 0, &mut supporting)?;
            let hash =
                files::digest(serde_json::to_vec(&(content.clone(), &supporting))?.as_slice());
            // Arbitrary downloaded scripts and external service bindings are not executable capabilities.
            let limitation = capability_limitation(&content, &supporting);
            result.push(Skill {
                id: hash.clone(),
                name,
                description,
                source: item.path().display().to_string(),
                digest: hash,
                content,
                files: supporting,
                available: limitation.is_none(),
                limitation,
            });
        }
    }
    Ok(())
}

fn archive_files(
    base: &Path,
    dir: &Path,
    depth: usize,
    files: &mut Vec<(String, String)>,
) -> Result<()> {
    if depth > 3 || files.len() >= 32 {
        return Ok(());
    }
    for entry in fs::read_dir(dir)?.take(100) {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_symlink() {
            continue;
        }
        if kind.is_dir() {
            archive_files(base, &entry.path(), depth + 1, files)?;
        } else if entry.file_name() != "SKILL.md" && entry.metadata()?.len() <= 65536 {
            if files.iter().map(|(_, s)| s.len()).sum::<usize>() > 512 * 1024 {
                break;
            }
            if let Ok(content) = fs::read_to_string(entry.path()) {
                files.push((
                    entry
                        .path()
                        .strip_prefix(base)
                        .unwrap()
                        .display()
                        .to_string(),
                    content,
                ));
            }
        }
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(())
}

pub fn select(catalog: &[Skill], role: &str) -> Vec<String> {
    let terms: Vec<&str> = match role.to_lowercase().as_str() {
        "mayor" => vec!["planning", "delegation", "coordinat", "unslop"],
        "writer" | "editor" => vec!["writing", "editing", "unslop"],
        "builder" | "tester" => vec!["coding", "testing", "debug", "unslop"],
        "scout" | "reader" | "analyst" => vec!["research", "analysis", "unslop"],
        _ => vec!["unslop"],
    };
    let mut selected = vec![];
    let mut bytes = 0;
    let mut seen = std::collections::HashSet::new();
    for skill in catalog
        .iter()
        .rev()
        .filter(|s| seen.insert(s.source.clone()) && s.available)
    {
        let text = format!("{} {}", skill.name, skill.description).to_lowercase();
        if terms.iter().any(|t| text.contains(t))
            && bytes + skill.content.len() < 16000
            && selected.len() < 4
        {
            bytes += skill.content.len();
            selected.push(skill.id.clone());
        }
    }
    selected
}

const SUPPORTED_TOOLS: &[&str] = &[
    "list_files",
    "read_file",
    "write_file",
    "read_skill",
    "list_skills",
    "delegate",
    "send_message",
    "remember",
    "review_result",
    "found_city",
    "manage_residents",
];
pub fn assignment(
    catalog: &[Skill],
    role: &str,
    scope: &str,
    requested: Option<&serde_json::Value>,
) -> Result<Vec<String>> {
    let mut seen = std::collections::HashSet::new();
    let latest: Vec<_> = catalog
        .iter()
        .rev()
        .filter(|s| seen.insert(s.source.clone()))
        .collect();
    let mut chosen = Vec::new();
    let mut bytes = 0;
    if let Some(value) = requested.filter(|v| !v.is_null()) {
        let ids = value.as_array().ok_or_else(|| {
            CityError::new(
                "invalid_skillset",
                "skill_ids must be an array of catalog version IDs.",
            )
        })?;
        for value in ids {
            let skill=latest.iter().find(|s|Some(s.id.as_str())==value.as_str()&&s.available).ok_or_else(||CityError::new("missing_capability","A requested skill version is unavailable, outdated, or requires unsupported tools."))?;
            if !chosen.contains(&skill.id) {
                chosen.push(skill.id.clone());
                bytes += skill.content.len();
            }
        }
        if chosen.len() > 4 || bytes > 16000 {
            return Err(CityError::new(
                "skill_budget",
                "Choose at most four packages within the 16,000-byte instruction budget.",
            ));
        }
    }
    let terms: Vec<_> = scope
        .split(|c: char| !c.is_alphanumeric())
        .filter(|term| term.len() > 4)
        .map(str::to_lowercase)
        .filter(|term| {
            ![
                "project",
                "should",
                "using",
                "their",
                "there",
                "these",
                "which",
                "provide",
                "result",
                "results",
                "criteria",
                "instructions",
            ]
            .contains(&term.as_str())
        })
        .collect();
    let defaults = select(catalog, role);
    let mut candidates: Vec<_> = latest
        .into_iter()
        .filter(|skill| skill.available)
        .map(|skill| {
            let text = format!("{} {}", skill.name, skill.description).to_lowercase();
            let score = usize::from(defaults.contains(&skill.id)) * 100
                + terms
                    .iter()
                    .filter(|term| text.contains(term.as_str()))
                    .count();
            (score, skill)
        })
        .filter(|(score, _)| *score > 0)
        .collect();
    candidates.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.id.cmp(&b.1.id)));
    for (_, skill) in candidates {
        if chosen.len() < 4 && bytes + skill.content.len() <= 16000 && !chosen.contains(&skill.id) {
            chosen.push(skill.id.clone());
            bytes += skill.content.len();
        }
    }
    Ok(chosen)
}

pub fn capability_limitation(content: &str, supporting: &[(String, String)]) -> Option<String> {
    let unsupported_text = [
        "app://",
        "mcp__",
        "curl ",
        "pip install",
        "npm install",
        "exec_command",
        "web.run",
        "web__run",
        "cua.",
        "python3 ",
        "python ",
    ]
    .iter()
    .any(|token| content.contains(token));
    let executable_helpers = supporting.iter().any(|(name, _)| {
        [".py", ".sh", ".js", ".ts", ".rb"]
            .iter()
            .any(|suffix| name.ends_with(suffix))
    });
    let declared = content
        .lines()
        .take(35)
        .find_map(|line| line.strip_prefix("requires_tools:"))
        .map(str::trim);
    let declared_supported = declared.is_none_or(|value| {
        value.starts_with('[')
            && value.ends_with(']')
            && value[1..value.len() - 1]
                .split(',')
                .map(|s| s.trim().trim_matches(['\'', '"']))
                .filter(|s| !s.is_empty())
                .all(|name| SUPPORTED_TOOLS.contains(&name))
    });
    (unsupported_text||executable_helpers||!declared_supported).then(||"This package declares unsupported tools, executable helpers, or unverified external dependencies. Its files remain inspectable, but it is not assigned for execution.".into())
}

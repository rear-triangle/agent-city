use crate::*;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    path::{Component, Path, PathBuf},
};
pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let temp = path.with_extension(format!("{}.tmp", id()));
    let mut f = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temp)?;
    f.write_all(bytes)?;
    f.sync_all()?;
    fs::rename(&temp, path)?;
    Ok(())
}
pub fn workspace_path(root: &str, relative: &str, must_exist: bool) -> Result<PathBuf> {
    let rel = Path::new(relative);
    if rel
        .components()
        .any(|c| matches!(c, Component::ParentDir | Component::Prefix(_)))
    {
        return Err(CityError::new(
            "denied_policy",
            "Use a path within the approved project folder without parent traversal.",
        ));
    }
    let base = fs::canonicalize(root)?;
    let path = base.join(rel);
    let checked = if must_exist || path.exists() {
        fs::canonicalize(&path)?
    } else {
        let parent = path
            .parent()
            .ok_or_else(|| CityError::new("invalid_path", "Missing parent folder."))?;
        fs::canonicalize(parent)?.join(
            path.file_name()
                .ok_or_else(|| CityError::new("invalid_path", "Missing filename."))?,
        )
    };
    if !checked.starts_with(&base) {
        return Err(CityError::new(
            "denied_policy",
            "This path leaves the approved project folder.",
        ));
    }
    if checked.components().any(|c| c.as_os_str() == ".git") {
        return Err(CityError::new(
            "denied_policy",
            "Direct Git metadata changes are not supported.",
        ));
    }
    Ok(checked)
}
pub fn read(root: &str, path: &str) -> Result<String> {
    let target = workspace_path(root, path, true)?;
    if fs::metadata(&target)?.len() > 128 * 1024 {
        return Err(CityError::new(
            "size_limit",
            "Read a file smaller than 128 KiB.",
        ));
    }
    Ok(fs::read_to_string(target)?)
}
pub fn list(root: &str, path: &str) -> Result<Vec<String>> {
    let target = workspace_path(root, path, true)?;
    let mut entries = vec![];
    for entry in fs::read_dir(target)?.take(200) {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name == ".git" || name == "node_modules" || name == ".env" {
            continue;
        }
        entries.push(format!(
            "{}{}",
            name,
            if entry.file_type()?.is_dir() { "/" } else { "" }
        ));
    }
    entries.sort();
    Ok(entries)
}

//! One extension process serializes transactions. No profile migration or automatic retry.
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
    path::Path,
};
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Disk {
    version: u8,
    selected: Vec<String>,
}
pub type Snapshot = (Option<Vec<u8>>, Vec<String>);
pub fn read(dir: &Path) -> Result<Snapshot, String> {
    let path = dir.join("selected-timezones.json");
    let mut file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok((None, vec![])),
        Err(e) => return Err(format!("Cannot read selected timezones: {e}")),
    };
    let mut bytes = Vec::new();
    std::io::Read::by_ref(&mut file)
        .take(65537)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 65536 {
        return Err(
            "Selected-timezone storage exceeds 65536 bytes; previous bytes preserved".into(),
        );
    }
    let d: Disk = serde_json::from_slice(&bytes)
        .map_err(|e| format!("Corrupt selected-timezone storage; previous bytes preserved: {e}"))?;
    if d.version != 1 || d.selected.len() > crate::time::catalog().len() {
        return Err("Unsupported selected-timezone storage version or count".into());
    }
    let mut seen = std::collections::HashSet::new();
    if d.selected
        .iter()
        .any(|z| !crate::time::catalog().contains(z) || !seen.insert(z))
    {
        return Err("Selected-timezone storage contains unknown or duplicate zones; previous bytes preserved".into());
    }
    Ok((Some(bytes), d.selected))
}
pub fn write(
    dir: &Path,
    expected: &Option<Vec<u8>>,
    selected: &[String],
) -> Result<Option<Vec<u8>>, String> {
    let (actual, _) = read(dir)?;
    if &actual != expected {
        return Err("Selected timezones changed in another session. Reload Saved Timezones, then deliberately repeat the change".into());
    }
    let bytes = serde_json::to_vec(&Disk {
        version: 1,
        selected: selected.to_vec(),
    })
    .map_err(|e| e.to_string())?;
    fs::create_dir_all(dir).map_err(|e| format!("Cannot create extension data directory: {e}"))?;
    let mut tmp = tempfile::NamedTempFile::new_in(dir)
        .map_err(|e| format!("Cannot create atomic storage replacement: {e}"))?;
    tmp.write_all(&bytes)
        .and_then(|()| tmp.as_file().sync_all())
        .map_err(|e| format!("Cannot write atomic storage replacement: {e}"))?;
    tmp.persist(dir.join("selected-timezones.json"))
        .map_err(|e| format!("Cannot replace selected-timezone storage: {e}"))?;
    // No fallible operation after rename: failed writes leave previous bytes untouched.
    Ok(Some(bytes))
}

use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use crate::error::{EchoAccessError, Result};
use crate::sync::state::FileState;

fn should_skip_dir_name(name: &OsStr) -> bool {
    let lossy = name.to_string_lossy();
    if lossy.starts_with('.') {
        return true;
    }
    name == OsStr::new("node_modules")
        || name == OsStr::new("target")
        || name == OsStr::new("__pycache__")
}

fn rel_path_str(root_base: &Path, full: &Path) -> Result<String> {
    let rel = full.strip_prefix(root_base).map_err(|_| {
        EchoAccessError::Sync(format!("Path outside scan root: {}", full.display()))
    })?;
    Ok(rel.to_string_lossy().replace('\\', "/"))
}

fn walk(
    root_base: &Path,
    dir: &Path,
    depth: usize,
    max_depth: usize,
    max_files: usize,
    out: &mut Vec<FileState>,
) -> Result<()> {
    if out.len() >= max_files {
        return Ok(());
    }

    let mut entries: Vec<_> = fs::read_dir(dir)?.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        if out.len() >= max_files {
            return Ok(());
        }

        let path = entry.path();
        let file_type = entry.file_type()?;

        if file_type.is_symlink() {
            continue;
        }

        let name = entry.file_name();

        if file_type.is_dir() {
            if should_skip_dir_name(&name) {
                continue;
            }
            if depth < max_depth {
                walk(root_base, &path, depth + 1, max_depth, max_files, out)?;
            }
            continue;
        }

        if !file_type.is_file() {
            continue;
        }

        let path_str = rel_path_str(root_base, &path)?;
        out.push(FileState::new(path_str, String::new()));
    }

    Ok(())
}

/// Recursively scans `root` for regular files, skipping hidden directories (names starting with
/// `.`), `node_modules`, `target`, and `__pycache__`.
///
/// - `max_depth`: maximum subdirectory depth to descend from `root` (0 = files in `root` only).
/// - `max_files`: stops after this many files have been collected.
pub fn scan_directory(root: &Path, max_depth: usize, max_files: usize) -> Result<Vec<FileState>> {
    if max_files == 0 {
        return Ok(Vec::new());
    }

    let meta = fs::metadata(root)?;
    if !meta.is_dir() {
        return Err(EchoAccessError::Sync(format!(
            "Not a directory: {}",
            root.display()
        )));
    }

    let root_base = fs::canonicalize(root)?;
    let mut out = Vec::new();
    walk(&root_base, &root_base, 0, max_depth, max_files, &mut out)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn scan_finds_files_at_root() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("a.txt");
        std::fs::write(&p, b"x").unwrap();

        let files = scan_directory(dir.path(), 3, 100).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "a.txt");
        assert_eq!(files[0].status, crate::sync::state::SyncStatus::New);
    }

    #[test]
    fn scan_nested_respects_max_depth() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("l1/l2/l3")).unwrap();
        std::fs::write(dir.path().join("root.txt"), b"1").unwrap();
        std::fs::write(dir.path().join("l1/a.txt"), b"2").unwrap();
        std::fs::write(dir.path().join("l1/l2/b.txt"), b"3").unwrap();
        std::fs::write(dir.path().join("l1/l2/l3/c.txt"), b"4").unwrap();

        let files = scan_directory(dir.path(), 2, 100).unwrap();
        let names: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();
        assert!(names.contains(&"root.txt"));
        assert!(names.contains(&"l1/a.txt"));
        assert!(names.contains(&"l1/l2/b.txt"));
        assert!(!names.contains(&"l1/l2/l3/c.txt"));
    }

    #[test]
    fn scan_respects_max_files() {
        let dir = tempdir().unwrap();
        for i in 0..5 {
            std::fs::write(dir.path().join(format!("f{i}.txt")), b"x").unwrap();
        }
        let files = scan_directory(dir.path(), 3, 2).unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn scan_skips_hidden_and_default_excludes() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("keep.txt"), b"x").unwrap();
        std::fs::create_dir_all(dir.path().join(".git/objects")).unwrap();
        std::fs::write(dir.path().join(".git/objects/x"), b"y").unwrap();
        std::fs::create_dir_all(dir.path().join("node_modules/pkg")).unwrap();
        std::fs::write(dir.path().join("node_modules/pkg/index.js"), b"z").unwrap();
        std::fs::create_dir_all(dir.path().join("target/debug")).unwrap();
        std::fs::write(dir.path().join("target/debug/app"), b"z").unwrap();
        std::fs::create_dir_all(dir.path().join("__pycache__")).unwrap();
        std::fs::write(dir.path().join("__pycache__/x.pyc"), b"z").unwrap();

        let files = scan_directory(dir.path(), 5, 100).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "keep.txt");
    }

    #[test]
    fn scan_rejects_non_directory() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("notdir");
        std::fs::write(&f, b"x").unwrap();
        let err = scan_directory(&f, 3, 10).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("Not a directory"), "got {msg}");
    }
}

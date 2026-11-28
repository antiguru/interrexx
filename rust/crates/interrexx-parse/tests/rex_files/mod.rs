use std::path::{Path, PathBuf};

/// Every `*.rex` file under `dir`, recursively, in a stable order.
pub fn rex_files_under(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("cannot read directory {}: {e}", dir.display()))
        {
            let path = entry.expect("readable directory entry").path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rex") {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

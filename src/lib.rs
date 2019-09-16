use std::path::{Path, PathBuf};

mod backend;
pub use backend::{create_backend, Backend};

pub fn get_absolute_path(path: &Path, cur_dir: &Path) -> PathBuf {
    use path_clean::PathClean;
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cur_dir.join(path)
    }
    .clean()
}

pub fn normalize_path(path: &Path, new_base: &Path) -> PathBuf {
    assert!(path.is_absolute());
    assert!(new_base.is_absolute());
    pathdiff::diff_paths(&path, &new_base).unwrap_or(path.to_path_buf())
}

use std::path::{Path, PathBuf};

mod backend;
pub use backend::{create_backend, Backend};

pub fn get_absolute_path(path: &Path) -> std::io::Result<PathBuf> {
    use path_clean::PathClean;
    Ok(if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    }
    .clean())
}

pub fn normalize_path(path: &Path, new_base: &Path) -> std::io::Result<PathBuf> {
    let path = get_absolute_path(path)?;
    let new_base = get_absolute_path(new_base)?;
    Ok(match pathdiff::diff_paths(&path, &new_base) {
        Some(x) => x,
        None => path,
    })
}

pub use std::path::Path;
use std::{collections::HashSet, hash::BuildHasher, path::PathBuf};

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
    pathdiff::diff_paths(&path, &new_base).unwrap_or_else(|| path.to_path_buf())
}

pub fn print_tags<H: BuildHasher>(ltitle: &str, tags: &HashSet<String, H>) {
    print!("{}:", ltitle);
    for i in tags {
        print!(" {}", i);
    }
    println!();
}

pub fn iter_dir_nonhid_abs<'a>(
    itdir: &Path,
    cur_dir: &'a Path,
) -> impl Iterator<Item = PathBuf> + 'a {
    fn is_not_hidden(entry: &walkdir::DirEntry) -> bool {
        entry
            .file_name()
            .to_str()
            .map(|s| entry.depth() == 0 || !s.starts_with('.'))
            .unwrap_or(false)
    }

    walkdir::WalkDir::new(itdir)
        .into_iter()
        .filter_entry(|e| is_not_hidden(e))
        .filter_map(|v| v.ok())
        .map(move |v| get_absolute_path(v.path(), cur_dir))
}

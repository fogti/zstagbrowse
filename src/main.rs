use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};
use walkdir::{DirEntry, WalkDir};

enum FoldOp {
    And,
    Or,
}

struct Query {
    foldop: FoldOp,
    tags: HashSet<String>,
}

impl Query {
    fn matches_intern(&self, path: &Path) -> Result<HashSet<String>, failure::Error> {
        match xattr::get(path, "user.zstags")? {
            Some(x) => x
                .split(|x| *x == 0 || *x == b'|')
                .map(|x| Ok(String::from(std::str::from_utf8(x)?)))
                .collect::<Result<HashSet<_>, failure::Error>>(),
            None => Ok(HashSet::new()),
        }
    }

    pub fn matches(&self, path: &Path) -> bool {
        match self.matches_intern(path) {
            Ok(tags_on_file) => {
                let icnt = self.tags.intersection(&tags_on_file).count();
                match self.foldop {
                    FoldOp::And => icnt == self.tags.len(),
                    FoldOp::Or => icnt != 0,
                }
            }
            Err(x) => {
                eprintln!("{}: unable to access xattrs: {:?}", path.display(), x);
                false
            }
        }
    }
}

fn is_not_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| entry.depth() == 0 || !s.starts_with("."))
        .unwrap_or(false)
}

fn normalize_path(path: &Path, new_base: &Path) -> std::io::Result<PathBuf> {
    fn get_absolute_path(path: &Path) -> std::io::Result<PathBuf> {
        use path_clean::PathClean;
        Ok(if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()?.join(path)
        }
        .clean())
    }

    let path = get_absolute_path(path)?;
    let new_base = get_absolute_path(new_base)?;
    Ok(match pathdiff::diff_paths(&path, &new_base) {
        Some(x) => x,
        None => path,
    })
}

fn main() {
    use clap::{crate_version, Arg};

    let matches = clap::App::new("zstagbrowse")
        .version(crate_version!())
        .author("Erik Zscheile <erik.zscheile@gmail.com>")
        .about("tag browser")
        .setting(clap::AppSettings::TrailingVarArg)
        .arg(
            Arg::with_name("source")
                .long("source")
                .takes_value(true)
                .required(true)
                .help("specifies the source tree"),
        )
        .arg(
            Arg::with_name("target")
                .long("target")
                .takes_value(true)
                .required(true)
                .help(
                    "specifies the target tree/directory (will be recreated if it already exists)",
                ),
        )
        .arg(
            Arg::with_name("foldop")
                .long("foldop")
                .short("o")
                .help("specifies the query folding operation (defaults to '&')"),
        )
        .arg(
            Arg::with_name("QUERY")
                .required(true)
                .multiple(true)
                .index(1),
        )
        .get_matches();

    let source: &Path = Path::new(matches.value_of("source").unwrap());
    let target: &Path = Path::new(matches.value_of("target").unwrap());

    let foldop = match matches.value_of("foldop").unwrap_or("&") {
        "&" | "&&" => FoldOp::And,
        "|" | "||" => FoldOp::Or,
        x => panic!("'{}': unknown foldop", x),
    };

    let query = Query {
        foldop,
        tags: matches
            .values_of("QUERY")
            .unwrap()
            .map(Into::into)
            .collect(),
    };

    if !source.is_dir() {
        panic!("{}: is not a directory", source.display());
    }

    match fs::remove_dir_all(target) {
        Ok(_) => {}
        Err(x) if x.kind() == std::io::ErrorKind::NotFound => {}
        Err(x) => panic!(
            "{}: failed to clear target directory: {:?}",
            target.display(),
            x
        ),
    }
    fs::create_dir_all(target).expect("failed to create target directory");

    let selfiles: Vec<PathBuf> = WalkDir::new(source)
        .into_iter()
        .filter_entry(|e| is_not_hidden(e))
        .filter_map(|v| v.ok())
        .filter_map(|v| {
            if query.matches(v.path()) {
                Some(v)
            } else {
                None
            }
        })
        .map(|v| v.path().into())
        .collect();

    let max_id_padding = format!("{}", selfiles.len()).len();

    for (i, trgpath) in selfiles.into_iter().enumerate() {
        let mut lnkpath = target.join(&format!("{:01$}", i, max_id_padding));
        if let Some(ext) = trgpath.extension() {
            lnkpath.set_extension(ext);
        }
        let trgpath = normalize_path(&trgpath, target).expect("failed to normalize path");
        println!("{} -> {}", lnkpath.display(), trgpath.display());
        if let Err(x) = symlink::symlink_auto(&trgpath, &lnkpath) {
            eprintln!(
                "{} -> {}: failed to create symlink: {:?}",
                lnkpath.display(),
                trgpath.display(),
                x
            );
        }
    }
}

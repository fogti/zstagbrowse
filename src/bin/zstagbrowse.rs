use ::zstags::*;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};
use walkdir::{DirEntry, WalkDir};

enum FoldOp {
    And,
    Or,
    Xor,
}

struct Query {
    foldop: FoldOp,
    tags: HashSet<String>,
}

impl Query {
    pub fn matches(&self, path: &Path, backend: impl AsRef<dyn Backend>) -> bool {
        match backend.as_ref().tags(path) {
            Ok(tags_on_file) => {
                let icnt = self.tags.intersection(&tags_on_file).count();
                match self.foldop {
                    FoldOp::And => icnt == self.tags.len(),
                    FoldOp::Or => icnt != 0,
                    FoldOp::Xor => icnt == 1,
                }
            }
            Err(x) => {
                eprintln!("{}: unable to access tags: {:?}", path.display(), x);
                false
            }
        }
    }
}

fn is_not_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| entry.depth() == 0 || !s.starts_with('.'))
        .unwrap_or(false)
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
                .takes_value(true)
                .help("specifies the query folding operation (defaults to '&')"),
        )
        .arg(
            Arg::with_name("backend")
                .long("backend")
                .short("b")
                .takes_value(true)
                .required(true)
                .help("specifies the backend (where the association {FILE -> TAGS*} is stored)"),
        )
        .arg(
            Arg::with_name("QUERY")
                .required(true)
                .multiple(true)
                .index(1),
        )
        .get_matches();

    let curdir = std::env::current_dir().unwrap();
    let source: &Path = Path::new(matches.value_of("source").unwrap());
    let target = get_absolute_path(Path::new(matches.value_of("target").unwrap()), &curdir);

    let foldop = match matches.value_of("foldop").unwrap_or("&") {
        "&" | "&&" => FoldOp::And,
        "|" | "||" => FoldOp::Or,
        "^" | "^^" => FoldOp::Xor,
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

    let backend =
        create_backend(matches.value_of("backend").unwrap()).expect("unable to initialize backend");

    match fs::remove_dir_all(&target) {
        Ok(_) => {}
        Err(x) if x.kind() == std::io::ErrorKind::NotFound => {}
        Err(x) => panic!(
            "{}: failed to clear target directory: {:?}",
            target.display(),
            x
        ),
    }
    fs::create_dir_all(&target).expect("failed to create target directory");

    let selfiles: Vec<PathBuf> = WalkDir::new(source)
        .into_iter()
        .filter_entry(|e| is_not_hidden(e))
        .filter_map(|v| v.ok())
        .map(|v| get_absolute_path(v.path(), &curdir))
        .filter_map(|v| {
            use boolinator::Boolinator;
            query.matches(&v, &backend).as_some(v)
        })
        .collect();

    let max_id_padding = format!("{}", selfiles.len()).len();

    for (i, trgpath) in selfiles.into_iter().enumerate() {
        let mut lnkpath = target.join(&format!("{:01$}", i, max_id_padding));
        if let Some(ext) = trgpath.extension() {
            lnkpath.set_extension(ext);
        }
        let trgpath = normalize_path(&trgpath, &target);
        print!(
            "{} -> {}",
            lnkpath.file_name().unwrap().to_str().unwrap(),
            trgpath.display()
        );
        if let Err(x) = symlink::symlink_auto(&trgpath, &lnkpath) {
            print!(": failed to create symlink: {:?}", x);
        }
        println!();
    }
}

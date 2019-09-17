use ::zstags::*;

fn main() {
    use clap::{crate_version, Arg};

    let matches = clap::App::new("zstagdump")
        .version(crate_version!())
        .author("Erik Zscheile <erik.zscheile@gmail.com>")
        .about("tag dumper")
        .setting(clap::AppSettings::TrailingVarArg)
        .arg(
            Arg::with_name("source")
                .long("source")
                .takes_value(true)
                .required(true)
                .help("specifies the source tree"),
        )
        .arg(
            Arg::with_name("backend")
                .long("backend")
                .short("b")
                .takes_value(true)
                .required(true)
                .help("specifies the backend (where the association {FILE -> TAGS*} is stored)"),
        )
        .get_matches();

    let source: &Path = Path::new(matches.value_of("source").unwrap());

    if !source.is_dir() {
        panic!("{}: is not a directory", source.display());
    }

    let backend =
        create_backend(matches.value_of("backend").unwrap()).expect("unable to initialize backend");

    for i in iter_dir_nonhid_abs(source, &std::env::current_dir().unwrap()) {
        match backend.tags(&i) {
            Ok(ref tags) if !tags.is_empty() => print_tags(&format!("{}", i.display()), tags),
            Ok(_) => {}
            Err(x) => eprintln!("{}: ERROR: {}", i.display(), x),
        }
    }
}

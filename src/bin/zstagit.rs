use ::zstags::*;
use std::{collections::HashSet, path::Path};

fn print_tags(ltitle: &str, tags: &HashSet<String>) {
    print!("{}:", ltitle);
    for i in tags {
        print!(" {}", i);
    }
    println!();
}

fn main() {
    use clap::{crate_version, Arg};

    let matches = clap::App::new("zstagit")
        .version(crate_version!())
        .author("Erik Zscheile <erik.zscheile@gmail.com>")
        .about("tag editor")
        .arg(
            Arg::with_name("file")
                .long("file")
                .short("f")
                .takes_value(true)
                .required(true)
                .help("specify the file whose tags should be edited"),
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
            Arg::with_name("verbose")
                .long("verbose")
                .help("increase verbosity"),
        )
        .arg(
            Arg::with_name("TAGMODS")
                .required(true)
                .multiple(true)
                .index(1),
        )
        .get_matches();

    let filepath = Path::new(matches.value_of("file").unwrap());
    let is_verbose = matches.is_present("verbose");

    let mut backend =
        create_backend(matches.value_of("backend").unwrap()).expect("unable to initialize backend");

    let mut tags = backend.tags(&filepath).expect("unable to read tags");

    if is_verbose {
        print_tags("old tags", &tags);
    }

    for i in matches.values_of("TAGMODS").unwrap() {
        if i.len() < 2 {
            eprintln!("got invalid tag modifier: '{}'", i);
        }
        let mut ici = i.chars();
        let msc = ici.next().unwrap();
        let rest = ici.collect();
        match msc {
            '+' => {
                tags.insert(rest);
            }
            '-' => {
                tags.remove(&rest);
            }
            _ => eprintln!("got invalid tag modifier: '{}'", i),
        }
    }

    if is_verbose {
        print_tags("new tags", &tags);
    }

    backend
        .set_tags(&filepath, tags)
        .expect("unable to write tags");
}

use cargo_metadata::MetadataCommand;
use clap::{App, Arg, SubCommand};
use std::path::PathBuf;
use cargo_include_licenses::{copy_licenses_to, search_for_all_licenses};

fn main() {
    let matches = App::new("Cargo include-licenses")
        .version("0.1.0")
        .about("Finds licenses for the dependencies of your program and copies the files to a directory")
        .subcommand(SubCommand::with_name("include-licenses")
            .arg(Arg::with_name("out_dir")
                .help("The directory where to put the licenses into")
                .required(true)
                .value_name("DIR")
                .validator(|out_dir| (PathBuf::from(&out_dir).is_dir() || !PathBuf::from(out_dir).exists()).then(|| ()).ok_or_else(|| String::from("Not a directory")))
            )
        )
        .get_matches();
    let out_dir = PathBuf::from(matches.subcommand().1.unwrap().value_of("out_dir").unwrap());
    let command = MetadataCommand::new();
    let metadata = command.exec().unwrap();

    copy_licenses_to(&out_dir, search_for_all_licenses(metadata)).unwrap();
}

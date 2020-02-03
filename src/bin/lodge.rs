use clap::{App, Arg};
use dirs::home_dir;

fn main() {
    let home = home_dir().expect("could not determine home directory");
    let home_path = home.to_str().expect("could not get path of home directory");

    let matches = App::new(env!("CARGO_PKG_NAME"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("SOURCES")
                .help("List of source directories to link into <target>")
                .multiple(true)
                .default_value("."),
        )
        .arg(
            Arg::with_name("target")
                .long("target")
                .short("t")
                .help("Target directory for recreating structure and linking contents of SOURCES")
                .default_value(home_path),
        )
        .get_matches();

    println!("{:#?}", matches);
}

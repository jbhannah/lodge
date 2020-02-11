use clap::{App, AppSettings, Arg, SubCommand};
use dirs::home_dir;
use lodge::{cmd, cmd::Command, ARG_SOURCES, ARG_TARGET, CMD_LINK};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let home = home_dir().expect("could not determine home directory");
    let home_path = home.to_str().expect("could not get path of home directory");

    let matches = App::new(env!("CARGO_PKG_NAME"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name(CMD_LINK)
                .about("Link one or more source directories into a target directory")
                .arg(
                    Arg::with_name(ARG_TARGET)
                        .long("target")
                        .short("t")
                        .help("Target directory for creating folders and symbolic links")
                        .default_value(home_path),
                )
                .arg(
                    Arg::with_name(ARG_SOURCES)
                        .help("List of source directories to link into <TARGET>")
                        .multiple(true)
                        .default_value("."),
                ),
        )
        .get_matches();

    let (subcommand, sub_matches) = matches.subcommand();

    let command = match subcommand {
        CMD_LINK => cmd::link::LinkCommand::init(sub_matches),
        _ => panic!("no subcommand"),
    }
    .expect("error initializing command");

    command.exec()
}

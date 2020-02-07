use clap::{App, AppSettings, Arg, SubCommand};
use crossbeam_channel::bounded;
use dirs::home_dir;
use ignore::{overrides::OverrideBuilder, WalkBuilder, WalkState};
use lodge::{link, source::Source, target::Target};
use std::convert::TryFrom;
use std::path::PathBuf;
use std::thread;

const ARG_TARGET: &str = "TARGET";
const ARG_SOURCES: &str = "SOURCES";

const CMD_LINK: &str = "link";

const OVERRIDES: [&str; 2] = ["!.git", "!.hg"];

fn main() -> Result<(), ignore::Error> {
    let home = home_dir().expect("could not determine home directory");
    let home_path = home.to_str().expect("could not get path of home directory");

    let app = App::new(env!("CARGO_PKG_NAME"))
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
        );
    let matches = app.clone().get_matches();

    if let (CMD_LINK, Some(matches)) = matches.subcommand() {
        let target = Target::new(
            matches
                .value_of(ARG_TARGET)
                .expect("no valid target specified"),
        )
        .expect("cannot use target path");

        let sources: Vec<PathBuf> = matches
            .values_of(ARG_SOURCES)
            .expect("no valid sources specificed")
            .map(PathBuf::from)
            .collect();

        let mut overrides: Vec<OverrideBuilder> = Vec::new();

        for source in sources.iter() {
            let mut over = OverrideBuilder::new(source);

            for rule in OVERRIDES.iter() {
                over.add(rule)?;
            }

            overrides.push(over);
        }

        if let Some((first, rest)) = sources.split_first() {
            let mut builder = WalkBuilder::new(first);
            builder.hidden(false);

            for over in overrides.iter() {
                builder.overrides(over.build()?);
            }

            for path in rest {
                builder.add(path);
            }

            let (tx, rx) = bounded::<Source>(10);

            let rx_thread = thread::spawn(move || {
                let mut count = 0;
                let mut skip = 0;

                for src in rx {
                    match target.build_link(&src) {
                        Ok(link) => {
                            match link.mklink() {
                                Ok(_) => {
                                    count += 1;
                                }
                                Err(err) => {
                                    eprintln!("could not create link: {}", err);
                                }
                            };
                        }
                        Err(link::Error::Skip(err)) => {
                            eprintln!("skipped building link: {}", err);
                            skip += 1;
                        }
                        Err(link::Error::Io(err)) => {
                            eprintln!("could not build link: {}", err);
                        }
                    }
                }

                println!("Linked: {}", count);
                println!("Skipped: {}", skip);
                println!("Total: {}", count + skip);
            });

            builder.build_parallel().run(|| {
                let tx = tx.clone();
                Box::new(move |entry| match entry {
                    Ok(entry) => match Source::try_from(entry) {
                        Ok(src) => {
                            tx.send(src).expect("could not process source file");
                            WalkState::Continue
                        }
                        Err(_) => WalkState::Continue,
                    },
                    Err(_) => WalkState::Continue,
                })
            });

            drop(tx);
            rx_thread.join().expect("could not join linking thread");
        }
    };

    Ok(())
}

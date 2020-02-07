use clap::{App, Arg};
use crossbeam_channel::bounded;
use dirs::home_dir;
use ignore::{overrides::OverrideBuilder, WalkBuilder, WalkState};
use lodge::{base::Base, link, source::Source};
use std::convert::TryFrom;
use std::path::PathBuf;
use std::thread;

const ARG_BASE: &str = "BASE";
const ARG_SOURCES: &str = "SOURCES";

const OVERRIDES: [&str; 2] = ["!.git", "!.hg"];

fn main() -> Result<(), ignore::Error> {
    let home = home_dir().expect("could not determine home directory");
    let home_path = home.to_str().expect("could not get path of home directory");

    let matches = App::new(env!("CARGO_PKG_NAME"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name(ARG_SOURCES)
                .help("List of source directories to link into <BASE>")
                .multiple(true)
                .default_value("."),
        )
        .arg(
            Arg::with_name(ARG_BASE)
                .long("base")
                .short("b")
                .help("Base directory for recreating structure and linking contents of SOURCES")
                .default_value(home_path),
        )
        .get_matches();

    let sources: Vec<PathBuf> = match matches.values_of(ARG_SOURCES) {
        Some(sources) => sources.collect::<Vec<&str>>(),
        None => vec!["."],
    }
    .iter()
    .map(PathBuf::from)
    .collect();

    let base = Base::new(match matches.value_of(ARG_BASE) {
        Some(base) => base,
        None => home_path,
    })
    .expect("cannot use base path");

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
                match base.build_link(&src) {
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

    Ok(())
}

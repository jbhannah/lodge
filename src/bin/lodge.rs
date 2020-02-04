use clap::{App, Arg};
use dirs::home_dir;
use ignore::{overrides::OverrideBuilder, WalkBuilder, WalkState};
use std::path::PathBuf;

const OVERRIDES: [&str; 2] = ["!.git", "!.hg"];

fn main() -> Result<(), ignore::Error> {
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

    let sources: Vec<PathBuf> = matches
        .values_of("SOURCES")
        .unwrap()
        .map(PathBuf::from)
        .collect();
    let _target = PathBuf::from(matches.value_of("target").unwrap());

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

        builder.build_parallel().run(|| {
            Box::new(|p| {
                if let Ok(path) = p {
                    let components = path
                        .path()
                        .components()
                        .rev()
                        .take(path.depth())
                        .collect::<Vec<_>>();

                    if !components.is_empty() {
                        println!("{:?}", components.iter().rev().collect::<Vec<_>>());
                    }
                }

                WalkState::Continue
            })
        })
    }
    Ok(())
}

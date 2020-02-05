use clap::{App, Arg};
use crossbeam_channel::bounded;
use dirs::home_dir;
use ignore::{overrides::OverrideBuilder, DirEntry, WalkBuilder, WalkState};
use lodge::base::Base;
use std::fs::{remove_file, DirBuilder};
use std::os::unix::fs::symlink;
use std::path::PathBuf;
use std::thread;

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
                .help("List of source directories to link into <base>")
                .multiple(true)
                .default_value("."),
        )
        .arg(
            Arg::with_name("base")
                .long("base")
                .short("b")
                .help("Base directory for recreating structure and linking contents of SOURCES")
                .default_value(home_path),
        )
        .get_matches();

    let sources: Vec<PathBuf> = matches
        .values_of("SOURCES")
        .unwrap()
        .map(PathBuf::from)
        .collect();
    let base = Base::new(matches.value_of("base").unwrap()).expect("cannot use base path");

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

        let (tx, rx) = bounded::<DirEntry>(10);

        let rx_thread = thread::spawn(move || {
            let mut count = 0;

            for src in rx {
                count += 1;

                let mut dst = base.clone();
                let components = src
                    .path()
                    .components()
                    .rev()
                    .take(src.depth())
                    .collect::<Vec<_>>();

                if components.is_empty() {
                    continue;
                }

                for component in components.iter().rev() {
                    dst.push(component);
                }
                println!("{}", dst.display());

                if let Some(parent) = dst.parent() {
                    DirBuilder::new()
                        .recursive(true)
                        .create(parent)
                        .expect("could not create parent directory");
                }

                let src_meta = src
                    .metadata()
                    .expect("could not read metadata for source file");

                if let Ok(dst_meta) = dst.symlink_metadata() {
                    if dst.exists() {
                        if dst_meta.file_type().is_file()
                            && dst_meta.file_type() == src_meta.file_type()
                            && dst_meta.len() == src_meta.len()
                        {
                            remove_file(dst.as_path())
                                .expect("could not remove identical file at destination path");
                        } else {
                            println!("Skipping {}", dst.as_path().display());
                            continue;
                        }
                    }

                    if dst_meta.file_type().is_symlink() {
                        remove_file(dst.as_path())
                            .expect("could not remove symlink at destination path");
                    }
                }

                symlink(
                    src.path()
                        .canonicalize()
                        .expect("could not determine canonical source path"),
                    dst,
                )
                .expect("could not create link");
            }

            println!("Final count: {}", count);
        });

        builder.build_parallel().run(|| {
            let tx = tx.clone();
            Box::new(move |entry| {
                if let Ok(entry) = entry {
                    if entry.path().is_file() {
                        tx.send(entry).unwrap();
                    }
                }

                WalkState::Continue
            })
        });

        drop(tx);
        rx_thread.join().unwrap();
    }

    Ok(())
}

use crate::cmd::Command;
use crate::link;
use crate::source::Source;
use crate::target::Target;
use crate::{ARG_SOURCES, ARG_TARGET, OVERRIDES};
use clap::ArgMatches;
use crossbeam_channel::{bounded, Receiver};
use ignore::overrides::OverrideBuilder;
use ignore::{WalkBuilder, WalkState};
use std::convert::TryFrom;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

pub struct LinkCommand {
    builder: WalkBuilder,
    target: Target,
}

impl Command for LinkCommand {
    fn init(matches: Option<&ArgMatches<'_>>) -> Result<Arc<Self>, Box<dyn Error>> {
        let matches = matches.expect("no arguments");

        let target = Target::new(matches.value_of(ARG_TARGET).expect("no valid target"))
            .expect("cannot use target path");

        let sources: Vec<PathBuf> = matches
            .values_of(ARG_SOURCES)
            .expect("no valid sources")
            .map(PathBuf::from)
            .collect();

        let (first, rest) = sources.split_first().expect("empty sources list");
        let mut builder = WalkBuilder::new(first);
        builder.hidden(false);

        for path in rest {
            builder.add(path);
        }

        for source in sources.iter() {
            let mut over = OverrideBuilder::new(source);

            for rule in OVERRIDES.iter() {
                over.add(rule)?;
            }

            builder.overrides(over.build()?);
        }

        Ok(Arc::from(Self { builder, target }))
    }

    fn exec(self: Arc<Self>) -> Result<(), Box<dyn Error>> {
        let (tx, rx) = bounded::<Source>(10);

        let walker = self.builder.build_parallel();
        let rx_thread = thread::spawn(move || self.link_rx(rx));

        walker.run(|| {
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
        let (count, skip) = rx_thread.join().expect("could not join thread");

        println!("Linked: {}", count);
        println!("Skipped: {}", skip);
        println!("Total: {}", count + skip);

        Ok(())
    }
}

impl LinkCommand {
    fn link_rx(&self, rx: Receiver<Source>) -> (usize, usize) {
        let mut count = 0;
        let mut skip = 0;

        for src in rx {
            match self.target.build_link(&src) {
                Ok(link) => {
                    if link.mklink().is_ok() {
                        count += 1;
                    }
                }
                Err(link::Error::Skip(_)) => {
                    skip += 1;
                }
                _ => {}
            }
        }

        (count, skip)
    }
}

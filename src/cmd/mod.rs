pub mod link;

use clap::ArgMatches;
use std::error::Error;
use std::sync::Arc;

pub trait Command {
    fn init(matches: Option<&ArgMatches<'_>>) -> Result<Arc<Self>, Box<dyn Error>>;
    fn exec(self: Arc<Self>) -> Result<(), Box<dyn Error>>;
}

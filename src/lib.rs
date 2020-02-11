pub mod cmd;
pub mod link;
pub mod source;
pub mod target;

pub const ARG_TARGET: &str = "TARGET";
pub const ARG_SOURCES: &str = "SOURCES";

pub const CMD_LINK: &str = "link";

pub const OVERRIDES: [&str; 2] = ["!.git", "!.hg"];

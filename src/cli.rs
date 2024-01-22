use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[clap(version = "1.0", author = "Owen G. Dechow")]
pub struct Arguments {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    #[clap(
        about = "Call a proxied command, it is suggested to call using 'c --' instead",
        alias = "c"
    )]
    Call { cmd: String, args: Vec<String> },
    #[clap(about = "Add a new proxy")]
    Add { path: PathBuf },
    #[clap(about = "Remove proxy")]
    Remove { cmd: String },
    #[clap(about = "List all proxies")]
    List,
}

use clap::CommandFactory;
use crate::cli::{Cli};
use crate::cli::command::Transform;

pub fn transform(_cli: &Cli, transform: &Transform) -> anyhow:: Result<()>{
    if !(transform.square || transform.landscape) {
        eprintln!("error: At least one of `--square` or `--landscape` must be provided.\n");
        Cli::command().print_help().expect("Failed to print help");
        std::process::exit(1);
    }

    Ok(())
}
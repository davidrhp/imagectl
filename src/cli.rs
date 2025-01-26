pub mod command;

use crate::cli::command::Command;
use clap::Parser;

/// Generates images optimized for Google Ads.
#[derive(Parser)]
pub struct Cli {
    
    #[command(subcommand)]
    pub command: Command,
}


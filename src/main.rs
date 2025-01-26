use clap::{Parser};
use imagectl::cli::{Cli};

fn main() -> anyhow::Result<()>{
    let cli = Cli::parse();
    
    cli.command.execute(&cli)
}

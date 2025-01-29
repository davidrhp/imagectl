
mod transform;

use crate::cli::Cli;
use clap::{Subcommand};
pub use transform::transform;
use strum_macros::Display;
use crate::cli::command::transform::Transform;

#[derive(Subcommand, Display)]
pub enum Command {

    /// Transforms images with defaults optimized for the web.
    Transform(Transform),

}


impl Command {
    pub fn execute(&self, global_args: &Cli) -> anyhow::Result<()> {
        match self {
            Command::Transform(args) => {
                self.eprint_processing_msg(args.images.len());
                transform(global_args, args)
            },

        }
    }

    fn eprint_processing_msg(&self, count_images: usize) {
        eprintln!("Executing `{}` for {} image(s).", self, count_images);
    }
}
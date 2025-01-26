
mod preview;
mod transform;

use std::path::PathBuf;
use clap::{Args, Subcommand};
use strum_macros::{Display};
pub use preview::preview;
pub use transform::transform;
use crate::cli::Cli;

#[derive(Subcommand, Display)]
pub enum Command {

    /// Generates previews of the images, optimized for the web.
    /// The resulting images may have a slight change to their aspect ratio
    /// due to the algorithm trying to get the images as close to the minimum img width.
    Preview (Preview),

    /// Transforms the images to jpegs with the requested aspect ratios.
    Transform(Transform),

}

#[derive(Args, Debug)]
pub struct Preview {

    /// The images that will be copied and converted.
    #[arg(required = true, num_args = 1)]
    pub images: Vec<PathBuf>,

    /// The suffix that the generated preview image name will have.
    #[arg(short, long, default_value = "preview")]
    pub suffix: String,

    /// The minimum width that the generated prefixes should have.
    #[arg(short, long, default_value = "600")]
    pub min_width: u32,
}

#[derive(Args, Debug)]
pub struct Transform {

    /// The images that will be copied and converted.
    #[arg(required = true, num_args = 1)]
    pub images: Vec<PathBuf>,

    /// Creates square images for each input image (1:1 - 1200x1200).
    #[arg(short, long)]
    pub square: bool,

    /// Creates landscape images for each input image (1.91:1 - 1200x628).
    #[arg(short, long)]
    pub landscape: bool,
}

impl Command {
    pub fn execute(&self, global_args: &Cli) -> anyhow::Result<()> {
        match self {
            Command::Preview(args) => {
                self.eprint_processing_msg(args.images.len());
                preview(global_args, args)
            },
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
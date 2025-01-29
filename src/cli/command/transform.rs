use crate::cli::Cli;
use crate::image::{crop_center_landscape, crop_center_square, crop_left_square, crop_right_square, resize};
use anyhow::{Context, Error};
use clap::ArgGroup;
use clap::{Args, ValueEnum};
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageFormat, ImageReader};
use indicatif::ProgressBar;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

#[derive(Args, Debug)]
pub struct Transform {

    /// The images that will be copied and converted.
    #[arg(required = true, num_args = 1)]
    pub images: Vec<PathBuf>,

    /// The suffix that the generated image name will have.
    #[arg(short, long, default_value = "generated")]
    pub suffix: String,

    /// The minimum width that the generated images should have.
    #[arg(short, long, default_value = "600")]
    pub min_width: u32,

    /// Scale the generated images according to the `min_width` argument. If the image is larger
    /// than `min_width`, the image will be downscaled, otherwise it will be upscaled.
    /// To preserve the aspect ratio, the actual width of the generated image may
    /// not be exactly equal to `min_width`.
    #[arg(long, default_value="false")]
    pub scale: bool,

    // /// If false, images may have a slight change to their aspect ratio
    // /// due to the algorithm trying to get the images as close to the `min_width` of the image.
    // /// Otherwise, the scaling algorithm may not exactly hit the desired `min_width`.
    // #[arg(long, default_value="true")]
    // pub strict_aspect_ratio: bool,

    /// The file format of the generated image.
    #[arg(long, default_value = "jpg")]
    pub format: FileFormat,

    #[arg(long = "aspect-ratio")]
    pub aspect_ratios: Vec<AspectRatios>
}

#[derive(Debug, Clone, ValueEnum)]
pub enum FileFormat {
    Avif,
    Webp,
    Jpeg,
    Jpg,
    Png
}

#[derive(Debug, Clone, ValueEnum)]
pub enum AspectRatios {
    /// 1:1 - crop with center bias
    Square,
    /// 1:1 - crop with right bias
    SquareRight,
    /// 1:1 - crop with left bias
    SquareLeft,
    /// 1.91:1 - crop with center bias
    Landscape
}

impl AspectRatios {
    pub fn crop(&self, img: &DynamicImage) -> DynamicImage {
        match self {
            AspectRatios::Square => crop_center_square(img),
            AspectRatios::SquareRight => crop_right_square(img),
            AspectRatios::SquareLeft => crop_left_square(img),
            AspectRatios::Landscape => crop_center_landscape(img),
        }
    }

    pub fn suffix(&self) -> &'static str {
        match self {
            AspectRatios::Square => "square_center",
            AspectRatios::SquareRight => "square_right",
            AspectRatios::SquareLeft => "square_left",
            AspectRatios::Landscape => "landscape_center"
        }
    }
}

impl From<FileFormat> for ImageFormat {
    
    fn from(value: FileFormat) -> Self {
        match value {
            FileFormat::Avif => ImageFormat::Avif,
            FileFormat::Webp => ImageFormat::WebP,
            FileFormat::Jpg | FileFormat::Jpeg => ImageFormat::Jpeg,
            FileFormat::Png => ImageFormat::Png
        }
    }
}



pub fn transform(_cli: &Cli, args: &Transform) -> anyhow::Result<()> {
    if !args.aspect_ratios.is_empty() {
        eprintln!("Generating images for the following aspect ratios {:?}", args.aspect_ratios);
    }

    let bar = ProgressBar::new(args.images.len() as u64);


    args
        .images
        .par_iter()
        .map(|path| {
            let img = open_image(path)?;

            let image_variants = generate_image_variants(args, img);

            let mut generated_names = vec![];
            for (mut img_variant, suffix) in image_variants {
                // resize image
                if args.scale {
                    let (width, height) = img_variant.dimensions();
                    let (updated_width, updated_height) = resize(width, height, args.min_width);
                    if updated_width != width {
                        img_variant = img_variant.resize(updated_width, updated_height, FilterType::Lanczos3);
                    }
                }

                let generated_name = write_image(img_variant, args.format.clone().into(), path, suffix)?;

                generated_names.push(generated_name)
            }

            Ok::<Vec<String>, Error>(generated_names)
        })
        .for_each(|result| {
            match result {
                Ok(file_names) => bar.println(format!("[+] generated [{}]", file_names.join(", "))),
                Err(err) => bar.println(format!("[+] failed to generate {:?}", err)),
            }
            bar.inc(1);
        });

    bar.finish_with_message("Processed all images");

    Ok(())
}

fn write_image(img: DynamicImage, format: ImageFormat, path: &PathBuf, suffix: String) -> Result<String, Error> {
    // write image

    let extension = format.extensions_str()[0];

    let mut out_path = new_path(path, &suffix);
    out_path.set_extension(extension);

    let out_file = File::create(&out_path).with_context(|| {
        format!("could not create output file: {:?}", out_path.as_path())
    })?;
    let mut writer = BufWriter::new(out_file);

    img.write_to(&mut writer, format)?;

    let generated_name = out_path
        .file_name()
        .expect("file name to be present after a file with that name has been written")
        .to_string_lossy()
        .to_string();
    Ok(generated_name)
}

fn generate_image_variants(args: &Transform, img: DynamicImage) -> Vec<(DynamicImage, String)> {
    let image_variants = if args.aspect_ratios.is_empty() {
        vec![(img, args.suffix.clone())]
    } else {
        args.aspect_ratios
            .iter()
            .map(|ar| {
                (ar.crop(&img), format!("{}_{}", args.suffix, ar.suffix().to_string()))
            })
            .collect()
    };
    image_variants
}

fn open_image(path: &PathBuf) -> Result<DynamicImage, Error> {
    let img = ImageReader::open(path)
        .with_context(|| format!("failed to open image at: {:?}", path.as_path()))?
        .with_guessed_format()
        .with_context(|| format!("could not guess format of: {:?}", path.as_path()))?
        .decode()
        .with_context(|| format!("failed to decode image at {:?}", path.as_path()))?;

    Ok(img)
}

fn new_path(old_path: &Path, file_name_suffix: &str) -> PathBuf {
    let mut preview_path = PathBuf::from(old_path);
    let file_stem = old_path
        .file_stem()
        .expect("file name to be present, since this image has been opened already");

    let new_file_name = format!("{}_{}", file_stem.to_string_lossy(), file_name_suffix);

    preview_path.set_file_name(new_file_name);

    preview_path
}

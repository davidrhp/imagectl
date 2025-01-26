use crate::cli::command::Preview;
use crate::cli::Cli;
use anyhow::{Context, Error};
use image::imageops::FilterType;
use image::{GenericImageView, ImageFormat, ImageReader};
use indicatif::ProgressBar;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

pub fn preview(_cli: &Cli, preview: &Preview) -> anyhow::Result<()> {
    let bar = ProgressBar::new(preview.images.len() as u64);

    preview
        .images
        .par_iter()
        .map(|path| {
            let mut img = ImageReader::open(path)
                .with_context(|| format!("failed to open image at: {:?}", path.as_path()))?
                .with_guessed_format()
                .with_context(|| format!("could not guess format of: {:?}", path.as_path()))?
                .decode()
                .with_context(|| format!("failed to decode image at {:?}", path.as_path()))?;

            // resize image
            let (width, height) = img.dimensions();
            let (updated_width, updated_height) = resize(width, height, preview.min_width);
            if updated_width < width {
                img = img.resize(updated_width, updated_height, FilterType::Lanczos3);
            }

            // write image
            let mut out_path = new_path(path, &preview.suffix);
            out_path.set_extension("avif");

            let out_file = File::create(&out_path).with_context(|| {
                format!("could not create output file: {:?}", out_path.as_path())
            })?;
            let mut writer = BufWriter::new(out_file);

            img.write_to(&mut writer, ImageFormat::Avif)?;

            let preview_name = out_path
                .file_name()
                .expect("file name to be present after a file with that name has been written");

            Ok::<String, Error>(preview_name.to_string_lossy().to_string())
        })
        .for_each(|result| {
            match result {
                Ok(file_name) => bar.println(format!("[+] generated {}", file_name)),
                Err(err) => bar.println(format!("[+] failed to generate {}", err)),
            }
            bar.inc(1);
        });

    bar.finish_with_message("Processed all images");

    Ok(())
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

/// Returns a new width, height combination, where the width is as close to MIN_WIDTH as possible.
/// This may result in a slight aspect ratio change.
fn resize(width: u32, height: u32, min_width: u32) -> (u32, u32) {
    
    let scaling_factor = min_width as f32 / width as f32;
    
    let new_width = width as f32 * scaling_factor;
    let new_height = (height as f32 * (new_width / width as f32)).round() as u32;

    (new_width.round() as u32, new_height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestCase {
        input_width: u32,
        input_height: u32,
        expected_width: u32,
        expected_height: u32,
    }
    
    #[test]
    fn test_resize_method() {
        let cases = vec![
            TestCase {
                input_width: 1200,
                input_height: 1200,
                expected_width: 600,
                expected_height: 600,
            },
            TestCase {
                input_width: 1200,
                input_height: 628,
                expected_width: 600,
                expected_height: 314,
            },
            TestCase {
                input_width: 2400,
                input_height: 2400,
                expected_width: 600,
                expected_height: 600,
            },
            TestCase {
                input_width: 950,
                input_height : 500,
                expected_width: 600,
                expected_height : 316,
            }
        ];

        let min_width = 600;

        for case in cases {
            let (actual_width, actual_height) = resize(case.input_width, case.input_height, min_width);

            assert_eq!(actual_width, case.expected_width, "Failed test case {:?}", case);
            assert_eq!(actual_height, case.expected_height, "Failed test case {:?}", case);


            let expected_ratio: f32 = case.input_width as f32 / case.input_height as f32;
            let actual_ratio: f32 = actual_width as f32 / actual_height as f32;
            let ratio_diff = expected_ratio - actual_ratio;

            let epsilon = 0.1;

            // we can tolerate a slight change of the aspect ratio
            assert!(ratio_diff.abs() < epsilon,
                    "Failed test case {:?}, aspect_ratio {} larger than epsilon {}",
                    case,
                    ratio_diff.abs(),
                    epsilon);
        }
    }
}

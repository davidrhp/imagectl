use image::{DynamicImage, GenericImageView};

/// Returns a new width, height combination, where the width is as close to MIN_WIDTH as possible.
/// This may result in a slight aspect ratio change.
pub fn resize(width: u32, height: u32, min_width: u32) -> (u32, u32) {

    let scaling_factor = min_width as f32 / width as f32;

    let new_width = width as f32 * scaling_factor;
    let new_height = (height as f32 * (new_width / width as f32)).round() as u32;

    (new_width.round() as u32, new_height)
}

pub fn crop_center_square(img: &DynamicImage) -> DynamicImage {
    let (width, height) = img.dimensions();

    // Determine the side length of the square (smallest dimension)
    let square_side = width.min(height);

    // Calculate the top-left corner to center the square
    let x = (width - square_side) / 2;
    let y = (height - square_side) / 2;

    // Crop using crop_imm (this returns the cropped image as a new image)
    img.crop_imm(x, y, square_side, square_side)
}

pub fn crop_right_square(img: &DynamicImage) -> DynamicImage {
    let (width, height) = img.dimensions();

    // Determine the side of the square (smallest dimension)
    let square_side = width.min(height);

    // Place the square at the far right; `x` is adjusted to align the square's right edge
    let x = width - square_side;
    let y = (height - square_side) / 2; // Center vertically

    // Crop using crop_imm
    img.crop_imm(x, y, square_side, square_side)
}

pub fn crop_left_square(img: &DynamicImage) -> DynamicImage {
    let (width, height) = img.dimensions();

    // Determine the side of the square (smallest dimension)
    let square_side = width.min(height);

    // Place the square at the far left; `x` is 0 for left alignment
    let x = 0;
    let y = (height - square_side) / 2; // Center vertically

    // Crop using crop_imm
    img.crop_imm(x, y, square_side, square_side)
}

pub fn crop_center_landscape(img: &DynamicImage) -> DynamicImage {
    let (orig_width, orig_height) = img.dimensions();

    // Target aspect ratio (1.91:1)
    let target_ratio = 1.91;

    let (crop_width, crop_height) = if orig_width as f32 / orig_height as f32 > target_ratio {
        // Image is wider than 1.91:1, limit width to fit height
        let crop_height = orig_height;
        let crop_width = (crop_height as f32 * target_ratio) as u32;
        (crop_width, crop_height)
    } else {
        // Image is taller than 1.91:1, limit height to fit width
        let crop_width = orig_width;
        let crop_height = (crop_width as f32 / target_ratio) as u32;
        (crop_width, crop_height)
    };

    // Center the crop
    let x = (orig_width - crop_width) / 2;
    let y = (orig_height - crop_height) / 2;

    // Perform cropping
    img.crop_imm(x, y, crop_width, crop_height)
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
            // Scale down
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
            },
            TestCase {
                input_width: 950,
                input_height : 500,
                expected_width: 600,
                expected_height : 316,
            },
            // Scale up
            TestCase {
                input_width: 300,
                input_height: 300,
                expected_width: 600,
                expected_height: 600,
            },
            TestCase {
                input_width: 300,
                input_height: 157,
                expected_width: 600,
                expected_height: 314,
            },
            TestCase {
                input_width: 150,
                input_height: 150,
                expected_width: 600,
                expected_height: 600,
            },
            TestCase {
                input_width: 475,
                input_height : 250,
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
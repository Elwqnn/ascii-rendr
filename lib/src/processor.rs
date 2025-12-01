use crate::ascii::{
    downscale_to_tiles, render_ascii_to_image, render_ascii_to_image_with_source,
    select_ascii_chars,
};
use crate::config::AsciiConfig;
use crate::edges::detect_edges_tiled;
use crate::filters::{calculate_luminance, difference_of_gaussians, sobel_filter};
use image::{RgbaImage, imageops};

/// Resize image to nearest dimensions that are multiples of 8
///
/// # Arguments
/// * `input` - The input RGBA image to resize
///
/// # Returns
/// A tuple of (resized_image, was_resized) where was_resized indicates if resizing occurred
fn resize_to_valid_dimensions(input: &RgbaImage) -> (RgbaImage, bool) {
    let (width, height) = input.dimensions();

    // Calculate target dimensions (round down to nearest multiple of 8)
    let target_width = (width / 8) * 8;
    let target_height = (height / 8) * 8;

    // If already valid dimensions, return original image
    if width == target_width && height == target_height {
        return (input.clone(), false);
    }

    // Resize using Lanczos3 filter for high quality
    let resized = imageops::resize(
        input,
        target_width,
        target_height,
        imageops::FilterType::Lanczos3,
    );
    (resized, true)
}

/// Processes an input image and converts it to ASCII art
///
/// This implements the full pipeline from the Acerola shader:
/// 1. Extract luminance from color image
/// 2. Apply Difference of Gaussians (DoG) for edge detection
/// 3. Apply Sobel filter to get edge directions
/// 4. Tile-based edge direction voting (8×8 tiles)
/// 5. Downscale luminance to tiles
/// 6. Select ASCII characters based on edges and luminance
/// 7. Render characters to output image
///
/// # Arguments
/// * `input` - The input RGBA image to convert
/// * `config` - Configuration parameters for the ASCII conversion
///
/// # Returns
/// An RGBA image containing the ASCII art representation
///
/// # Note
/// If the input image dimensions are not multiples of 8, it will be automatically
/// resized (rounded down) to the nearest valid dimensions using Lanczos3 filtering.
pub fn process_image(input: &RgbaImage, config: &AsciiConfig) -> RgbaImage {
    // Validate config
    config.validate().expect("Invalid configuration");

    // Automatically resize if dimensions are not multiples of 8
    let (working_image, _was_resized) = resize_to_valid_dimensions(input);
    let (width, height) = working_image.dimensions();

    // Step 1: Extract luminance
    let lum = calculate_luminance(&working_image);

    // Step 2: Difference of Gaussians (DoG) for edge detection
    let sigma1 = config.sigma;
    let sigma2 = config.sigma * config.sigma_scale;
    let dog = difference_of_gaussians(
        &lum,
        sigma1,
        sigma2,
        config.kernel_size,
        config.tau,
        config.threshold,
    );

    // Step 3: Sobel filter for edge gradients
    let (angles, valid_mask) = sobel_filter(&dog);

    // Step 4: Tile-based edge detection (8×8 tiles with voting)
    let edges = detect_edges_tiled(&angles, &valid_mask, width, height, config.edge_threshold);

    // Step 5: Downscale luminance to 8×8 tiles
    let tile_lum = downscale_to_tiles(&lum, 8);

    // Step 6: Select ASCII characters for each tile
    let tile_width = width / 8;
    let tile_height = height / 8;
    let chars = select_ascii_chars(&edges, &tile_lum, tile_width, tile_height, config);

    // Step 7: Render ASCII characters to image
    render_ascii_to_image(&chars, tile_width, tile_height, config)
}

/// Processes an input image and converts it to ASCII art while preserving original colors
///
/// This is the same as process_image but preserves colors from the source image
/// instead of using solid colors from the config.
///
/// # Arguments
/// * `input` - The input RGBA image to convert
/// * `config` - Configuration parameters for the ASCII conversion
///
/// # Returns
/// An RGBA image containing the ASCII art representation with preserved colors
///
/// # Note
/// If the input image dimensions are not multiples of 8, it will be automatically
/// resized (rounded down) to the nearest valid dimensions using Lanczos3 filtering.
pub fn process_image_preserve_colors(input: &RgbaImage, config: &AsciiConfig) -> RgbaImage {
    // Validate config
    config.validate().expect("Invalid configuration");

    // Automatically resize if dimensions are not multiples of 8
    let (working_image, _was_resized) = resize_to_valid_dimensions(input);
    let (width, height) = working_image.dimensions();

    // Step 1: Extract luminance
    let lum = calculate_luminance(&working_image);

    // Step 2: Difference of Gaussians (DoG) for edge detection
    let sigma1 = config.sigma;
    let sigma2 = config.sigma * config.sigma_scale;
    let dog = difference_of_gaussians(
        &lum,
        sigma1,
        sigma2,
        config.kernel_size,
        config.tau,
        config.threshold,
    );

    // Step 3: Sobel filter for edge gradients
    let (angles, valid_mask) = sobel_filter(&dog);

    // Step 4: Tile-based edge detection (8×8 tiles with voting)
    let edges = detect_edges_tiled(&angles, &valid_mask, width, height, config.edge_threshold);

    // Step 5: Downscale luminance to 8×8 tiles
    let tile_lum = downscale_to_tiles(&lum, 8);

    // Step 6: Select ASCII characters for each tile
    let tile_width = width / 8;
    let tile_height = height / 8;
    let chars = select_ascii_chars(&edges, &tile_lum, tile_width, tile_height, config);

    // Step 7: Render ASCII characters to image with color preservation
    render_ascii_to_image_with_source(
        &chars,
        tile_width,
        tile_height,
        config,
        Some(&working_image),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resize_to_valid_dimensions_no_resize() {
        let img = RgbaImage::new(160, 160); // Already valid (20*8 x 20*8)
        let (resized, was_resized) = resize_to_valid_dimensions(&img);
        assert_eq!(resized.dimensions(), (160, 160));
        assert!(!was_resized);
    }

    #[test]
    fn test_resize_to_valid_dimensions_resize_needed() {
        let img = RgbaImage::new(100, 100); // Not multiple of 8
        let (resized, was_resized) = resize_to_valid_dimensions(&img);
        assert_eq!(resized.dimensions(), (96, 96)); // 100 -> 96 (12*8)
        assert!(was_resized);
    }

    #[test]
    fn test_resize_to_valid_dimensions_asymmetric() {
        let img = RgbaImage::new(127, 85); // Both not multiples of 8
        let (resized, was_resized) = resize_to_valid_dimensions(&img);
        assert_eq!(resized.dimensions(), (120, 80)); // 127 -> 120, 85 -> 80
        assert!(was_resized);
    }

    #[test]
    fn test_process_invalid_dimensions_auto_resize() {
        let img = RgbaImage::new(100, 100); // Not multiple of 8, will be auto-resized
        let config = AsciiConfig::default();
        let result = process_image(&img, &config);
        assert_eq!(result.dimensions(), (96, 96)); // Resized to 96x96
    }

    #[test]
    fn test_process_valid_dimensions() {
        let img = RgbaImage::new(160, 160); // 20*8 x 20*8
        let config = AsciiConfig::default();
        let result = process_image(&img, &config);
        assert_eq!(result.dimensions(), (160, 160));
    }
}

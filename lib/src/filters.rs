use image::{GrayImage, Luma, RgbaImage};

/// Calculate luminance from an RGBA image using the standard formula
///
/// Formula: L = 0.2127*R + 0.7152*G + 0.0722*B
/// This matches the luminance calculation from AcerolaFX_Common.fxh
///
/// # Arguments
/// * `img` - Input RGBA image
///
/// # Returns
/// Grayscale image with luminance values
pub fn calculate_luminance(img: &RgbaImage) -> GrayImage {
    let (width, height) = img.dimensions();
    let mut output = GrayImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            let r = pixel[0] as f32 / 255.0;
            let g = pixel[1] as f32 / 255.0;
            let b = pixel[2] as f32 / 255.0;

            // Standard luminance coefficients
            let luminance = 0.2127 * r + 0.7152 * g + 0.0722 * b;

            // Clamp to [0, 1] and convert to u8
            let lum_u8 = (luminance.clamp(0.0, 1.0) * 255.0) as u8;
            output.put_pixel(x, y, Luma([lum_u8]));
        }
    }

    output
}

/// Calculate Gaussian weight for a given sigma and position
///
/// Formula: (1 / sqrt(2π σ²)) * exp(-(pos²) / (2σ²))
/// This matches the gaussian() function from AcerolaFX_ASCII.fx:222
///
/// # Arguments
/// * `sigma` - Standard deviation of the Gaussian
/// * `pos` - Position relative to center
///
/// # Returns
/// Gaussian weight at the given position
pub fn gaussian(sigma: f32, pos: f32) -> f32 {
    let two_pi = 2.0 * std::f32::consts::PI;
    let sigma_sq = sigma * sigma;

    (1.0 / (two_pi * sigma_sq).sqrt()) * (-pos * pos / (2.0 * sigma_sq)).exp()
}

/// Apply horizontal Gaussian blur
///
/// This implements the horizontal pass of the separable Gaussian blur
/// Corresponds to PS_HorizontalBlur from AcerolaFX_ASCII.fx:277
///
/// # Arguments
/// * `img` - Input grayscale image
/// * `sigma` - Standard deviation of the Gaussian
/// * `kernel_size` - Radius of the kernel (total width = 2*kernel_size + 1)
///
/// # Returns
/// Horizontally blurred image
pub fn gaussian_blur_h(img: &GrayImage, sigma: f32, kernel_size: u32) -> GrayImage {
    let (width, height) = img.dimensions();
    let mut output = GrayImage::new(width, height);
    let kernel_size = kernel_size as i32;

    for y in 0..height {
        for x in 0..width {
            let mut sum = 0.0;
            let mut weight_sum = 0.0;

            // Convolve with horizontal Gaussian kernel
            for offset in -kernel_size..=kernel_size {
                let sample_x = (x as i32 + offset).clamp(0, width as i32 - 1) as u32;
                let sample = img.get_pixel(sample_x, y)[0] as f32 / 255.0;
                let weight = gaussian(sigma, offset as f32);

                sum += sample * weight;
                weight_sum += weight;
            }

            // Normalize and convert back to u8
            let result = (sum / weight_sum).clamp(0.0, 1.0);
            output.put_pixel(x, y, Luma([(result * 255.0) as u8]));
        }
    }

    output
}

/// Apply vertical Gaussian blur
///
/// This implements the vertical pass of the separable Gaussian blur
/// Corresponds to PS_VerticalBlurAndDifference from AcerolaFX_ASCII.fx:296
/// (without the DoG part)
///
/// # Arguments
/// * `img` - Input grayscale image
/// * `sigma` - Standard deviation of the Gaussian
/// * `kernel_size` - Radius of the kernel (total height = 2*kernel_size + 1)
///
/// # Returns
/// Vertically blurred image
pub fn gaussian_blur_v(img: &GrayImage, sigma: f32, kernel_size: u32) -> GrayImage {
    let (width, height) = img.dimensions();
    let mut output = GrayImage::new(width, height);
    let kernel_size = kernel_size as i32;

    for y in 0..height {
        for x in 0..width {
            let mut sum = 0.0;
            let mut weight_sum = 0.0;

            // Convolve with vertical Gaussian kernel
            for offset in -kernel_size..=kernel_size {
                let sample_y = (y as i32 + offset).clamp(0, height as i32 - 1) as u32;
                let sample = img.get_pixel(x, sample_y)[0] as f32 / 255.0;
                let weight = gaussian(sigma, offset as f32);

                sum += sample * weight;
                weight_sum += weight;
            }

            // Normalize and convert back to u8
            let result = (sum / weight_sum).clamp(0.0, 1.0);
            output.put_pixel(x, y, Luma([(result * 255.0) as u8]));
        }
    }

    output
}

/// Apply full 2D Gaussian blur (separable)
///
/// # Arguments
/// * `img` - Input grayscale image
/// * `sigma` - Standard deviation of the Gaussian
/// * `kernel_size` - Radius of the kernel
///
/// # Returns
/// Blurred image
pub fn gaussian_blur(img: &GrayImage, sigma: f32, kernel_size: u32) -> GrayImage {
    let temp = gaussian_blur_h(img, sigma, kernel_size);
    gaussian_blur_v(&temp, sigma, kernel_size)
}

/// Compute Difference of Gaussians (DoG) edge detection
///
/// DoG = blur(sigma1) - tau * blur(sigma2)
/// Then threshold: result >= threshold ? 1.0 : 0.0
///
/// This implements the core of PS_VerticalBlurAndDifference from AcerolaFX_ASCII.fx:296-317
///
/// # Arguments
/// * `img` - Input grayscale image
/// * `sigma1` - First Gaussian sigma (typically smaller)
/// * `sigma2` - Second Gaussian sigma (typically larger)
/// * `kernel_size` - Kernel radius for both blurs
/// * `tau` - Multiplier for second blur (default 1.0)
/// * `threshold` - Binary threshold value (default 0.005)
///
/// # Returns
/// Binary edge image (0 or 255)
pub fn difference_of_gaussians(
    img: &GrayImage,
    sigma1: f32,
    sigma2: f32,
    kernel_size: u32,
    tau: f32,
    threshold: f32,
) -> GrayImage {
    let (width, height) = img.dimensions();
    let mut output = GrayImage::new(width, height);

    // Apply two Gaussian blurs with different sigmas
    let blur1 = gaussian_blur(img, sigma1, kernel_size);
    let blur2 = gaussian_blur(img, sigma2, kernel_size);

    // Compute difference and threshold
    for y in 0..height {
        for x in 0..width {
            let g1 = blur1.get_pixel(x, y)[0] as f32 / 255.0;
            let g2 = blur2.get_pixel(x, y)[0] as f32 / 255.0;

            // DoG formula from shader: D = (blur1 - tau * blur2)
            let dog = g1 - tau * g2;

            // Binary threshold: D >= threshold ? 1 : 0
            let result = if dog >= threshold { 255 } else { 0 };
            output.put_pixel(x, y, Luma([result]));
        }
    }

    output
}

/// Apply Sobel filter to detect edge gradients and directions
///
/// This implements PS_HorizontalSobel and PS_VerticalSobel from AcerolaFX_ASCII.fx:381-415
///
/// # Arguments
/// * `edges` - Binary edge image (from DoG)
///
/// # Returns
/// A tuple of (angles, valid_mask) where:
/// - angles: Vec of edge angles in radians (atan2(Gy, Gx))
/// - valid_mask: Vec of booleans indicating if the edge is valid (non-zero gradient)
pub fn sobel_filter(edges: &GrayImage) -> (Vec<f32>, Vec<bool>) {
    let (width, height) = edges.dimensions();
    let size = (width * height) as usize;

    let mut angles = vec![0.0; size];
    let mut valid_mask = vec![false; size];

    // Sobel kernels
    // Gx (horizontal):     Gy (vertical):
    // [-1  0  1]           [-1 -2 -1]
    // [-2  0  2]           [ 0  0  0]
    // [-1  0  1]           [ 1  2  1]

    for y in 1..(height - 1) {
        for x in 1..(width - 1) {
            // Get 3x3 neighborhood
            let nw = edges.get_pixel(x - 1, y - 1)[0] as f32;
            let n  = edges.get_pixel(x,     y - 1)[0] as f32;
            let ne = edges.get_pixel(x + 1, y - 1)[0] as f32;
            let w  = edges.get_pixel(x - 1, y    )[0] as f32;
            let e  = edges.get_pixel(x + 1, y    )[0] as f32;
            let sw = edges.get_pixel(x - 1, y + 1)[0] as f32;
            let s  = edges.get_pixel(x,     y + 1)[0] as f32;
            let se = edges.get_pixel(x + 1, y + 1)[0] as f32;

            // Compute Sobel gradients
            let gx = (-nw + ne - 2.0*w + 2.0*e - sw + se) / 255.0;
            let gy = (-nw - 2.0*n - ne + sw + 2.0*s + se) / 255.0;

            let magnitude = (gx * gx + gy * gy).sqrt();
            let idx = (y * width + x) as usize;

            if magnitude > 0.01 {
                // Edge is valid if gradient magnitude is significant
                angles[idx] = gy.atan2(gx);  // angle = atan2(Gy, Gx)
                valid_mask[idx] = true;
            } else {
                angles[idx] = 0.0;
                valid_mask[idx] = false;
            }
        }
    }

    (angles, valid_mask)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_luminance_black() {
        let img = RgbaImage::from_pixel(10, 10, image::Rgba([0, 0, 0, 255]));
        let lum = calculate_luminance(&img);
        assert_eq!(lum.get_pixel(0, 0)[0], 0);
    }

    #[test]
    fn test_luminance_white() {
        let img = RgbaImage::from_pixel(10, 10, image::Rgba([255, 255, 255, 255]));
        let lum = calculate_luminance(&img);
        assert_eq!(lum.get_pixel(0, 0)[0], 255);
    }

    #[test]
    fn test_luminance_gray() {
        let img = RgbaImage::from_pixel(10, 10, image::Rgba([128, 128, 128, 255]));
        let lum = calculate_luminance(&img);
        // Should be close to 128
        let val = lum.get_pixel(0, 0)[0];
        assert!(val >= 127 && val <= 129);
    }

    #[test]
    fn test_gaussian_at_center() {
        let sigma = 1.0;
        let weight = gaussian(sigma, 0.0);
        // At center (pos=0), weight should be maximum
        assert!(weight > 0.3);
    }

    #[test]
    fn test_gaussian_symmetry() {
        let sigma = 2.0;
        let w1 = gaussian(sigma, 1.0);
        let w2 = gaussian(sigma, -1.0);
        // Should be symmetric
        assert!((w1 - w2).abs() < 0.0001);
    }

    #[test]
    fn test_gaussian_blur_preserves_dimensions() {
        let img = GrayImage::new(64, 64);
        let blurred = gaussian_blur(&img, 1.0, 2);
        assert_eq!(blurred.dimensions(), (64, 64));
    }

    #[test]
    fn test_dog_output_is_binary() {
        let img = GrayImage::from_pixel(32, 32, Luma([128]));
        let dog = difference_of_gaussians(&img, 1.0, 1.6, 2, 1.0, 0.005);

        // All pixels should be either 0 or 255
        for pixel in dog.pixels() {
            assert!(pixel[0] == 0 || pixel[0] == 255);
        }
    }

    #[test]
    fn test_sobel_filter_dimensions() {
        let edges = GrayImage::new(64, 64);
        let (angles, valid) = sobel_filter(&edges);
        assert_eq!(angles.len(), 64 * 64);
        assert_eq!(valid.len(), 64 * 64);
    }
}

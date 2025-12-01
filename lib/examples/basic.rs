/// Basic example: Convert a simple test image to ASCII art
///
/// This creates a test image with some basic shapes and converts it to ASCII
use ascii_rendr::{AsciiConfig, process_image};
use image::{Rgba, RgbaImage};

fn main() {
    println!("ASCII Renderer - Basic Example");
    println!("==============================\n");

    // Create a simple 160x160 test image (20x20 tiles @ 8x8 pixels)
    let width = 160;
    let height = 160;
    let mut img = RgbaImage::new(width, height);

    // Fill with gray background
    for y in 0..height {
        for x in 0..width {
            img.put_pixel(x, y, Rgba([100, 100, 100, 255]));
        }
    }

    // Draw a white circle in the center
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    let radius = 50.0;

    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < radius {
                // White circle
                img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
            } else if (dist - radius).abs() < 5.0 {
                // Black edge
                img.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            }
        }
    }

    // Draw a diagonal line
    for i in 0..width {
        img.put_pixel(i, i, Rgba([255, 0, 0, 255]));
        if i > 0 {
            img.put_pixel(i - 1, i, Rgba([255, 0, 0, 255]));
            img.put_pixel(i, i - 1, Rgba([255, 0, 0, 255]));
        }
    }

    println!("Created test image: {}x{}", width, height);

    // Configure ASCII conversion
    let config = AsciiConfig {
        sigma: 2.0,
        sigma_scale: 1.6,
        kernel_size: 2,
        tau: 1.0,
        threshold: 0.01,
        edge_threshold: 8,
        ascii_color: [0, 255, 0], // Green ASCII
        bg_color: [0, 0, 0],      // Black background
        draw_edges: true,
        draw_fill: true,
        invert_luminance: false,
    };

    println!("Processing with config:");
    println!("  - Sigma: {}", config.sigma);
    println!("  - Edge threshold: {}", config.edge_threshold);
    println!("  - Draw edges: {}", config.draw_edges);
    println!("  - Draw fill: {}", config.draw_fill);
    println!();

    // Process the image
    let output = process_image(&img, &config);

    // Save both images
    img.save("basic_input.png").expect("Failed to save input");
    output
        .save("basic_output.png")
        .expect("Failed to save output");

    println!("✓ Saved input to:  basic_input.png");
    println!("✓ Saved output to: basic_output.png");
    println!("\nASCII conversion complete!");
}

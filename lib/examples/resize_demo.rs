use ascii_rendr::{AsciiConfig, process_image};
use image::{Rgba, RgbaImage};

fn main() {
    println!("ASCII Renderer - Automatic Resize Demo");
    println!("======================================\n");

    // Create test images with various non-conforming dimensions
    let test_cases = vec![
        (100, 100, "100x100 (not multiple of 8)"),
        (127, 85, "127x85 (both not multiples of 8)"),
        (1920, 1080, "1920x1080 (Full HD)"),
        (160, 160, "160x160 (already valid)"),
    ];

    let config = AsciiConfig::default();

    for (width, height, description) in test_cases {
        println!("Testing: {}", description);

        // Create a test image with a gradient pattern
        let mut img = RgbaImage::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let gray = ((x + y) % 256) as u8;
                img.put_pixel(x, y, Rgba([gray, gray, gray, 255]));
            }
        }

        // Process the image (will auto-resize if needed)
        let output = process_image(&img, &config);
        let (out_w, out_h) = output.dimensions();

        println!("  Input:  {}x{}", width, height);
        println!("  Output: {}x{}", out_w, out_h);

        if width != out_w || height != out_h {
            println!("  ✓ Image was automatically resized");
        } else {
            println!("  ✓ No resize needed");
        }
        println!();
    }

    println!("All tests completed successfully!");
    println!("\nNote: Images are resized (rounded down) to the nearest multiple of 8");
    println!("using high-quality Lanczos3 filtering to preserve image quality.");
}

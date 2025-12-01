mod app;

use app::AsciiApp;
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    // Configure logging
    env_logger::init();

    // Configure viewport/window
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("ASCII Renderer")
            .with_icon(load_icon()),
        ..Default::default()
    };

    // Run the application
    eframe::run_native(
        "ASCII Renderer",
        options,
        Box::new(|cc| Box::new(AsciiApp::new(cc))),
    )
}

/// Load application icon (placeholder for now)
fn load_icon() -> egui::IconData {
    // Create a simple 32x32 icon with ASCII art pattern
    let icon_size = 32;
    let mut pixels = vec![0u8; icon_size * icon_size * 4];

    // Simple pattern: white '#' on green background
    for y in 0..icon_size {
        for x in 0..icon_size {
            let idx = (y * icon_size + x) * 4;

            // Draw a simple hash pattern
            let is_hash = (x % 8 == 2 || x % 8 == 5) || (y % 8 == 2 || y % 8 == 5);

            if is_hash {
                pixels[idx] = 255;     // R
                pixels[idx + 1] = 255; // G
                pixels[idx + 2] = 255; // B
                pixels[idx + 3] = 255; // A
            } else {
                pixels[idx] = 0;       // R
                pixels[idx + 1] = 100; // G
                pixels[idx + 2] = 0;   // B
                pixels[idx + 3] = 255; // A
            }
        }
    }

    egui::IconData {
        rgba: pixels,
        width: icon_size as u32,
        height: icon_size as u32,
    }
}

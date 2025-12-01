//! ASCII Renderer - CPU-based image to ASCII art converter
//!
//! This library implements the Acerola shader algorithms for converting images
//! to ASCII art using CPU-based image processing.
//!
//! # Example
//! ```no_run
//! use ascii_rendr::{process_image, AsciiConfig};
//! use image;
//!
//! let input = image::open("photo.jpg").unwrap().to_rgba8();
//! let config = AsciiConfig::default();
//! let output = process_image(&input, &config);
//! output.save("ascii_art.png").unwrap();
//! ```

pub mod ascii;
pub mod config;
pub mod edges;
pub mod filters;
pub mod lut;
pub mod processor;

// Re-export main types for convenience
pub use config::AsciiConfig;
pub use processor::{process_image, process_image_preserve_colors};

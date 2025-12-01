# ASCII Renderer

[![CI](https://github.com/elwqnn/ascii-rendr/workflows/CI/badge.svg)](https://github.com/elwqnn/ascii-rendr/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/ascii-rendr.svg)](https://crates.io/crates/ascii-rendr)
[![Documentation](https://docs.rs/ascii-rendr/badge.svg)](https://docs.rs/ascii-rendr)

A CPU-based ASCII art converter implementing [Acerola's](https://www.youtube.com/@Acerola_t) shader algorithms in Rust.

## Features

- Advanced edge detection using Difference of Gaussians (DoG) and Sobel filters
- 8×8 tile-based directional ASCII character selection
- Interactive egui GUI with real-time parameter tweaking
- Multi-threaded processing with Rayon (2-4x speedup)
- 11 configurable parameters

## Usage

### GUI

```bash
cargo run --bin ascii-gui
```

Load images via `File > Open Image`, adjust parameters, and export with `File > Save Output`.

### Library

```rust
use ascii_rendr::{process_image, AsciiConfig};

let input = image::open("photo.jpg")?.to_rgba8();
let config = AsciiConfig::default();
let output = process_image(&input, &config);
output.save("ascii_art.png")?;
```

### CLI Example

```bash
cargo run --example basic
```

## Algorithm

7-step pipeline based on AcerolaFX ASCII shader:

1. Luminance extraction
2. Difference of Gaussians (DoG) edge detection
3. Sobel filter for edge gradients
4. Tile-based edge direction voting (8×8)
5. Luminance downscaling per tile
6. ASCII character selection
7. Character rendering to image

## Configuration

Key parameters: `kernel_size` (1-10), `sigma` (0.0-5.0), `edge_threshold` (0-64), `ascii_color`, `bg_color`. See code documentation for full list.

## Building

```bash
cargo build --release
cargo test
```

## Credits

- Algorithm: [Acerola's](https://www.youtube.com/@Acerola_t) ASCII shader
- Implementation: Rust + [egui](https://github.com/emilk/egui) + [image-rs](https://github.com/image-rs/image)

## License

MIT License - see [LICENSE](LICENSE)

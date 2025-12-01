use crate::config::AsciiConfig;
use crate::edges::EdgeDirection;
use crate::lut::{get_edge_char, get_fill_char};
use image::{GrayImage, Rgba, RgbaImage};
use rayon::prelude::*;

/// Select ASCII character for a tile
///
/// Based on CS_RenderASCII logic from AcerolaFX_ASCII.fx:478-496
///
/// # Arguments
/// * `edge_dir` - Edge direction for this tile
/// * `luminance` - Average luminance for this tile [0.0, 1.0]
/// * `tile_x` - Tile X coordinate
/// * `tile_y` - Tile Y coordinate
/// * `local_x` - Local X within tile (0-7)
/// * `local_y` - Local Y within tile (0-7)
/// * `config` - Configuration settings
///
/// # Returns
/// The ASCII character to render
pub fn select_ascii_char(
    edge_dir: EdgeDirection,
    luminance: f32,
    _tile_x: u32,
    _tile_y: u32,
    local_x: u32,
    local_y: u32,
    config: &AsciiConfig,
) -> char {
    // Priority: edges first, then fill
    // Matches shader logic at line 478-496
    if config.draw_edges && edge_dir != EdgeDirection::None {
        get_edge_char(edge_dir, local_x, local_y)
    } else if config.draw_fill {
        get_fill_char(luminance, config.invert_luminance)
    } else {
        ' '
    }
}

/// Downscale image luminance to 8×8 tiles by averaging
///
/// # Arguments
/// * `lum` - Input luminance image
/// * `tile_size` - Size of tiles (8)
///
/// # Returns
/// Vec of average luminance values, one per tile
pub fn downscale_to_tiles(lum: &GrayImage, tile_size: u32) -> Vec<f32> {
    let (width, height) = lum.dimensions();
    assert!(width % tile_size == 0 && height % tile_size == 0);

    let tile_width = width / tile_size;
    let tile_height = height / tile_size;
    let num_tiles = (tile_width * tile_height) as usize;

    // Parallelize tile averaging
    (0..num_tiles)
        .into_par_iter()
        .map(|tile_idx| {
            let tile_x = (tile_idx as u32) % tile_width;
            let tile_y = (tile_idx as u32) / tile_width;
            let mut sum = 0.0;

            // Average all pixels in this tile
            for local_y in 0..tile_size {
                for local_x in 0..tile_size {
                    let px = tile_x * tile_size + local_x;
                    let py = tile_y * tile_size + local_y;
                    sum += lum.get_pixel(px, py)[0] as f32 / 255.0;
                }
            }

            sum / (tile_size * tile_size) as f32
        })
        .collect()
}

/// Select ASCII characters for all tiles
///
/// # Arguments
/// * `edges` - Vec of edge directions, one per tile
/// * `tile_lum` - Vec of average luminance values, one per tile
/// * `tile_width` - Number of tiles horizontally
/// * `tile_height` - Number of tiles vertically
/// * `config` - Configuration settings
///
/// # Returns
/// 2D array of characters: [tile][pixel_in_tile] where pixel_in_tile is 64 chars (8x8)
pub fn select_ascii_chars(
    edges: &[EdgeDirection],
    tile_lum: &[f32],
    tile_width: u32,
    tile_height: u32,
    config: &AsciiConfig,
) -> Vec<Vec<char>> {
    let num_tiles = (tile_width * tile_height) as usize;
    assert_eq!(edges.len(), num_tiles);
    assert_eq!(tile_lum.len(), num_tiles);

    // Parallelize tile processing
    (0..num_tiles)
        .into_par_iter()
        .map(|tile_idx| {
            let tile_x = (tile_idx as u32) % tile_width;
            let tile_y = (tile_idx as u32) / tile_width;
            let edge_dir = edges[tile_idx];
            let lum = tile_lum[tile_idx];

            // Generate 64 characters for this 8x8 tile
            let mut tile_chars = Vec::with_capacity(64);

            for local_y in 0..8 {
                for local_x in 0..8 {
                    let ch =
                        select_ascii_char(edge_dir, lum, tile_x, tile_y, local_x, local_y, config);
                    tile_chars.push(ch);
                }
            }

            tile_chars
        })
        .collect()
}

/// Render ASCII characters to an image
///
/// Creates an 8x8 pixel representation of each character
/// This is a simple bitmap rendering - later could use actual font rendering
///
/// # Arguments
/// * `chars` - 2D array of characters (one vec per tile, 64 chars per tile)
/// * `tile_width` - Number of tiles horizontally
/// * `tile_height` - Number of tiles vertically
/// * `config` - Configuration with colors
///
/// # Returns
/// RGBA image with rendered ASCII art
pub fn render_ascii_to_image(
    chars: &[Vec<char>],
    tile_width: u32,
    tile_height: u32,
    config: &AsciiConfig,
) -> RgbaImage {
    render_ascii_to_image_with_source(chars, tile_width, tile_height, config, None)
}

/// Render ASCII characters to an image with optional color preservation
///
/// Creates an 8x8 pixel representation of each character
///
/// # Arguments
/// * `chars` - 2D array of characters (one vec per tile, 64 chars per tile)
/// * `tile_width` - Number of tiles horizontally
/// * `tile_height` - Number of tiles vertically
/// * `config` - Configuration with colors
/// * `source_image` - Optional source image to sample colors from
///
/// # Returns
/// RGBA image with rendered ASCII art
pub fn render_ascii_to_image_with_source(
    chars: &[Vec<char>],
    tile_width: u32,
    tile_height: u32,
    config: &AsciiConfig,
    source_image: Option<&RgbaImage>,
) -> RgbaImage {
    let width = tile_width * 8;
    let height = tile_height * 8;
    let mut output = RgbaImage::new(width, height);

    let fg_color = Rgba([
        config.ascii_color[0],
        config.ascii_color[1],
        config.ascii_color[2],
        255,
    ]);
    let bg_color = Rgba([
        config.bg_color[0],
        config.bg_color[1],
        config.bg_color[2],
        255,
    ]);

    for tile_y in 0..tile_height {
        for tile_x in 0..tile_width {
            let tile_idx = (tile_y * tile_width + tile_x) as usize;
            let tile_chars = &chars[tile_idx];

            for local_y in 0..8 {
                for local_x in 0..8 {
                    let char_idx = (local_y * 8 + local_x) as usize;
                    let ch = tile_chars[char_idx];

                    let px = tile_x * 8 + local_x;
                    let py = tile_y * 8 + local_y;

                    // Determine color based on source image or config
                    let color = if let Some(src) = source_image {
                        // Sample color from source image at this pixel
                        let src_pixel = src.get_pixel(px, py);
                        if should_draw_pixel(ch, local_x, local_y) {
                            *src_pixel // Use original color for foreground
                        } else {
                            // Darken the original color for background
                            Rgba([
                                (src_pixel[0] as f32 * 0.2) as u8,
                                (src_pixel[1] as f32 * 0.2) as u8,
                                (src_pixel[2] as f32 * 0.2) as u8,
                                255,
                            ])
                        }
                    } else {
                        // Use solid colors from config
                        if should_draw_pixel(ch, local_x, local_y) {
                            fg_color
                        } else {
                            bg_color
                        }
                    };

                    output.put_pixel(px, py, color);
                }
            }
        }
    }

    output
}

/// Determine if a pixel should be drawn for a character at a given position
///
/// This is a simple 8x8 bitmap representation of ASCII characters
/// In a real implementation, this would use actual font rendering
///
/// # Arguments
/// * `ch` - The character
/// * `x` - X position within 8x8 grid (0-7)
/// * `y` - Y position within 8x8 grid (0-7)
///
/// # Returns
/// true if pixel should be drawn (foreground color), false for background
fn should_draw_pixel(ch: char, x: u32, y: u32) -> bool {
    match ch {
        ' ' => false, // Space: always empty

        '|' => x == 3 || x == 4, // Vertical bar in middle

        '-' => y == 3 || y == 4, // Horizontal bar in middle

        '/' => {
            // Diagonal from bottom-left to top-right
            let expected_x = 7 - y;
            x == expected_x || x == expected_x.saturating_sub(1)
        }

        '\\' => {
            // Diagonal from top-left to bottom-right
            x == y || x == y.saturating_sub(1)
        }

        '.' => (3..=4).contains(&x) && (3..=4).contains(&y), // Small dot in center

        ':' => {
            // Two dots vertically
            (3..=4).contains(&x) && (y == 2 || y == 5)
        }

        '=' => y == 2 || y == 5, // Two horizontal lines

        '+' => {
            // Plus sign
            (x == 3 || x == 4) || (y == 3 || y == 4)
        }

        '*' => {
            // Star/asterisk - simplified
            (x == 3 || x == 4) || (y == 3 || y == 4) || (x == y) || (x == 7 - y)
        }

        '#' => {
            // Hash/pound
            (x == 2 || x == 5) || (y == 2 || y == 5)
        }

        '%' => {
            // Percent - simplified
            (x + y == 7) || (x == 1 && y == 1) || (x == 6 && y == 6)
        }

        '@' => {
            // At symbol - filled circle approximation
            let dx = x as i32 - 3;
            let dy = y as i32 - 3;
            dx * dx + dy * dy <= 12
        }

        _ => {
            // Unknown character: use a filled square
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Luma;

    #[test]
    fn test_select_ascii_char_edge() {
        let config = AsciiConfig {
            draw_edges: true,
            draw_fill: true,
            ..Default::default()
        };

        let ch = select_ascii_char(EdgeDirection::Vertical, 0.5, 0, 0, 0, 0, &config);
        assert_eq!(ch, '|');
    }

    #[test]
    fn test_select_ascii_char_fill() {
        let config = AsciiConfig {
            draw_edges: false,
            draw_fill: true,
            ..Default::default()
        };

        let ch = select_ascii_char(EdgeDirection::None, 0.0, 0, 0, 0, 0, &config);
        assert_eq!(ch, ' '); // Darkest = space

        let ch = select_ascii_char(EdgeDirection::None, 1.0, 0, 0, 0, 0, &config);
        assert_eq!(ch, '@'); // Brightest = @
    }

    #[test]
    fn test_downscale_to_tiles() {
        // Create 16x16 image (2x2 tiles)
        let img = GrayImage::from_pixel(16, 16, Luma([128]));
        let tiles = downscale_to_tiles(&img, 8);

        assert_eq!(tiles.len(), 4); // 2x2 tiles
        // All tiles should have average luminance ~0.5 (128/255)
        for &lum in &tiles {
            assert!((lum - 0.5).abs() < 0.01);
        }
    }

    #[test]
    fn test_select_ascii_chars() {
        let edges = vec![EdgeDirection::Vertical, EdgeDirection::None];
        let tile_lum = vec![0.5, 0.8];
        let config = AsciiConfig::default();

        let chars = select_ascii_chars(&edges, &tile_lum, 2, 1, &config);

        assert_eq!(chars.len(), 2); // 2 tiles
        assert_eq!(chars[0].len(), 64); // 64 chars per tile
        assert_eq!(chars[1].len(), 64);
    }

    #[test]
    fn test_render_ascii_to_image() {
        let chars = vec![
            vec!['|'; 64], // Tile 0: all vertical bars
            vec![' '; 64], // Tile 1: all spaces
        ];
        let config = AsciiConfig::default();

        let img = render_ascii_to_image(&chars, 2, 1, &config);

        assert_eq!(img.dimensions(), (16, 8)); // 2 tiles wide, 1 tile high, 8x8 pixels each
    }

    #[test]
    fn test_should_draw_pixel_space() {
        assert!(!should_draw_pixel(' ', 0, 0));
        assert!(!should_draw_pixel(' ', 7, 7));
    }

    #[test]
    fn test_should_draw_pixel_vertical() {
        assert!(should_draw_pixel('|', 3, 0));
        assert!(should_draw_pixel('|', 4, 7));
        assert!(!should_draw_pixel('|', 0, 0));
    }

    #[test]
    fn test_should_draw_pixel_horizontal() {
        assert!(should_draw_pixel('-', 0, 3));
        assert!(should_draw_pixel('-', 7, 4));
        assert!(!should_draw_pixel('-', 0, 0));
    }
}

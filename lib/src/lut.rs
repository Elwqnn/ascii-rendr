//! ASCII character lookup tables
//!
//! These define the character sets used for edges and luminance-based fill.
//! Later these can be loaded from PNG files (edgesASCII.png, fillASCII.png).

use crate::edges::EdgeDirection;

/// Edge characters organized by direction
///
/// Each edge direction gets 8 characters (one for each position in the 8x8 tile)
/// This is a simplified version - in the full shader, these come from a texture
pub const EDGE_CHARS: [[char; 8]; 4] = [
    // Vertical: |
    ['|', '|', '|', '|', '|', '|', '|', '|'],

    // Horizontal: -
    ['-', '-', '-', '-', '-', '-', '-', '-'],

    // Diagonal1: /
    ['/', '/', '/', '/', '/', '/', '/', '/'],

    // Diagonal2: \
    ['\\', '\\', '\\', '\\', '\\', '\\', '\\', '\\'],
];

/// Fill characters organized by luminance level
///
/// 10 levels from darkest (space) to brightest (@)
/// Based on common ASCII art ramps
pub const FILL_CHARS: [char; 10] = [
    ' ',  // 0: darkest
    '.',  // 1
    ':',  // 2
    '-',  // 3
    '=',  // 4
    '+',  // 5
    '*',  // 6
    '#',  // 7
    '%',  // 8
    '@',  // 9: brightest
];

/// Get the appropriate edge character for a direction and tile position
///
/// # Arguments
/// * `direction` - The edge direction
/// * `tile_x` - X position within the tile (0-7)
/// * `tile_y` - Y position within the tile (0-7)
///
/// # Returns
/// The character to use for this edge
pub fn get_edge_char(direction: EdgeDirection, tile_x: u32, tile_y: u32) -> char {
    assert!(tile_x < 8 && tile_y < 8, "Tile coordinates must be 0-7");

    match direction {
        EdgeDirection::Vertical => EDGE_CHARS[0][tile_y as usize],
        EdgeDirection::Horizontal => EDGE_CHARS[1][tile_y as usize],
        EdgeDirection::Diagonal1 => EDGE_CHARS[2][tile_y as usize],
        EdgeDirection::Diagonal2 => EDGE_CHARS[3][tile_y as usize],
        EdgeDirection::None => ' ',
    }
}

/// Get the appropriate fill character for a luminance value
///
/// # Arguments
/// * `luminance` - Normalized luminance value [0.0, 1.0]
/// * `invert` - Whether to invert the luminance mapping
///
/// # Returns
/// The character to use for this luminance
pub fn get_fill_char(luminance: f32, invert: bool) -> char {
    let mut lum = luminance.clamp(0.0, 1.0);

    if invert {
        lum = 1.0 - lum;
    }

    // Quantize to 0-9 range
    // Shader logic: luminance = max(0, (floor(luminance * 10) - 1)) / 10.0f;
    // We just need the index, so: floor(luminance * 10)
    let index = (lum * 10.0).floor() as usize;
    let index = index.min(9);  // Clamp to 0-9

    FILL_CHARS[index]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_edge_char_vertical() {
        assert_eq!(get_edge_char(EdgeDirection::Vertical, 0, 0), '|');
        assert_eq!(get_edge_char(EdgeDirection::Vertical, 7, 7), '|');
    }

    #[test]
    fn test_get_edge_char_horizontal() {
        assert_eq!(get_edge_char(EdgeDirection::Horizontal, 0, 0), '-');
    }

    #[test]
    fn test_get_edge_char_diagonal1() {
        assert_eq!(get_edge_char(EdgeDirection::Diagonal1, 0, 0), '/');
    }

    #[test]
    fn test_get_edge_char_diagonal2() {
        assert_eq!(get_edge_char(EdgeDirection::Diagonal2, 0, 0), '\\');
    }

    #[test]
    fn test_get_edge_char_none() {
        assert_eq!(get_edge_char(EdgeDirection::None, 0, 0), ' ');
    }

    #[test]
    fn test_get_fill_char_darkest() {
        assert_eq!(get_fill_char(0.0, false), ' ');
    }

    #[test]
    fn test_get_fill_char_brightest() {
        assert_eq!(get_fill_char(1.0, false), '@');
    }

    #[test]
    fn test_get_fill_char_mid() {
        let mid_char = get_fill_char(0.5, false);
        assert!(FILL_CHARS.contains(&mid_char));
    }

    #[test]
    fn test_get_fill_char_inverted() {
        // Dark should become bright
        assert_eq!(get_fill_char(0.0, true), '@');
        // Bright should become dark
        assert_eq!(get_fill_char(1.0, true), ' ');
    }

    #[test]
    #[should_panic(expected = "Tile coordinates must be 0-7")]
    fn test_get_edge_char_invalid_coords() {
        get_edge_char(EdgeDirection::Vertical, 8, 0);
    }
}

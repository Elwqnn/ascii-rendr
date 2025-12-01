use rayon::prelude::*;
use std::f32::consts::PI;

/// Edge direction classification for ASCII character selection
///
/// Corresponds to the direction classification in CS_RenderASCII from AcerolaFX_ASCII.fx:427-435
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EdgeDirection {
    None = -1,
    Vertical = 0,   // | (0° or 180°)
    Horizontal = 1, // - (90°)
    Diagonal1 = 2,  // / (45° to 90°, negative angles or 135° to 180°)
    Diagonal2 = 3,  // \ (45° to 90°, positive angles or -135° to -45°)
}

/// Classify edge direction from angle
///
/// Based on the shader logic from CS_RenderASCII:427-435
/// Angles are in radians from atan2(Gy, Gx)
///
/// # Arguments
/// * `angle` - Edge angle in radians [-π, π]
///
/// # Returns
/// EdgeDirection classification
pub fn classify_edge_direction(angle: f32) -> EdgeDirection {
    // Normalize angle to [0, π] and get absolute value
    let abs_theta = angle.abs() / PI;

    // Classification based on shader logic:
    // if ((0.0f <= absTheta) && (absTheta < 0.05f)) direction = 0; // VERTICAL
    // else if ((0.9f < absTheta) && (absTheta <= 1.0f)) direction = 0;
    // else if ((0.45f < absTheta) && (absTheta < 0.55f)) direction = 1; // HORIZONTAL
    // else if (0.05f < absTheta && absTheta < 0.45f) direction = sign(theta) > 0 ? 3 : 2; // DIAGONAL
    // else if (0.55f < absTheta && absTheta < 0.9f) direction = sign(theta) > 0 ? 2 : 3; // DIAGONAL

    if (0.0..0.05).contains(&abs_theta) || (0.9..=1.0).contains(&abs_theta) {
        EdgeDirection::Vertical
    } else if (0.45..0.55).contains(&abs_theta) {
        EdgeDirection::Horizontal
    } else if (0.05..0.45).contains(&abs_theta) {
        if angle > 0.0 {
            EdgeDirection::Diagonal2 // \ (positive angles)
        } else {
            EdgeDirection::Diagonal1 // / (negative angles)
        }
    } else if (0.55..0.9).contains(&abs_theta) {
        if angle > 0.0 {
            EdgeDirection::Diagonal1 // /
        } else {
            EdgeDirection::Diagonal2 // \
        }
    } else {
        EdgeDirection::None
    }
}

/// Detect edges with direction voting in 8×8 tiles
///
/// This implements the tile-based edge direction voting algorithm from CS_RenderASCII:418-465
/// Each 8×8 tile votes on the most common edge direction among its pixels
///
/// # Arguments
/// * `angles` - Vec of edge angles for each pixel (from Sobel filter)
/// * `valid_mask` - Vec of booleans indicating which pixels have valid edges
/// * `width` - Image width
/// * `height` - Image height
/// * `edge_threshold` - Minimum number of pixels in a tile needed to declare an edge
///
/// # Returns
/// Vec of EdgeDirection, one per 8×8 tile (size: (width/8) * (height/8))
pub fn detect_edges_tiled(
    angles: &[f32],
    valid_mask: &[bool],
    width: u32,
    height: u32,
    edge_threshold: u32,
) -> Vec<EdgeDirection> {
    assert_eq!(angles.len(), (width * height) as usize);
    assert_eq!(valid_mask.len(), (width * height) as usize);
    assert!(
        width.is_multiple_of(8) && height.is_multiple_of(8),
        "Dimensions must be multiples of 8"
    );

    let tile_width = width / 8;
    let tile_height = height / 8;
    let num_tiles = (tile_width * tile_height) as usize;

    // Parallelize tile processing
    (0..num_tiles)
        .into_par_iter()
        .map(|tile_idx| {
            let tile_x = (tile_idx as u32) % tile_width;
            let tile_y = (tile_idx as u32) / tile_width;

            // Count edge directions in this tile
            let mut buckets = [0u32; 4]; // [Vertical, Horizontal, Diagonal1, Diagonal2]

            // Scan all 64 pixels in this 8×8 tile
            for local_y in 0..8 {
                for local_x in 0..8 {
                    let pixel_x = tile_x * 8 + local_x;
                    let pixel_y = tile_y * 8 + local_y;
                    let idx = (pixel_y * width + pixel_x) as usize;

                    if valid_mask[idx] {
                        let direction = classify_edge_direction(angles[idx]);
                        match direction {
                            EdgeDirection::Vertical => buckets[0] += 1,
                            EdgeDirection::Horizontal => buckets[1] += 1,
                            EdgeDirection::Diagonal1 => buckets[2] += 1,
                            EdgeDirection::Diagonal2 => buckets[3] += 1,
                            EdgeDirection::None => {}
                        }
                    }
                }
            }

            // Find the most common edge direction (max bucket)
            let mut max_count = 0;
            let mut common_edge = EdgeDirection::None;

            for (i, &count) in buckets.iter().enumerate() {
                if count > max_count {
                    max_count = count;
                    common_edge = match i {
                        0 => EdgeDirection::Vertical,
                        1 => EdgeDirection::Horizontal,
                        2 => EdgeDirection::Diagonal1,
                        3 => EdgeDirection::Diagonal2,
                        _ => EdgeDirection::None,
                    };
                }
            }

            // Only use the edge if enough pixels voted for it
            // Matches shader logic: if (maxValue < _EdgeThreshold) commonEdgeIndex = -1;
            if max_count < edge_threshold {
                common_edge = EdgeDirection::None;
            }

            common_edge
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_vertical() {
        // Angles near 0° or 180°
        assert_eq!(classify_edge_direction(0.0), EdgeDirection::Vertical);
        assert_eq!(classify_edge_direction(0.01 * PI), EdgeDirection::Vertical);
        assert_eq!(classify_edge_direction(-0.01 * PI), EdgeDirection::Vertical);
        assert_eq!(classify_edge_direction(0.95 * PI), EdgeDirection::Vertical);
        assert_eq!(classify_edge_direction(-0.95 * PI), EdgeDirection::Vertical);
    }

    #[test]
    fn test_classify_horizontal() {
        // Angles near 90° (π/2)
        assert_eq!(classify_edge_direction(0.5 * PI), EdgeDirection::Horizontal);
        assert_eq!(
            classify_edge_direction(-0.5 * PI),
            EdgeDirection::Horizontal
        );
        assert_eq!(
            classify_edge_direction(0.48 * PI),
            EdgeDirection::Horizontal
        );
    }

    #[test]
    fn test_classify_diagonal() {
        // Positive angles in 0.05-0.45 range
        assert_eq!(classify_edge_direction(0.2 * PI), EdgeDirection::Diagonal2);
        assert_eq!(classify_edge_direction(0.3 * PI), EdgeDirection::Diagonal2);

        // Negative angles in -0.05 to -0.45 range
        assert_eq!(classify_edge_direction(-0.2 * PI), EdgeDirection::Diagonal1);
        assert_eq!(classify_edge_direction(-0.3 * PI), EdgeDirection::Diagonal1);
    }

    #[test]
    fn test_detect_edges_tiled_all_none() {
        // Image with no valid edges
        let width = 64;
        let height = 64;
        let angles = vec![0.0; (width * height) as usize];
        let valid = vec![false; (width * height) as usize];

        let edges = detect_edges_tiled(&angles, &valid, width, height, 8);

        // Should be 8×8 tiles
        assert_eq!(edges.len(), 8 * 8);

        // All should be None
        for edge in edges {
            assert_eq!(edge, EdgeDirection::None);
        }
    }

    #[test]
    fn test_detect_edges_tiled_uniform_vertical() {
        let width = 64;
        let height = 64;
        // All edges pointing vertically (0 radians)
        let angles = vec![0.0; (width * height) as usize];
        let valid = vec![true; (width * height) as usize];

        let edges = detect_edges_tiled(&angles, &valid, width, height, 8);

        // Should detect vertical edges in all tiles
        for edge in edges {
            assert_eq!(edge, EdgeDirection::Vertical);
        }
    }

    #[test]
    fn test_detect_edges_tiled_threshold() {
        let width = 64;
        let height = 64;
        let mut angles = vec![0.0; (width * height) as usize];
        let mut valid = vec![false; (width * height) as usize];

        // Set only 7 pixels in first tile to have vertical edges (below threshold of 8)
        for i in 0..7 {
            angles[i] = 0.0;
            valid[i] = true;
        }

        let edges = detect_edges_tiled(&angles, &valid, width, height, 8);

        // First tile should be None (7 < 8 threshold)
        assert_eq!(edges[0], EdgeDirection::None);
    }

    #[test]
    #[should_panic(expected = "must be multiples of 8")]
    fn test_detect_edges_invalid_dimensions() {
        let angles = vec![0.0; 100];
        let valid = vec![false; 100];
        detect_edges_tiled(&angles, &valid, 10, 10, 8); // Not multiples of 8
    }
}

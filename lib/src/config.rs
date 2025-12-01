/// Configuration for ASCII art conversion
#[derive(Debug, Clone)]
pub struct AsciiConfig {
    /// Blur settings
    pub kernel_size: u32,        // 1-10, default 2
    pub sigma: f32,              // 0.0-5.0, default 2.0
    pub sigma_scale: f32,        // DoG second sigma scale, default 1.6

    /// Edge detection
    pub tau: f32,                // DoG threshold multiplier, default 1.0
    pub threshold: f32,          // DoG threshold, default 0.005
    pub edge_threshold: u32,     // Pixels needed for edge (in 8x8 tile), default 8

    /// Colors
    pub ascii_color: [u8; 3],    // RGB, default white [255, 255, 255]
    pub bg_color: [u8; 3],       // RGB, default black [0, 0, 0]

    /// Rendering
    pub draw_edges: bool,        // default true
    pub draw_fill: bool,         // default true
    pub invert_luminance: bool,  // default false
}

impl Default for AsciiConfig {
    fn default() -> Self {
        Self {
            // Blur settings
            kernel_size: 2,
            sigma: 2.0,
            sigma_scale: 1.6,

            // Edge detection
            tau: 1.0,
            threshold: 0.005,
            edge_threshold: 8,

            // Colors
            ascii_color: [255, 255, 255],
            bg_color: [0, 0, 0],

            // Rendering
            draw_edges: true,
            draw_fill: true,
            invert_luminance: false,
        }
    }
}

impl AsciiConfig {
    /// Validates the configuration parameters
    pub fn validate(&self) -> Result<(), String> {
        if self.kernel_size < 1 || self.kernel_size > 10 {
            return Err(format!("kernel_size must be between 1 and 10, got {}", self.kernel_size));
        }
        if self.sigma < 0.0 || self.sigma > 5.0 {
            return Err(format!("sigma must be between 0.0 and 5.0, got {}", self.sigma));
        }
        if self.sigma_scale < 0.0 || self.sigma_scale > 5.0 {
            return Err(format!("sigma_scale must be between 0.0 and 5.0, got {}", self.sigma_scale));
        }
        if self.tau < 0.0 || self.tau > 1.1 {
            return Err(format!("tau must be between 0.0 and 1.1, got {}", self.tau));
        }
        if self.threshold < 0.001 || self.threshold > 0.1 {
            return Err(format!("threshold must be between 0.001 and 0.1, got {}", self.threshold));
        }
        if self.edge_threshold > 64 {
            return Err(format!("edge_threshold must be <= 64, got {}", self.edge_threshold));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_is_valid() {
        let config = AsciiConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_kernel_size() {
        let mut config = AsciiConfig::default();
        config.kernel_size = 0;
        assert!(config.validate().is_err());

        config.kernel_size = 11;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_sigma() {
        let mut config = AsciiConfig::default();
        config.sigma = -1.0;
        assert!(config.validate().is_err());

        config.sigma = 6.0;
        assert!(config.validate().is_err());
    }
}

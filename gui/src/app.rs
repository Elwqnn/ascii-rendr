use ascii_rendr::{AsciiConfig, process_image, process_image_preserve_colors};
use eframe::egui;
use image::RgbaImage;
use std::time::Instant;

/// Main application state for the ASCII renderer GUI
pub struct AsciiApp {
    /// Input image (original)
    input_image: Option<RgbaImage>,
    /// Output image (ASCII art)
    output_image: Option<RgbaImage>,
    /// Configuration parameters
    config: AsciiConfig,

    /// Texture handle for input image display
    input_texture: Option<egui::TextureHandle>,
    /// Texture handle for output image display
    output_texture: Option<egui::TextureHandle>,

    /// Whether to automatically reprocess when parameters change
    auto_process: bool,
    /// Flag indicating parameters have changed and reprocessing is needed
    needs_reprocess: bool,

    /// Whether to preserve original colors (vs using color picker)
    preserve_original_colors: bool,

    /// Last processing time in milliseconds
    last_process_time_ms: f64,
    /// Error message to display (if any)
    error_message: Option<String>,
}

impl Default for AsciiApp {
    fn default() -> Self {
        Self {
            input_image: None,
            output_image: None,
            config: AsciiConfig::default(),
            input_texture: None,
            output_texture: None,
            auto_process: false,
            needs_reprocess: false,
            preserve_original_colors: true,
            last_process_time_ms: 0.0,
            error_message: None,
        }
    }
}

impl AsciiApp {
    /// Create a new ASCII renderer application
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    /// Load an image from file path
    pub fn load_image(&mut self, path: &std::path::Path) {
        match image::open(path) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let (width, height) = rgba.dimensions();

                // Check if dimensions need adjustment (not multiples of 8)
                let target_width = (width / 8) * 8;
                let target_height = (height / 8) * 8;

                if width != target_width || height != target_height {
                    self.error_message = Some(format!(
                        "Image will be automatically resized from {}x{} to {}x{} (nearest multiple of 8)",
                        width, height, target_width, target_height
                    ));
                } else {
                    self.error_message = None;
                }

                self.input_image = Some(rgba);
                self.input_texture = None; // Clear old texture
                self.output_texture = None;
                self.needs_reprocess = true;
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to load image: {}", e));
            }
        }
    }

    /// Save the output image to file
    pub fn save_output(&self, path: &std::path::Path) -> Result<(), String> {
        match &self.output_image {
            Some(img) => img.save(path).map_err(|e| format!("Failed to save: {}", e)),
            None => Err("No output image to save".to_string()),
        }
    }

    /// Process the input image with current configuration
    fn process(&mut self) {
        if let Some(ref input) = self.input_image {
            let start = Instant::now();

            match self.config.validate() {
                Ok(_) => {
                    let output = if self.preserve_original_colors {
                        process_image_preserve_colors(input, &self.config)
                    } else {
                        process_image(input, &self.config)
                    };
                    self.last_process_time_ms = start.elapsed().as_secs_f64() * 1000.0;
                    self.output_image = Some(output);
                    self.output_texture = None; // Clear old texture
                    self.needs_reprocess = false;
                    self.error_message = None;
                }
                Err(e) => {
                    self.error_message = Some(format!("Invalid config: {}", e));
                }
            }
        }
    }

    /// Render the control panel UI
    fn render_controls(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;

        ui.heading("Controls");
        ui.separator();

        // Blur settings
        ui.collapsing("Blur Settings", |ui| {
            changed |= ui
                .add(egui::Slider::new(&mut self.config.kernel_size, 1..=10).text("Kernel Size"))
                .on_hover_text("Size of the blur kernel (radius)")
                .changed();

            changed |= ui
                .add(egui::Slider::new(&mut self.config.sigma, 0.0..=5.0).text("Sigma"))
                .on_hover_text("Gaussian blur standard deviation")
                .changed();

            changed |= ui
                .add(egui::Slider::new(&mut self.config.sigma_scale, 0.0..=5.0).text("Sigma Scale"))
                .on_hover_text("Scale for second Gaussian in DoG")
                .changed();
        });

        ui.add_space(8.0);

        // Edge detection settings
        ui.collapsing("Edge Detection", |ui| {
            changed |= ui
                .add(egui::Slider::new(&mut self.config.tau, 0.0..=1.1).text("Tau"))
                .on_hover_text("DoG threshold multiplier")
                .changed();

            changed |= ui
                .add(egui::Slider::new(&mut self.config.threshold, 0.001..=0.1).text("Threshold"))
                .on_hover_text("DoG binary threshold")
                .changed();

            changed |= ui
                .add(
                    egui::Slider::new(&mut self.config.edge_threshold, 0..=64)
                        .text("Edge Threshold"),
                )
                .on_hover_text("Pixels needed in 8x8 tile for edge detection")
                .changed();
        });

        ui.add_space(8.0);

        // Rendering settings
        ui.collapsing("Rendering", |ui| {
            changed |= ui
                .checkbox(&mut self.config.draw_edges, "Draw Edges")
                .on_hover_text("Render detected edges as ASCII characters")
                .changed();

            changed |= ui
                .checkbox(&mut self.config.draw_fill, "Draw Fill")
                .on_hover_text("Fill areas with luminance-based ASCII characters")
                .changed();

            changed |= ui
                .checkbox(&mut self.config.invert_luminance, "Invert Luminance")
                .on_hover_text("Invert brightness mapping")
                .changed();
        });

        ui.add_space(8.0);

        // Color settings
        ui.collapsing("Colors", |ui| {
            changed |= ui
                .checkbox(
                    &mut self.preserve_original_colors,
                    "Preserve Original Colors",
                )
                .on_hover_text("Keep colors from source image instead of using solid colors")
                .changed();

            ui.add_space(4.0);

            // Only show color pickers when not preserving original colors
            ui.add_enabled_ui(!self.preserve_original_colors, |ui| {
                let mut ascii_color = [
                    self.config.ascii_color[0] as f32 / 255.0,
                    self.config.ascii_color[1] as f32 / 255.0,
                    self.config.ascii_color[2] as f32 / 255.0,
                ];
                if ui.color_edit_button_rgb(&mut ascii_color).changed() {
                    self.config.ascii_color = [
                        (ascii_color[0] * 255.0) as u8,
                        (ascii_color[1] * 255.0) as u8,
                        (ascii_color[2] * 255.0) as u8,
                    ];
                    changed = true;
                }
                ui.label("ASCII Color");

                ui.add_space(4.0);

                let mut bg_color = [
                    self.config.bg_color[0] as f32 / 255.0,
                    self.config.bg_color[1] as f32 / 255.0,
                    self.config.bg_color[2] as f32 / 255.0,
                ];
                if ui.color_edit_button_rgb(&mut bg_color).changed() {
                    self.config.bg_color = [
                        (bg_color[0] * 255.0) as u8,
                        (bg_color[1] * 255.0) as u8,
                        (bg_color[2] * 255.0) as u8,
                    ];
                    changed = true;
                }
                ui.label("Background Color");
            });
        });

        ui.add_space(16.0);
        ui.separator();

        // Auto-process toggle
        ui.checkbox(&mut self.auto_process, "Auto-process")
            .on_hover_text("Automatically reprocess when parameters change");

        // Manual process button
        ui.add_enabled_ui(!self.auto_process || !self.needs_reprocess, |ui| {
            if ui.button("Process").clicked() {
                self.process();
            }
        });

        // Show processing time
        if self.last_process_time_ms > 0.0 {
            ui.label(format!("Last process: {:.1} ms", self.last_process_time_ms));
        }

        changed
    }

    /// Convert RgbaImage to egui ColorImage
    fn rgba_to_color_image(img: &RgbaImage) -> egui::ColorImage {
        let (width, height) = img.dimensions();
        let pixels = img
            .pixels()
            .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
            .collect();

        egui::ColorImage {
            size: [width as usize, height as usize],
            source_size: egui::Vec2::new(width as f32, height as f32),
            pixels,
        }
    }

    /// Display an image in the UI (standalone helper function)
    fn display_image(
        ui: &mut egui::Ui,
        image: Option<&RgbaImage>,
        texture: &mut Option<egui::TextureHandle>,
        label: &str,
    ) {
        ui.vertical(|ui| {
            ui.heading(label);

            if let Some(img) = image {
                // Update texture if needed
                if texture.is_none() {
                    let color_image = Self::rgba_to_color_image(img);
                    *texture = Some(ui.ctx().load_texture(
                        label,
                        color_image,
                        egui::TextureOptions::default(),
                    ));
                }

                // Display the texture
                if let Some(tex) = texture {
                    let size = tex.size_vec2();
                    let max_size = ui.available_size();
                    // Allow upscaling for small images, but limit to reasonable max scale
                    let scale = ((max_size.x / size.x).min(max_size.y / size.y)).min(4.0);
                    let display_size = size * scale;

                    ui.image((tex.id(), display_size));
                    ui.label(format!(
                        "{}x{} (scale: {:.1}x)",
                        img.width(),
                        img.height(),
                        scale
                    ));
                }
            } else {
                ui.label("No image loaded");
            }
        });
    }
}

impl eframe::App for AsciiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open Image...").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Images", &["png", "jpg", "jpeg", "bmp"])
                            .pick_file()
                        {
                            self.load_image(&path);
                        }
                        ui.close();
                    }

                    if ui.button("Save Output...").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("PNG", &["png"])
                            .save_file()
                            && let Err(e) = self.save_output(&path)
                        {
                            self.error_message = Some(e);
                        }
                        ui.close();
                    }

                    ui.separator();

                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.error_message = Some(
                            "ASCII Renderer\nBased on Acerola's shader algorithms\n\nBuilt with Rust + egui".to_string()
                        );
                        ui.close();
                    }
                });
            });
        });

        // Left panel: Controls
        egui::SidePanel::left("control_panel")
            .resizable(true)
            .default_width(250.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let changed = self.render_controls(ui);

                    if changed {
                        self.needs_reprocess = true;
                    }
                });
            });

        // Central panel: Image display
        egui::CentralPanel::default().show(ctx, |ui| {
            // Show error message if any
            if let Some(ref msg) = self.error_message {
                ui.colored_label(egui::Color32::RED, msg);
                if ui.button("Clear Error").clicked() {
                    self.error_message = None;
                }
                ui.separator();
            }

            // Auto-process if needed
            if self.auto_process && self.needs_reprocess && self.input_image.is_some() {
                self.process();
            }

            // Display images side-by-side
            ui.horizontal(|ui| {
                let available_width = ui.available_width();
                let half_width = available_width / 2.0 - 8.0;

                ui.allocate_ui_with_layout(
                    egui::vec2(half_width, ui.available_height()),
                    egui::Layout::top_down(egui::Align::Center),
                    |ui| {
                        Self::display_image(
                            ui,
                            self.input_image.as_ref(),
                            &mut self.input_texture,
                            "Original",
                        );
                    },
                );

                ui.separator();

                ui.allocate_ui_with_layout(
                    egui::vec2(half_width, ui.available_height()),
                    egui::Layout::top_down(egui::Align::Center),
                    |ui| {
                        Self::display_image(
                            ui,
                            self.output_image.as_ref(),
                            &mut self.output_texture,
                            "ASCII Output",
                        );
                    },
                );
            });
        });
    }
}

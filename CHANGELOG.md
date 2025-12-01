# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

## [0.1.0] - 2025-12-01

### Added
- CPU-based ASCII art converter implementing Acerola shader algorithms
- Interactive egui GUI with real-time parameter tweaking
- 7-step processing pipeline: luminance extraction, DoG edge detection, Sobel gradients, tile-based edge voting, character selection and rendering
- 11 configurable parameters (kernel size, sigma, thresholds, colors, etc.)
- Multi-threaded processing with Rayon (2-4x speedup on multi-core CPUs)
- Automatic image resizing to nearest multiple of 8
- 41 tests covering all modules
- CLI examples and documentation

### Performance
- Release builds: ~20-900ms for typical images (CPU-dependent)

[Unreleased]: https://github.com/elwqnn/ascii-rendr/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/elwqnn/ascii-rendr/releases/tag/v0.1.0

# Performance Notes and Optimizations

## CPU vs GPU Performance

**Yes, the processing time is longer because it's CPU-based.** This is expected because:

1. **GPU Architecture**: GPUs have thousands of cores designed for parallel image processing
2. **CPU Architecture**: CPUs have fewer cores (typically 4-16) but more general-purpose capabilities
3. **Memory Access**: GPUs have high-bandwidth memory optimized for texture operations

**Expected Performance:**
- **CPU (current)**: 50-500ms for typical images (depending on size and CPU cores)
- **GPU (planned)**: 1-16ms for real-time 60+ FPS rendering

## Current CPU Optimizations (v0.1.0)

We've implemented several optimizations to improve CPU performance:

### 1. Parallel Processing with Rayon
Added multi-threading using Rayon to parallelize:
- **Tile-based edge detection** (`edges.rs`): Process multiple 8×8 tiles concurrently
- **ASCII character selection** (`ascii.rs`): Select characters for tiles in parallel
- **Luminance downscaling** (`ascii.rs`): Average tile luminance in parallel

**Impact**: ~2-4x speedup on multi-core CPUs (depends on core count)

### 2. Efficient Memory Layout
- Pre-allocated vectors with `.with_capacity()` to reduce reallocations
- Single-pass algorithms where possible
- Minimize image cloning (only when resizing is needed)

### 3. Release Build Optimizations
Build with `--release` for significant speedups:
```bash
cargo build --release --bin ascii-gui
./target/release/ascii-gui
```

**Release mode includes:**
- Level 3 optimization (`-O3`)
- Link-time optimization (LTO)
- No debug symbols
- Inlining optimizations

**Impact**: ~10-30x faster than debug builds

## Future: GPU Acceleration

See `GPU_PLAN.md` for the complete roadmap. The GPU version will:

1. **Use wgpu** for cross-platform GPU compute
2. **Implement compute shaders** in WGSL for all 7 pipeline steps
3. **Support real-time processing** at 60+ FPS
4. **Run on all modern GPUs** (Vulkan, Metal, DX12, WebGPU)

The GPU version will be implemented as an **additional module**, keeping the CPU library available:
```
ascii-rendr/
├── lib/          # CPU implementation (current)
├── lib-gpu/      # GPU implementation (future)
└── gui/          # GUI supporting both backends
```

## Performance Comparison (Estimated)

| Resolution | CPU (Debug) | CPU (Release) | GPU (Planned) |
|------------|-------------|---------------|---------------|
| 160×160    | ~500ms      | ~20ms         | ~2ms          |
| 640×640    | ~8000ms     | ~250ms        | ~5ms          |
| 1920×1080  | ~30000ms    | ~900ms        | ~10ms         |

*Note: CPU times are approximate and vary based on processor*

## Benchmarking Your System

To measure performance on your hardware:

```bash
cargo run --example resize_demo --release
```

The output will show processing times for various image sizes.

## Recommendation

**For now:**
- Use the CPU version with `--release` builds
- Performance is acceptable for batch processing and interactive tweaking
- Multi-core CPUs will see better performance thanks to parallelization

**When GPU is needed:**
- Real-time video processing
- Interactive preview at 60+ FPS
- Processing very large images (>4K)
- Running on systems with powerful GPUs but weaker CPUs

## Keeping Both Implementations

As you requested, **we will absolutely keep the CPU library for posterity**:

✅ **Advantages of CPU version:**
- Works on any system (no GPU required)
- Easier to debug and understand
- Deterministic results
- Better for batch processing scripts
- Reference implementation for algorithm correctness

✅ **Advantages of GPU version (future):**
- Real-time performance
- Better for interactive tools
- Scales with GPU capability
- Can process video streams

Both implementations will be maintained and tested independently.

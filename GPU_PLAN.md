# ASCII Renderer Implementation Plan (GPU Version)
## Converting ReShade HLSL Shaders to Rust + wgpu

> **Note:** This is the original plan for a full GPU-accelerated real-time renderer.
> For the simpler CPU-based version we're building first, see the current development plan.

### Overview
Convert the AcerolaFX ASCII ReShade shader into a real-time, GPU-accelerated Rust library using wgpu. Target: 60+ FPS at 1080p with full feature parity including edge detection, luminance-based fill, and all 28+ parameters.

---

## Architecture

### Crate Structure
```
ascii-rendr/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Public API exports
│   ├── renderer.rs         # AsciiRenderer - main orchestrator
│   ├── pipeline.rs         # Pipeline creation & management
│   ├── config.rs           # AsciiConfig with 28+ parameters
│   ├── texture.rs          # Texture allocation & management
│   ├── bind_groups.rs      # Bind group layouts
│   ├── assets.rs           # ASCII LUT texture loading
│   └── shaders/
│       ├── common.wgsl         # Shared utilities (gaussian, luminance)
│       ├── luminance.wgsl      # Pass 1: Extract luminance
│       ├── downscale.wgsl      # Pass 2: 1/8 downscale
│       ├── blur_h.wgsl         # Pass 3: Horizontal Gaussian blur
│       ├── blur_v_dog.wgsl     # Pass 4: Vertical blur + DoG
│       ├── normals.wgsl        # Pass 5: Screen-space normals
│       ├── edge_detect.wgsl    # Pass 6: Combine edge sources
│       ├── sobel_h.wgsl        # Pass 7: Horizontal Sobel
│       ├── sobel_v.wgsl        # Pass 8: Vertical Sobel
│       └── ascii_compute.wgsl  # Pass 9: 8x8 tile ASCII rendering
├── examples/
│   ├── basic.rs            # Static image example
│   ├── interactive.rs      # Parameter tweaking GUI
│   └── AcerolaFX_*.fx      # Reference HLSL shaders (existing)
└── assets/
    ├── edgesASCII.png      # 40x8 edge character LUT
    └── fillASCII.png       # 80x8 fill character LUT
```

### Dependencies (Cargo.toml)
```toml
wgpu = "0.18"
bytemuck = { version = "1.14", features = ["derive"] }
image = "0.24"
glam = "0.25"
encase = "0.7"
winit = "0.29"  # For examples
pollster = "0.3"  # For examples
```

---

## Rendering Pipeline

### 9-Pass Architecture

| Pass | Type | Input | Output | Purpose |
|------|------|-------|--------|---------|
| 1 | Render | Color texture | Luminance (R16F) | Extract grayscale |
| 2 | Render | Color texture | Downscale (RGBA16F, 1/8 res) | Performance optimization |
| 3 | Render | Luminance | Blur H (RG16F) | Horizontal Gaussian (2 sigmas) |
| 4 | Render | Blur H | DoG (R16F) | Vertical Gaussian + Difference |
| 5 | Render | Depth texture | Normals (RGBA16F) | Screen-space normals |
| 6 | Render | DoG + Normals | Edges (R16F) | Combine edge sources |
| 7 | Render | Edges | Sobel H (RG16F) | Horizontal gradient |
| 8 | Render | Sobel H | Sobel V (RG16F) | Vertical gradient + angle |
| 9 | Compute | All | ASCII (RGBA16F) | 8x8 tile rendering |

### Texture Requirements
- **Intermediate textures**: 6 (with ping-pong reuse between blur/Sobel)
- **Input textures**: 2 (color RGBA8/16, depth D32F)
- **LUT textures**: 2 (edges 40x8 R8, fill 80x8 R8)
- **Output texture**: 1 (RGBA16F or output format)

### Bind Group Strategy
- **Group 0**: Textures (varies per pass - texture views)
- **Group 1**: Samplers (shared - point, linear, repeat)
- **Group 2**: Uniforms (config buffer, 256-byte aligned)

---

## Key Technical Details

### HLSL → WGSL Translation Notes

**Major Differences:**
- **Texture/Sampler separation**: HLSL `sampler2D` → WGSL `texture_2d<f32>` + `sampler`
- **Shared memory**: `groupshared` → `var<workgroup>`
- **Barriers**: `barrier()` → `workgroupBarrier()`
- **Built-ins**: Add `saturate()` helper as `clamp(x, 0.0, 1.0)`
- **Texture fetch**: `tex2D(s, uv)` → `textureSample(t, s, uv)`, `tex2Dfetch` → `textureLoad`

**Compute Shader Specifics:**
```wgsl
@compute @workgroup_size(8, 8, 1)
fn cs_render_ascii(@builtin(global_invocation_id) tid: vec3<u32>,
                   @builtin(local_invocation_id) gid: vec3<u32>) {
    // groupshared array for edge direction voting
    var<workgroup> edge_count: array<i32, 64>;
    // ... implementation
}
```

### Configuration System

**AsciiConfig Structure** (28+ parameters organized into categories):
```rust
pub struct AsciiConfig {
    pub preprocess: PreprocessSettings,
    pub color: ColorSettings,
    pub debug: DebugSettings,
}

pub struct PreprocessSettings {
    pub zoom: f32,                    // 0.0-5.0, default 1.0
    pub offset: Vec2,                 // -1.0 to 1.0
    pub kernel_size: i32,             // 1-10, default 2
    pub sigma: f32,                   // 0.0-5.0, default 2.0
    pub sigma_scale: f32,             // 0.0-5.0, default 1.6
    pub tau: f32,                     // 0.0-1.1, default 1.0
    pub threshold: f32,               // 0.001-0.1, default 0.005
    pub use_depth: bool,              // default true
    pub depth_threshold: f32,         // 0.0-5.0, default 0.1
    pub use_normals: bool,            // default true
    pub normal_threshold: f32,        // 0.0-5.0, default 0.1
    pub depth_cutoff: f32,            // 0.0-1000.0, default 0.0
    pub edge_threshold: i32,          // 0-64, default 8
}
```

**Uniform Buffer** (GPU representation with `encase` for alignment):
```rust
#[derive(ShaderType)]
pub struct AsciiUniforms {
    // Must follow std140 layout - vec3 padded to 16 bytes
    pub zoom: f32,
    pub offset: Vec2,
    pub kernel_size: i32,
    pub sigma: f32,
    pub sigma_scale: f32,
    pub tau: f32,
    pub threshold: f32,
    // ... all 28+ fields with proper padding
}
```

### Asset Management

**ASCII LUT Textures:**
- `edgesASCII.png`: 40×8 pixels, 5 columns × 8 rows
  - Column 0-4: Vertical, Horizontal, Diagonal1, Diagonal2, unused
  - Each character is 8×8 pixels
- `fillASCII.png`: 80×8 pixels, 10 columns × 8 rows
  - 10 luminance levels from dark to bright
  - Each character is 8×8 pixels

**Loading Strategy:**
1. Try loading from `assets/` directory
2. If missing, generate fallback textures programmatically
3. Upload to GPU as R8Unorm textures with repeat sampling

---

## Implementation Phases

### Phase 1: Foundation (Week 1)
**Goal:** Project scaffolding and core infrastructure

**Tasks:**
1. Set up Cargo.toml with dependencies
2. Create module structure (lib.rs, renderer.rs, config.rs, etc.)
3. Define AsciiConfig structs with all parameters
4. Implement AsciiUniforms with encase alignment
5. Create texture allocation system (TextureManager)
6. Set up basic wgpu device/queue initialization

**Deliverable:** Compiling project with config system and texture management

---

### Phase 2: Shader Translation (Week 2-3)
**Goal:** Translate all 9 HLSL passes to WGSL

**Tasks:**
1. Create `common.wgsl` with shared functions:
   - `gaussian(sigma, pos)` - Gaussian weight calculation
   - `luminance(color)` - RGB to luminance
   - `transform_uv(uv)` - Apply zoom/offset
2. Translate pixel shaders (passes 1-8):
   - luminance.wgsl (straightforward)
   - downscale.wgsl (includes luminance in alpha)
   - blur_h.wgsl (horizontal loop)
   - blur_v_dog.wgsl (vertical loop + DoG calculation)
   - normals.wgsl (depth gradient cross product)
   - edge_detect.wgsl (DoG + depth + normal edges)
   - sobel_h.wgsl (horizontal Sobel kernel)
   - sobel_v.wgsl (vertical Sobel + atan2 direction)
3. Translate compute shader (pass 9):
   - 8×8 workgroup with shared memory
   - Edge direction voting algorithm
   - ASCII LUT sampling
   - Luminance-based character selection

**Deliverable:** All 9 WGSL shaders written and syntax-validated

---

### Phase 3: Pipeline Construction (Week 3-4)
**Goal:** Wire up rendering pipeline with all passes

**Tasks:**
1. Create bind group layouts (3 groups: textures, samplers, uniforms)
2. Build render pipelines for passes 1-8:
   - Fullscreen triangle vertex shader
   - Correct texture formats for each pass
   - Proper blend states (none needed)
3. Build compute pipeline for pass 9:
   - Dispatch size: `(width+7)/8, (height+7)/8`
4. Implement pipeline chaining in renderer.rs:
   - Pass texture outputs as next pass inputs
   - Ping-pong buffer reuse
5. Create uniform buffer and update mechanism

**Deliverable:** Complete render graph executing all passes

---

### Phase 4: ASCII Integration (Week 4-5)
**Goal:** Get ASCII characters rendering correctly

**Tasks:**
1. Implement asset loading (assets.rs):
   - PNG loading with `image` crate
   - Fallback texture generation
2. Create ASCII LUT samplers (repeat addressing)
3. Debug compute shader:
   - Verify edge direction classification
   - Check LUT coordinate calculations
   - Validate 8×8 tile addressing
4. Tune edge threshold to get visible edges
5. Implement luminance-based fill character selection

**Deliverable:** Working ASCII art output with edges and fill

---

### Phase 5: Parameter Tuning & Optimization (Week 5-6)
**Goal:** Achieve 60+ FPS and good visual quality

**Tasks:**
1. Benchmark each pass (wgpu timestamp queries)
2. Optimize hot spots:
   - Reduce blur kernel size if needed
   - Minimize texture barriers
3. Implement runtime config updates:
   - `update_config(&mut self, config)` method
   - Write to uniform buffer
4. Tune default parameters for good visuals:
   - Sigma values for DoG
   - Edge/normal/depth thresholds
   - Edge count threshold
5. Add validation (parameter ranges)

**Deliverable:** 60+ FPS at 1080p with production-ready defaults

---

### Phase 6: Examples & Documentation (Week 6-7)
**Goal:** Usable library with clear examples

**Tasks:**
1. Create `examples/basic.rs`:
   - Load a PNG, convert to ASCII, save output
2. Create `examples/interactive.rs`:
   - Live parameter tweaking with egui
   - Real-time preview
3. Write API documentation:
   - Rustdoc for all public items
   - Usage guide in README.md
4. Add visual regression tests:
   - Known-good reference images
   - Automated comparison
5. Performance benchmarking suite

**Deliverable:** Polished, documented library ready for use

---

### Phase 7: Advanced Features (Optional)
**Goal:** Extended capabilities beyond original shader

**Potential additions:**
- Custom ASCII LUT support (user-provided fonts)
- Colored ASCII (preserve hue from original)
- Animation support (temporal stability)
- WASM target (for web)
- Higher quality edge detection (more kernels)

---

## Public API Design

### Core API
```rust
pub struct AsciiRenderer {
    // Private fields
}

impl AsciiRenderer {
    /// Create new renderer for given dimensions
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        config: AsciiConfig,
    ) -> Result<Self, AsciiError>;

    /// Render ASCII effect
    pub fn render(
        &mut self,
        color_input: &wgpu::TextureView,
        depth_input: &wgpu::TextureView,
        output: &wgpu::TextureView,
    ) -> Result<(), AsciiError>;

    /// Update configuration
    pub fn update_config(&mut self, config: &AsciiConfig);

    /// Get mutable config (changes take effect next frame)
    pub fn config_mut(&mut self) -> &mut AsciiConfig;

    /// Resize render targets
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32);
}
```

### Usage Example
```rust
use ascii_rendr::{AsciiRenderer, AsciiConfig};

// Initialize wgpu (device, queue, surface)
// ...

let config = AsciiConfig {
    preprocess: PreprocessSettings {
        sigma: 2.5,
        edge_threshold: 12,
        ..Default::default()
    },
    color: ColorSettings {
        ascii_color: [0.0, 1.0, 0.0].into(),  // Green
        blend_with_base: 0.2,
        ..Default::default()
    },
    ..Default::default()
};

let mut renderer = AsciiRenderer::new(
    &device,
    &queue,
    1920,
    1080,
    config,
)?;

// Render loop
loop {
    // ... render your scene to color_texture and depth_texture

    renderer.render(
        &color_texture.view,
        &depth_texture.view,
        &output_texture.view,
    )?;

    // Runtime adjustment
    renderer.config_mut().preprocess.sigma += 0.1;
}
```

---

## Critical Implementation Details

### Compute Shader Edge Voting Algorithm

**Original HLSL logic:**
1. Each thread in 8×8 workgroup calculates edge direction (0=vertical, 1=horizontal, 2/3=diagonal)
2. Writes direction to `groupshared int edgeCount[64]`
3. Thread (0,0) counts directions in buckets
4. Selects most common direction (majority vote)
5. If < threshold (default 8), discard edge
6. All threads use common edge direction to sample LUT

**WGSL translation:**
```wgsl
var<workgroup> edge_count: array<i32, 64>;

// Each thread writes its edge direction
edge_count[gid.x + gid.y * 8u] = direction;
workgroupBarrier();

// Thread (0,0) does voting
var common_edge_index: i32 = -1;
if gid.x == 0u && gid.y == 0u {
    var buckets: array<u32, 4> = array(0u, 0u, 0u, 0u);
    for (var i = 0; i < 64; i++) {
        if edge_count[i] >= 0 {
            buckets[edge_count[i]] += 1u;
        }
    }
    // Find max bucket
    // ...
    edge_count[0] = common_edge_index;
}
workgroupBarrier();

common_edge_index = edge_count[0];  // All threads read result
```

### Bind Group Layout (Group 0 Example - Final Compute Pass)
```rust
let compute_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    label: Some("ASCII Compute Bind Group Layout"),
    entries: &[
        // 0: Sobel texture (RG16F - edge angles)
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        },
        // 1: Downscale texture (RGBA16F - color + lum)
        wgpu::BindGroupLayoutEntry { binding: 1, /* ... */ },
        // 2: Normals texture (RGBA16F - xyz + depth)
        wgpu::BindGroupLayoutEntry { binding: 2, /* ... */ },
        // 3: EdgesASCII LUT
        wgpu::BindGroupLayoutEntry { binding: 3, /* ... */ },
        // 4: FillASCII LUT
        wgpu::BindGroupLayoutEntry { binding: 4, /* ... */ },
        // 5: Output storage texture
        wgpu::BindGroupLayoutEntry {
            binding: 5,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::StorageTexture {
                access: wgpu::StorageTextureAccess::WriteOnly,
                format: wgpu::TextureFormat::Rgba16Float,
                view_dimension: wgpu::TextureViewDimension::D2,
            },
            count: None,
        },
    ],
});
```

---

## Testing Strategy

### Unit Tests
- Config validation (parameter ranges)
- Texture allocation (correct formats/sizes)
- Uniform buffer packing (alignment)

### Integration Tests
- Full render with known input → compare output
- Pipeline creation (doesn't panic)
- Resize handling

### Visual Regression Tests
- Render reference images through pipeline
- Compare with known-good outputs
- Tolerance for minor GPU differences

### Performance Benchmarks
- Criterion.rs benchmarks for each pass
- 1080p/1440p/4K frame time measurements
- Memory allocation tracking

---

## Potential Challenges & Solutions

| Challenge | Solution |
|-----------|----------|
| **Depth buffer access** | Require user to provide separate depth texture view (can't access in ReShade-like way) |
| **LUT textures missing** | Generate simple ASCII fallbacks programmatically (draw basic shapes) |
| **Barrier overhead** | Use ping-pong buffers, minimize texture dependencies |
| **Different GPU behavior** | Test on multiple vendors (NVIDIA, AMD, Intel, Apple) |
| **WGSL differences** | Reference wgpu examples, use naga validation |
| **Shared memory limits** | 64 i32s = 256 bytes (well within limits) |

---

## Performance Optimization Checklist

- [ ] Use separable filters (blur, Sobel) instead of 2D
- [ ] Ping-pong texture reuse (blur→Sobel share buffers)
- [ ] Single-channel formats where possible (R16F vs RGBA16F)
- [ ] Point sampling for intermediate textures (no filtering needed)
- [ ] Compute dispatch size rounded up correctly
- [ ] Uniform buffer updates only when config changes
- [ ] Timestamp queries to identify bottlenecks
- [ ] Consider half-resolution intermediate textures for extreme performance

---

## Success Criteria

✅ **Functional:**
- ASCII art output matches ReShade shader quality
- All 28+ parameters functional
- Edge detection + luminance fill both working
- No visual artifacts or glitches

✅ **Performance:**
- 60+ FPS at 1080p on mid-range GPU (e.g., GTX 1060 / RX 580)
- < 5ms GPU time at 1080p
- Stable frame time (no stutters)

✅ **Quality:**
- Clean, documented public API
- Examples demonstrate all features
- Tests pass on CI
- Works on Windows, Linux, macOS

---

## Critical Files Reference

1. **examples/AcerolaFX_ASCII.fx** - Complete HLSL reference implementation
2. **examples/AcerolaFX_Common.fxh** - Utility functions (luminance, depth)
3. **[To create] src/shaders/ascii_compute.wgsl** - Most complex shader
4. **[To create] src/renderer.rs** - Main orchestration logic
5. **[To create] src/config.rs** - Parameter system

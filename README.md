Title says it all

- Barnes Hutt N-Body Simulation (using Octree)
- Modern Bloom Filter, 7 mips
  - Downsample: (x6) 13 sample taps per texel, weighted average
  - Upsample: (x6) 9 taps per texel, 3x3 tent filter
- Siggraph Film Grain Shader
- Made using [wgpu](https://github.com/gfx-rs/wgpu)

### Using wgpu

Did a few projects in Javascript using WebGPU, wanted to compare the experience of using the same API in Rust.

### Todo
WIP

### Shaders / Bindgroups
This is just planning currently, nothing final

| Shader | Bindgroup 0| Bindgroup 1 | Bindgroup 2 | Bindgroup 3 |
| - | - | - | - | - |
| `tree-construction` | Barnes Hutt Octree (rw) | | | |
| `barnes-hutt` | Barnes Hutt Octree (rw) |
| `render` |Barnes Hutt Octree (rw) | Render Settings Uniform| | |
| `bloom-downsample` | Texture (rw) | Bloom Settings Uniform |  | |
| `bloom-upsample` | Texture (rw) | Bloom Settings Uniform | | |
| `film-grain` | Texture (rw) | Render Texture & Grain Settings Uniform | | |



  
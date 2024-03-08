## Barnes - Hutt

Octree on GPU
- https://developer.nvidia.com/gpugems/gpugems2/part-v-image-oriented-computing/chapter-37-octree-textures-gpu
- https://www.cse.iitb.ac.in/~rhushabh/publications/octree
- https://www.antexel.com/sylefeb-research/octreetex/

- Explore storing tree as Buffer vs. Texture

### Neccesary Data
Nodes need
- Center of Mass
- Total Mass
- Indirection Index
- Leaf Boolean (maybe just compare indiretion index to 0 or something)

#### Store in Compute Buffers
- Method 1, more space efficient, more lookups
```rust
struct OctTreeData {
  center_of_mass: vec3f,
  total_mass: f32,
} // Equal to # stars

struct OctTreeNode {
  indirection_index: u32,
  leaf: bool
}

struct OctTree {
  nodes: array<OctTreeNode,80>, // Would need about 8*Star Count worst case
  data: array<OctTreeData,10> // At most 1 data per star
}
```
- Method 2, less space efficient, less lookups
```rust
struct OctTreeNode {
  center_of_mass: vec3<f32>,
  total_mass: f32,
  indirection_index: u32,
  leaf: bool
} // 32 bytes per Node, 1 million stars means 64MB of Memory


struct OctTree {
  nodes: array<OctTreeNode,80>, // Would need about 8*Star Count worst case
}
```

- Octtree size should never really exceed 2*n nodes, but will have to find a way to handlebuffer overflows

- Ideally we want to stay either under 120MB buffers or under 1GB Buffers

- I think overall a good goal is 5 million stars

#### Store in Texture Buffer
Similair to Nvidia Solution


### Construction
In order to achieve construction the GPU, we need to do some fancy prefix-sum stuff OR multiple passes each at different resolution.

### Simulation
https://en.wikipedia.org/wiki/Barnes%E2%80%93Hut_simulation


## Rendering

- 3D Rendering
- Call of Duty Bloom Filter
- Research Paper Film Grain Filter
- Screen Dirt??


use {
    super::Galaxy,
    super::WgpuState,
};

struct SimState {
    wgpu_state: WgpuState,
    galaxy: Galaxy,
}

impl SimState {
    pub fn new(wgpu_state: WgpuState) -> Self {
        let galaxy = Galaxy::new();
        Self {
            wgpu_state,
            galaxy,
        }
    }
}
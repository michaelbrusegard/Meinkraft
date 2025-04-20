#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LOD {
    High,
    Low,
}

impl LOD {
    pub fn scale_factor(&self) -> f32 {
        match self {
            LOD::High => 1.0,
            LOD::Low => 4.0,
        }
    }

    pub fn downsample_factor(&self) -> usize {
        match self {
            LOD::High => 1,
            LOD::Low => 4,
        }
    }
}

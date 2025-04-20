#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LOD {
    LOD1,
    LOD2,
    LOD4,
    LOD8,
}

impl LOD {
    pub fn scale_factor(&self) -> f32 {
        match self {
            LOD::LOD1 => 1.0,
            LOD::LOD2 => 2.0,
            LOD::LOD4 => 4.0,
            LOD::LOD8 => 8.0,
        }
    }

    pub fn downsample_factor(&self) -> usize {
        match self {
            LOD::LOD1 => 1,
            LOD::LOD2 => 2,
            LOD::LOD4 => 4,
            LOD::LOD8 => 8,
        }
    }
}

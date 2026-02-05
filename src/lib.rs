pub mod reader;
pub mod writer;

pub use reader::ReadTsi;
pub use writer::WriteTsi;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Tsi {
    /// Box dimensions in nm.
    pub dimensions: [f32; 3],
    pub vertices: Vec<Vertex>,
    pub triangles: Vec<Triangle>,
    pub inclusions: Vec<Inclusion>,
    pub exclusions: Vec<Exclusion>,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Vertex {
    pub position: [f32; 3],
    pub domain: i32,
}

// In the TS2CG implementation, this is an `int`.
pub type VertexIndex = u32;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash)]
pub struct Triangle {
    pub vertices: [VertexIndex; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Inclusion {
    pub ty: i32,
    pub vertex_index: VertexIndex,
    pub vector: [f32; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Exclusion {
    pub vertex_index: VertexIndex,
    pub radius: f32,
}

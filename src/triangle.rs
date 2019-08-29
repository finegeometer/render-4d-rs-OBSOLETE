/// A triangle in the 3D render.
#[derive(Debug, Copy, Clone)]
pub struct Triangle {
    /// The vertices of the triangle.
    pub vertices: [Vertex; 3],
    /// If this is true, the triangle needs to be subtracted from the picture, rather than added.
    pub negated: bool,
}

/// A vertex of a triangle in the 3D render.
#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    /// The position, in homogeneous coordinates.
    pub position: [f64; 4],
    /// The location in the texture, in homogeneous coordinates.
    pub texcoord: [f64; 3],
}

impl Vertex {
    pub(crate) fn new(point: nalgebra::Vector3<f64>, embedding: nalgebra::Matrix4x3<f64>) -> Self {
        Self {
            position: (embedding * point).into(),
            texcoord: point.into(),
        }
    }
}

use nalgebra as na;

/// The mesh, in 4D space.
pub struct Mesh {
    pub facets: Vec<crate::facet::Facet>,
}

impl Mesh {
    pub fn project(
        &self,
        p: na::Matrix5<f64>,
    ) -> impl Iterator<Item = crate::triangle::Triangle> + '_ {
        crate::facet::Facet::do_all_occlusions(&self.facets, p)
            .flat_map(crate::texture::Texture::get_triangles)
    }
}

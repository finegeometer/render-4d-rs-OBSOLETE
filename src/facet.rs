use nalgebra as na;

/// A 3D facet of the mesh.
pub struct Facet {
    pub embedding: na::Matrix5x4<f64>,
    pub regions: Vec<Vec<na::RowVector4<f64>>>,
    pub texture: Vec<crate::texture::Texture>,
}

impl Facet {
    fn to_screen_depth_space(&self, p: na::Matrix5<f64>) -> Vec<Vec<na::RowVector5<f64>>> {
        let m0: na::Matrix5x4<f64> /* Facet -> Screen w/ Depth */ = p * self.embedding;
        let m1: na::Matrix4x5<f64> /* Screen w/ Depth -> Screen */ = matrix_forget_depth();
        let m2: na::Matrix4<f64> /* Facet -> Screen */ = m1 * m0;
        let m3: na::Matrix4<f64> /* Screen -> Facet */ = match m2.try_inverse() {
            Some(x) => x,
            None => {return Vec::new();}
        };
        let m4: na::Matrix4x5<f64> /* Screen w/ Depth -> Facet */ = m3 * m1;

        let r = match region_behind(m0) {
            Some(x) => x,
            None => {
                return Vec::new();
            }
        };

        self.regions
            .iter()
            .map(|region| {
                region
                    .iter()
                    .map(|h| h * m4)
                    .chain(std::iter::once(r))
                    .collect()
            })
            .collect()
    }

    pub(crate) fn do_all_occlusions<'r>(
        facets: &'r [Self],
        p: na::Matrix5<f64>,
    ) -> impl Iterator<Item = crate::texture::Texture> + 'r {
        let screen_depth_space_regions: Vec<_> =
            facets.iter().map(|f| f.to_screen_depth_space(p)).collect();

        let mut out = Vec::new();
        for i in 0..facets.len() {
            if region_behind(p * facets[i].embedding).is_none() {
                continue;
            }
            for tex in facets[i].texture.iter() {
                out.push(
                    tex.subtract_regions((0..facets.len()).filter(|j| i != *j).flat_map(|j| {
                        screen_depth_space_regions[j]
                            .iter()
                            .map(|region| region.iter().map(|h| h * p * facets[i].embedding))
                    }))
                    .transform(matrix_forget_depth() * p * facets[i].embedding),
                );
            }
        }
        out.into_iter()
    }
}

fn region_behind(embedding: na::Matrix5x4<f64>) -> Option<na::RowVector5<f64>> {
    // This is a cross product.
    let x1 = embedding.remove_row(0).determinant();
    let x2 = -embedding.remove_row(1).determinant();
    let x3 = embedding.remove_row(2).determinant();
    let x4 = -embedding.remove_row(3).determinant();
    let x5 = embedding.remove_row(4).determinant();
    let hyperplane = na::RowVector5::new(x1, x2, x3, x4, x5);

    // Make sure the positive (included in region) direction is in the positive depth direction.
    if x4 < 0. {
        None
    } else {
        Some(hyperplane)
    }
}

fn matrix_forget_depth() -> na::Matrix4x5<f64> {
    na::Matrix4x5::new(
        1., 0., 0., 0., 0., //
        0., 1., 0., 0., 0., //
        0., 0., 1., 0., 0., //
        0., 0., 0., 0., 1.,
    )
}

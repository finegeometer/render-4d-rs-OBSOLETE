use nalgebra as na;

/// A 3D facet of the mesh.
pub struct Facet {
    embedding: na::Matrix5x4<f64>,
    region: Vec<na::RowVector4<f64>>,
    texture: Vec<crate::texture::Texture>,
}

impl Facet {
    fn to_screen_depth_space(&self, p: na::Matrix5<f64>) -> Option<Vec<na::RowVector5<f64>>> {
        let m0: na::Matrix5x4<f64> /* Facet -> Screen w/ Depth */ = p * self.embedding;
        let m1: na::Matrix4x5<f64> /* Screen w/ Depth -> Screen */ =
            na::Matrix4x5::new(
                1., 0., 0., 0.,
                0., 1., 0., 0.,
                0., 0., 1., 0.,
                0., 0., 0., 0.,
                0., 0., 0., 1.);
        let m2: na::Matrix4<f64> /* Facet -> Screen */ = m1 * m0;
        let m3: na::Matrix4<f64> /* Screen -> Facet */ = m2.try_inverse()?;
        let m4: na::Matrix4x5<f64> /* Screen w/ Depth -> Facet */ = m3 * m1;

        let region = self
            .region
            .iter()
            .map(|h| h * m4)
            .chain(std::iter::once(region_behind(m0)));

        // Extraneous allocation because the typechecker won't let me return an iterator for some unknown reason.
        Some(region.collect())
    }

    pub(crate) fn do_all_occlusions<'r>(
        facets: &'r [Self],
        p: na::Matrix5<f64>,
    ) -> impl Iterator<Item = crate::texture::Texture> + 'r {
        facets.iter().enumerate().flat_map(move |(i, f1)| {
            f1.texture.iter().map(move |tex| {
                tex.subtract_regions(
                    facets
                        .iter()
                        .enumerate()
                        .filter(|(j, _)| i != *j)
                        .filter_map(|(_, f2)| {
                            Some(
                                f2.to_screen_depth_space(p)?
                                    .into_iter()
                                    .map(|h| h * p * f1.embedding),
                            )
                        }),
                )
            })
        })
    }
}

fn region_behind(embedding: na::Matrix5x4<f64>) -> na::RowVector5<f64> {
    // This is a cross product.
    let x1 = embedding.remove_row(0).determinant();
    let x2 = -embedding.remove_row(1).determinant();
    let x3 = embedding.remove_row(2).determinant();
    let x4 = -embedding.remove_row(3).determinant();
    let x5 = embedding.remove_row(4).determinant();
    let hyperplane = na::RowVector5::new(x1, x2, x3, x4, x5);

    // Make sure the positive (included in region) direction is in the positive depth direction.
    if x4 < 0. {
        -hyperplane
    } else {
        hyperplane
    }
}

impl Facet {
    /// Create a cubical facet.
    pub fn new_cube(embedding: na::Matrix5x4<f64>) -> Self {
        Self {
            embedding,
            region: vec![
                na::RowVector4::new(1., 0., 0., 0.),
                na::RowVector4::new(0., 1., 0., 0.),
                na::RowVector4::new(0., 0., 1., 0.),
                na::RowVector4::new(-1., 0., 0., 1.),
                na::RowVector4::new(0., -1., 0., 1.),
                na::RowVector4::new(0., 0., -1., 1.),
            ],
            texture: vec![
                crate::texture::Texture::new_square(
                    [0.05, 0.05, 0.05],
                    [0.95, 0.05, 0.05],
                    [0.05, 0.95, 0.05],
                ),
                crate::texture::Texture::new_square(
                    [0.05, 0.05, 0.05],
                    [0.05, 0.95, 0.05],
                    [0.05, 0.05, 0.95],
                ),
                crate::texture::Texture::new_square(
                    [0.05, 0.05, 0.05],
                    [0.05, 0.05, 0.95],
                    [0.95, 0.05, 0.05],
                ),
                crate::texture::Texture::new_square(
                    [0.95, 0.95, 0.95],
                    [0.05, 0.95, 0.95],
                    [0.95, 0.05, 0.95],
                ),
                crate::texture::Texture::new_square(
                    [0.95, 0.95, 0.95],
                    [0.95, 0.05, 0.95],
                    [0.95, 0.95, 0.05],
                ),
                crate::texture::Texture::new_square(
                    [0.95, 0.95, 0.95],
                    [0.95, 0.95, 0.05],
                    [0.05, 0.95, 0.95],
                ),
            ],
        }
    }
}

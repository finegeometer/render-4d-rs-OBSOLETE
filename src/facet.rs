use nalgebra as na;

/// A 3D facet of the mesh.
pub struct Facet {
    embedding: na::Matrix5x4<f64>,
    region: Vec<na::RowVector4<f64>>,
    texture: Vec<crate::texture::Texture>,
    convex_hull: Vec<na::Vector4<f64>>,
}

impl Facet {
    fn to_screen_depth_space(&self, p: na::Matrix5<f64>) -> Option<Vec<na::RowVector5<f64>>> {
        let m0: na::Matrix5x4<f64> /* Facet -> Screen w/ Depth */ = p * self.embedding;
        let m1: na::Matrix4x5<f64> /* Screen w/ Depth -> Screen */ = matrix_forget_depth();
        let m2: na::Matrix4<f64> /* Facet -> Screen */ = m1 * m0;
        let m3: na::Matrix4<f64> /* Screen -> Facet */ = m2.try_inverse()?;
        let m4: na::Matrix4x5<f64> /* Screen w/ Depth -> Facet */ = m3 * m1;

        let region = self
            .region
            .iter()
            .map(|h| h * m4)
            .chain(std::iter::once(region_behind(m0)?));

        // Extraneous allocation because the typechecker won't let me return an iterator for some unknown reason.
        Some(region.collect())
    }

    pub(crate) fn do_all_occlusions<'r>(
        facets: &'r [Self],
        p: na::Matrix5<f64>,
    ) -> impl Iterator<Item = crate::texture::Texture> + 'r {
        let aabbs: std::rc::Rc<[[[f64; 3]; 2]]> = facets
            .iter()
            .map(
                |Facet {
                     embedding,
                     convex_hull,
                     ..
                 }| {
                    aabb(
                        convex_hull
                            .iter()
                            .map(|pt| matrix_forget_depth() * p * embedding * pt),
                    )
                },
            )
            .collect::<Vec<_>>()
            .into();

        (0..facets.len())
            .filter(move |&i| region_behind(p * facets[i].embedding).is_some())
            .flat_map(move |i| {
                let aabbs = aabbs.clone();
                facets[i].texture.iter().map(move |tex| {
                    tex.subtract_regions((0..facets.len()).filter_map(|j| {
                        let [min1, max1] = aabbs[i];
                        let [min2, max2] = aabbs[j];

                        if i == j
                            || min1[0] > max2[0]
                            || min1[1] > max2[1]
                            || min1[2] > max2[2]
                            || min2[0] > max1[0]
                            || min2[1] > max1[1]
                            || min2[2] > max1[2]
                        {
                            return None;
                        }

                        Some(
                            facets[j]
                                .to_screen_depth_space(p)?
                                .into_iter()
                                .map(|h| h * p * facets[i].embedding),
                        )
                    }))
                    .transform(matrix_forget_depth() * p * facets[i].embedding)
                })
            })
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
            convex_hull: vec![
                na::Vector4::new(0., 0., 0., 1.),
                na::Vector4::new(0., 0., 1., 1.),
                na::Vector4::new(0., 1., 0., 1.),
                na::Vector4::new(0., 1., 1., 1.),
                na::Vector4::new(1., 0., 0., 1.),
                na::Vector4::new(1., 0., 1., 1.),
                na::Vector4::new(1., 1., 0., 1.),
                na::Vector4::new(1., 1., 1., 1.),
            ],
        }
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

fn aabb(hull: impl IntoIterator<Item = na::Vector4<f64>>) -> [[f64; 3]; 2] {
    let mut max_x = std::f64::MIN;
    let mut max_y = std::f64::MIN;
    let mut max_z = std::f64::MIN;
    let mut min_x = std::f64::MAX;
    let mut min_y = std::f64::MAX;
    let mut min_z = std::f64::MAX;

    for pt in hull {
        let [mut x, mut y, mut z, w]: [f64; 4] = pt.into();
        x /= w;
        y /= w;
        z /= w;

        max_x = max_x.max(x);
        max_y = max_y.max(y);
        max_z = max_z.max(z);
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        min_z = min_z.min(z);
    }

    [[min_x, min_y, min_z], [max_x, max_y, max_z]]
}

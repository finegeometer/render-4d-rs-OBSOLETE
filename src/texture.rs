use nalgebra as na;
use std::convert::TryInto;

pub struct Texture {
    pub embedding: na::Matrix4x3<f64>,
    pub poly: polygon3::Polygon,
}

impl Texture {
    pub(crate) fn get_triangles(self) -> impl Iterator<Item = crate::triangle::Triangle> {
        let embedding: na::Matrix4x3<f64> = self.embedding;
        self.poly
            .vertices()
            .into_iter()
            .flat_map(move |mut polygon| {
                if area(&polygon) < 0. {
                    polygon.reverse();
                }
                let v1 = polygon[0];
                polygon[1..]
                    .windows(2)
                    .map(move |w| {
                        let v2 = w[0];
                        let v3 = w[1];

                        crate::triangle::Triangle {
                            // WRONG in case of polygons with holes
                            negated: area(&[v1, v2, v3]) < 0.,
                            vertices: [
                                crate::triangle::Vertex::new(v1.to_f64_array().into(), embedding),
                                crate::triangle::Vertex::new(v2.to_f64_array().into(), embedding),
                                crate::triangle::Vertex::new(v3.to_f64_array().into(), embedding),
                            ],
                        }
                    })
                    .collect::<Vec<_>>()
                    .into_iter()
            })
    }

    pub(crate) fn subtract_regions(
        &self,
        regions: impl IntoIterator<Item = impl IntoIterator<Item = na::RowVector4<f64>>>,
    ) -> Self {
        Self {
            embedding: self.embedding,
            poly: self.poly.difference(
                &regions
                    .into_iter()
                    .filter_map(|r| region_to_polygon(r, self.embedding))
                    .collect::<Vec<_>>(),
            ),
        }
    }

    pub(crate) fn transform(mut self, mat: na::Matrix4<f64>) -> Self {
        self.embedding = mat * self.embedding;
        self
    }
}

fn region_to_polygon(
    region: impl IntoIterator<Item = na::RowVector4<f64>>,
    embedding: na::Matrix4x3<f64>,
) -> Option<polygon3::Polygon> {
    let boundaries = region
        .into_iter()
        .filter_map(|h| polygon3::Line::try_from_f64_array((h * embedding).into()))
        .chain(giant_square());

    let convex_polygon = polygon3::ConvexPolygon::from_boundaries(boundaries)?;

    Some(convex_polygon.try_into().unwrap())
}

fn giant_square() -> Vec<polygon3::Line> {
    vec![
        [1, 0, 1000].try_into().unwrap(),
        [0, 1, 1000].try_into().unwrap(),
        [-1, 0, 1000].try_into().unwrap(),
        [0, -1, 1000].try_into().unwrap(),
    ]
}

fn area(p: &[polygon3::Point]) -> f64 {
    let n = p.len();
    let mut out = 0.;
    for i in 0..n {
        let j = (i + 1) % n;

        let [mut x1, mut y1, z1] = p[i].to_f64_array();
        x1 /= z1;
        y1 /= z1;
        let [mut x2, mut y2, z2] = p[j].to_f64_array();
        x2 /= z2;
        y2 /= z2;

        out += x1 * y2 - y1 * x2;
    }
    out
}

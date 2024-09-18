use std::collections::HashSet;

use crate::mesh::{Dynamic, Mesh, MeshProvider, PDVertex, PNVertex, Static};

#[derive(Clone, Copy)]
pub struct Box;

#[derive(Clone, Copy)]
pub struct Ellipsoid;

#[derive(Clone, Copy)]
pub struct BoxLines;

impl MeshProvider for Box {
    type Vertex = PNVertex;
    type Kind = Static;

    fn create_mesh(self) -> Mesh<Self::Vertex> {
        #[rustfmt::skip]
        let vertices = vec![
           PNVertex { position: [ 0.5,  0.5,  0.5], normal: [ 0.0,  0.0,  1.0] },
           PNVertex { position: [-0.5,  0.5,  0.5], normal: [ 0.0,  0.0,  1.0] },
           PNVertex { position: [ 0.5, -0.5,  0.5], normal: [ 0.0,  0.0,  1.0] },
           PNVertex { position: [-0.5, -0.5,  0.5], normal: [ 0.0,  0.0,  1.0] },

           PNVertex { position: [-0.5,  0.5, -0.5], normal: [ 0.0,  0.0, -1.0] },
           PNVertex { position: [ 0.5,  0.5, -0.5], normal: [ 0.0,  0.0, -1.0] },
           PNVertex { position: [-0.5, -0.5, -0.5], normal: [ 0.0,  0.0, -1.0] },
           PNVertex { position: [ 0.5, -0.5, -0.5], normal: [ 0.0,  0.0, -1.0] },

           PNVertex { position: [ 0.5,  0.5,  0.5], normal: [ 1.0,  0.0,  0.0] },
           PNVertex { position: [ 0.5, -0.5,  0.5], normal: [ 1.0,  0.0,  0.0] },
           PNVertex { position: [ 0.5,  0.5, -0.5], normal: [ 1.0,  0.0,  0.0] },
           PNVertex { position: [ 0.5, -0.5, -0.5], normal: [ 1.0,  0.0,  0.0] },

           PNVertex { position: [-0.5, -0.5,  0.5], normal: [-1.0,  0.0,  0.0] },
           PNVertex { position: [-0.5,  0.5,  0.5], normal: [-1.0,  0.0,  0.0] },
           PNVertex { position: [-0.5, -0.5, -0.5], normal: [-1.0,  0.0,  0.0] },
           PNVertex { position: [-0.5,  0.5, -0.5], normal: [-1.0,  0.0,  0.0] },

           PNVertex { position: [ 0.5,  0.5,  0.5], normal: [ 0.0,  1.0,  0.0] },
           PNVertex { position: [ 0.5,  0.5, -0.5], normal: [ 0.0,  1.0,  0.0] },
           PNVertex { position: [-0.5,  0.5,  0.5], normal: [ 0.0,  1.0,  0.0] },
           PNVertex { position: [-0.5,  0.5, -0.5], normal: [ 0.0,  1.0,  0.0] },

           PNVertex { position: [ 0.5, -0.5, -0.5], normal: [ 0.0, -1.0,  0.0] },
           PNVertex { position: [ 0.5, -0.5,  0.5], normal: [ 0.0, -1.0,  0.0] },
           PNVertex { position: [-0.5, -0.5, -0.5], normal: [ 0.0, -1.0,  0.0] },
           PNVertex { position: [-0.5, -0.5,  0.5], normal: [ 0.0, -1.0,  0.0] },
        ];
        #[rustfmt::skip]
        let indices = vec![
             0,  1,  2,  2,  1,  3,
             4,  5,  6,  6,  5,  7,
             8,  9, 10, 10,  9, 11,
            12, 13, 14, 14, 13, 15,
            16, 17, 18, 18, 17, 19,
            20, 21, 22, 22, 21, 23,
        ];
        Mesh { vertices, indices }
    }
}

impl MeshProvider for Ellipsoid {
    type Vertex = PNVertex;
    type Kind = Static;

    fn create_mesh(self) -> Mesh<Self::Vertex> {
        let Polyhedron { vertices, faces } = icosphere(3);
        let vertices = vertices
            .into_iter()
            .map(|p| PNVertex {
                position: p,
                normal: p,
            })
            .collect();
        let indices = faces
            .into_iter()
            .flat_map(|f| f.map(|i| i as u32))
            .collect();
        Mesh { vertices, indices }
    }
}

impl MeshProvider for BoxLines {
    type Vertex = PDVertex;
    type Kind = Static;

    fn create_mesh(self) -> Mesh<Self::Vertex> {
        #[rustfmt::skip]
        let vertices = vec![
            PDVertex { position: [ 0.5,  0.5,  0.5], direction: [1.0, 0.0, 0.0] },
            PDVertex { position: [-0.5,  0.5,  0.5], direction: [1.0, 0.0, 0.0] },

            PDVertex { position: [ 0.5,  0.5, -0.5], direction: [1.0, 0.0, 0.0] },
            PDVertex { position: [-0.5,  0.5, -0.5], direction: [1.0, 0.0, 0.0] },

            PDVertex { position: [ 0.5, -0.5,  0.5], direction: [1.0, 0.0, 0.0] },
            PDVertex { position: [-0.5, -0.5,  0.5], direction: [1.0, 0.0, 0.0] },

            PDVertex { position: [ 0.5, -0.5, -0.5], direction: [1.0, 0.0, 0.0] },
            PDVertex { position: [-0.5, -0.5, -0.5], direction: [1.0, 0.0, 0.0] },

            PDVertex { position: [ 0.5,  0.5,  0.5], direction: [0.0, 1.0, 0.0] },
            PDVertex { position: [ 0.5, -0.5,  0.5], direction: [0.0, 1.0, 0.0] },

            PDVertex { position: [ 0.5,  0.5, -0.5], direction: [0.0, 1.0, 0.0] },
            PDVertex { position: [ 0.5, -0.5, -0.5], direction: [0.0, 1.0, 0.0] },

            PDVertex { position: [-0.5,  0.5,  0.5], direction: [0.0, 1.0, 0.0] },
            PDVertex { position: [-0.5, -0.5,  0.5], direction: [0.0, 1.0, 0.0] },

            PDVertex { position: [-0.5,  0.5, -0.5], direction: [0.0, 1.0, 0.0] },
            PDVertex { position: [-0.5, -0.5, -0.5], direction: [0.0, 1.0, 0.0] },

            PDVertex { position: [ 0.5,  0.5,  0.5], direction: [0.0, 0.0, 1.0] },
            PDVertex { position: [ 0.5,  0.5, -0.5], direction: [0.0, 0.0, 1.0] },

            PDVertex { position: [ 0.5, -0.5,  0.5], direction: [0.0, 0.0, 1.0] },
            PDVertex { position: [ 0.5, -0.5, -0.5], direction: [0.0, 0.0, 1.0] },

            PDVertex { position: [-0.5,  0.5,  0.5], direction: [0.0, 0.0, 1.0] },
            PDVertex { position: [-0.5,  0.5, -0.5], direction: [0.0, 0.0, 1.0] },

            PDVertex { position: [-0.5, -0.5,  0.5], direction: [0.0, 0.0, 1.0] },
            PDVertex { position: [-0.5, -0.5, -0.5], direction: [0.0, 0.0, 1.0] },
        ];
        #[rustfmt::skip]
        let indices = vec![
             0,  1,  2,  3,  4,  5,  6,  7,
             8,  9, 10, 11, 12, 13, 14, 15,
            16, 17, 18, 19, 20, 21, 22, 23,
        ];
        Mesh { vertices, indices }
    }
}

struct Polyhedron {
    vertices: Vec<[f32; 3]>,
    faces: Vec<[usize; 3]>,
}

fn half_point(v1: &[f32; 3], v2: &[f32; 3]) -> [f32; 3] {
    [
        (v1[0] + v2[0]) / 2.0,
        (v1[1] + v2[1]) / 2.0,
        (v1[2] + v2[2]) / 2.0,
    ]
}

impl Polyhedron {
    fn normalize(&mut self) {
        for [x, y, z] in &mut self.vertices {
            let len = (x.powi(2) + y.powi(2) + z.powi(2)).sqrt();
            *x /= 2.0 * len;
            *y /= 2.0 * len;
            *z /= 2.0 * len;
        }
    }

    fn subdivide(&mut self) {
        // collect edges
        let edges: Vec<_> = self
            .faces
            .iter()
            .flat_map(|&[a, b, c]| [[a, b], [b, c], [c, a]])
            .map(|[a, b]| [a.min(b), b.max(a)])
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        // panic!("{:?}", edges.len());

        // split edges into oldedge -> newedges maps (this creates the new vertices)
        // let mut newedges = vec![];
        // edge -> newvertex
        let mut edge_map = vec![];
        for [a, b] in &edges {
            let n = self.vertices.len();
            self.vertices
                .push(half_point(&self.vertices[*a], &self.vertices[*b]));
            // let k = newedges.len();
            // newedges.push([*a, n]);
            // newedges.push([n, *b]);
            edge_map.push(n);
        }
        // face is defined as a -> b -> c (vertices as ccw)
        // edges are min(a, b) -> max(a, b) ...
        // face with new vertices is a -> x -> b -> y -> c -> z
        // new faces are a -> x -> z, b -> y -> x, c -> z -> y, x -> y -> z
        let oldfaces = std::mem::take(&mut self.faces);
        for [a, b, c] in oldfaces {
            // x == find a -> b edge
            let x = edges
                .iter()
                .enumerate()
                .find(|(_, &[i, j])| (i == a && j == b) || (i == b && j == a))
                .map(|(e, _)| edge_map[e])
                .unwrap();
            let y = edges
                .iter()
                .enumerate()
                .find(|(_, &[i, j])| (i == b && j == c) || (i == c && j == b))
                .map(|(e, _)| edge_map[e])
                .unwrap();
            let z = edges
                .iter()
                .enumerate()
                .find(|(_, &[i, j])| (i == c && j == a) || (i == a && j == c))
                .map(|(e, _)| edge_map[e])
                .unwrap();
            self.faces.push([a, x, z]);
            self.faces.push([b, y, x]);
            self.faces.push([c, z, y]);
            self.faces.push([x, y, z]);
        }
    }
}

fn icosphere(subdivisions: usize) -> Polyhedron {
    let phi = (1.0 + 5.0_f32.sqrt()) / 2.0;
    let a = 1.0;
    let b = 1.0 / phi;
    let vertices = vec![
        [0.0, b, -a],
        [b, a, 0.0],
        [-b, a, 0.0],
        [0.0, b, a],
        [0.0, -b, a],
        [-a, 0.0, b],
        [0.0, -b, -a],
        [a, 0.0, -b],
        [a, 0.0, b],
        [-a, 0.0, -b],
        [b, -a, 0.0],
        [-b, -a, 0.0],
    ];
    let faces = vec![
        [2, 1, 0],
        [1, 2, 3],
        [5, 4, 3],
        [4, 8, 3],
        [7, 6, 0],
        [6, 9, 0],
        [11, 10, 4],
        [10, 11, 6],
        [9, 5, 2],
        [5, 9, 11],
        [8, 7, 1],
        [7, 8, 10],
        [2, 5, 3],
        [8, 1, 3],
        [9, 2, 0],
        [1, 7, 0],
        [11, 9, 6],
        [7, 10, 6],
        [5, 11, 4],
        [10, 8, 4],
    ];
    let mut icosphere = Polyhedron { vertices, faces };
    icosphere.normalize();
    for _ in 0..subdivisions {
        icosphere.subdivide();
        icosphere.normalize();
    }
    icosphere
}

// TODO: add automatic derivation
#[derive(Clone, Copy)]
pub struct ParametricSquare<GenFn: Fn(f32, f32) -> (f32, f32, f32)> {
    steps: usize,
    generator: GenFn,
}

impl<GenFn: Fn(f32, f32) -> (f32, f32, f32)> ParametricSquare<GenFn> {
    pub fn new(steps: usize, generator: GenFn) -> Self {
        Self { steps, generator }
    }
}

impl<GenFn: Fn(f32, f32) -> (f32, f32, f32) + Copy> MeshProvider
    for ParametricSquare<GenFn>
{
    type Vertex = PNVertex;
    type Kind = Dynamic;

    fn create_mesh(self) -> Mesh<Self::Vertex> {
        let mut vertices = Vec::with_capacity(self.steps * self.steps);
        let gen_steps = self.steps - 1;
        for i in 0..=gen_steps {
            for j in 0..=gen_steps {
                let x = i as f32 / gen_steps as f32 - 0.5;
                let z = j as f32 / gen_steps as f32 - 0.5;
                let (y, x_grad, z_grad) = (self.generator)(x, z);
                let x_angle = x_grad.atan();
                let z_angle = z_grad.atan();
                let x_vec = [x_angle.cos(), x_angle.sin(), 0.0];
                let z_vec = [0.0, z_angle.sin(), z_angle.cos()];
                // normal = z_vec cross x_vec
                let normal = [
                    z_vec[1] * x_vec[2] - z_vec[2] * x_vec[1],
                    z_vec[2] * x_vec[0] - z_vec[0] * x_vec[2],
                    z_vec[0] * x_vec[1] - z_vec[1] * x_vec[0],
                ];
                // let normal_len = normal[0].hypot(normal[1]).hypot(normal[2]);
                // let normal = normal.map(|n| n / normal_len);
                vertices.push(PNVertex {
                    position: [x, y, z],
                    normal,
                });
            }
        }
        let mut indices = Vec::with_capacity(gen_steps * gen_steps * 6);
        for i in 0..gen_steps as u32 {
            #[allow(clippy::identity_op)]
            for j in 0..gen_steps as u32 {
                indices.push(self.steps as u32 * (j + 0) + i + 0);
                indices.push(self.steps as u32 * (j + 0) + i + 1);
                indices.push(self.steps as u32 * (j + 1) + i + 0);

                indices.push(self.steps as u32 * (j + 1) + i + 0);
                indices.push(self.steps as u32 * (j + 0) + i + 1);
                indices.push(self.steps as u32 * (j + 1) + i + 1);
            }
        }
        Mesh { vertices, indices }
    }
}

#[derive(Clone, Copy)]
pub struct LowPoly<Provider: MeshProvider<Vertex = PNVertex, Kind = Dynamic>>(
    pub Provider,
);

impl<Provider: MeshProvider<Vertex = PNVertex, Kind = Dynamic>> MeshProvider
    for LowPoly<Provider>
{
    type Vertex = PNVertex;

    type Kind = Dynamic;

    fn create_mesh(self) -> Mesh<Self::Vertex> {
        low_poly_triangles(self.0.create_mesh())
    }
}

#[derive(Clone, Copy)]
pub struct StaticLowPoly<
    Provider: MeshProvider<Vertex = PNVertex, Kind = Static> + 'static,
>(pub Provider);

impl<Provider: MeshProvider<Vertex = PNVertex, Kind = Static> + 'static>
    MeshProvider for StaticLowPoly<Provider>
{
    type Vertex = PNVertex;

    type Kind = Static;

    fn create_mesh(self) -> Mesh<Self::Vertex> {
        low_poly_triangles(self.0.create_mesh())
    }
}

fn low_poly_triangles(mesh: Mesh<PNVertex>) -> Mesh<PNVertex> {
    let mut vertices = Vec::with_capacity(mesh.indices.len());
    let indices = (0..mesh.indices.len() as u32).collect();
    for i in mesh
        .indices
        .chunks(3)
        .map(|c| std::convert::TryInto::<[_; 3]>::try_into(c).unwrap())
    {
        let v = i.map(|i| mesh.vertices[i as usize]);
        let a = [0, 1, 2].map(|i| v[1].position[i] - v[0].position[i]);
        let b = [0, 1, 2].map(|i| v[2].position[i] - v[0].position[i]);
        let normal = [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ];
        let normal_len = normal[0].hypot(normal[1]).hypot(normal[2]);
        let normal = normal.map(|n| n / normal_len);
        vertices.extend_from_slice(&v.map(|v| PNVertex {
            position: v.position,
            normal,
        }));
    }
    Mesh { vertices, indices }
}

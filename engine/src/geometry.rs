use std::collections::HashSet;

use crate::mesh::{Mesh, MeshProvider, Vertex};

#[derive(Clone, Copy)]
pub struct Box;

#[derive(Clone, Copy)]
pub struct Ellipsoid;

impl MeshProvider for Box {
    fn create_mesh() -> Mesh {
        #[rustfmt::skip]
        let vertices = vec![
           Vertex { position: [ 0.5,  0.5,  0.5], normal: [ 0.0,  0.0,  1.0,] },
           Vertex { position: [-0.5,  0.5,  0.5], normal: [ 0.0,  0.0,  1.0,] },
           Vertex { position: [ 0.5, -0.5,  0.5], normal: [ 0.0,  0.0,  1.0,] },
           Vertex { position: [-0.5, -0.5,  0.5], normal: [ 0.0,  0.0,  1.0,] },

           Vertex { position: [-0.5,  0.5, -0.5], normal: [ 0.0,  0.0, -1.0,] },
           Vertex { position: [ 0.5,  0.5, -0.5], normal: [ 0.0,  0.0, -1.0,] },
           Vertex { position: [-0.5, -0.5, -0.5], normal: [ 0.0,  0.0, -1.0,] },
           Vertex { position: [ 0.5, -0.5, -0.5], normal: [ 0.0,  0.0, -1.0,] },

           Vertex { position: [ 0.5,  0.5,  0.5], normal: [ 1.0,  0.0,  0.0,] },
           Vertex { position: [ 0.5, -0.5,  0.5], normal: [ 1.0,  0.0,  0.0,] },
           Vertex { position: [ 0.5,  0.5, -0.5], normal: [ 1.0,  0.0,  0.0,] },
           Vertex { position: [ 0.5, -0.5, -0.5], normal: [ 1.0,  0.0,  0.0,] },

           Vertex { position: [-0.5, -0.5,  0.5], normal: [-1.0,  0.0,  0.0,] },
           Vertex { position: [-0.5,  0.5,  0.5], normal: [-1.0,  0.0,  0.0,] },
           Vertex { position: [-0.5, -0.5, -0.5], normal: [-1.0,  0.0,  0.0,] },
           Vertex { position: [-0.5,  0.5, -0.5], normal: [-1.0,  0.0,  0.0,] },

           Vertex { position: [ 0.5,  0.5,  0.5], normal: [ 0.0,  1.0,  0.0,] },
           Vertex { position: [ 0.5,  0.5, -0.5], normal: [ 0.0,  1.0,  0.0,] },
           Vertex { position: [-0.5,  0.5,  0.5], normal: [ 0.0,  1.0,  0.0,] },
           Vertex { position: [-0.5,  0.5, -0.5], normal: [ 0.0,  1.0,  0.0,] },

           Vertex { position: [ 0.5, -0.5, -0.5], normal: [ 0.0, -1.0,  0.0,] },
           Vertex { position: [ 0.5, -0.5,  0.5], normal: [ 0.0, -1.0,  0.0,] },
           Vertex { position: [-0.5, -0.5, -0.5], normal: [ 0.0, -1.0,  0.0,] },
           Vertex { position: [-0.5, -0.5,  0.5], normal: [ 0.0, -1.0,  0.0,] },
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
    fn create_mesh() -> Mesh {
        let Polyhedron { vertices, faces } = icosphere(3);
        let vertices = vertices
            .into_iter()
            .map(|p| Vertex {
                position: p,
                normal: p,
            })
            .collect();
        let indices = faces
            .into_iter()
            .flat_map(|f| f.map(|i| i as u16))
            .collect();
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
            *x /= len;
            *y /= len;
            *z /= len;
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

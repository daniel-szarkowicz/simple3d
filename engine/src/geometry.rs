use crate::mesh::{Mesh, MeshProvider};

#[derive(Clone, Copy)]
pub struct Box;

#[derive(Clone, Copy)]
pub struct Ellipsoid;

impl MeshProvider for Box {
    fn create_mesh() -> Mesh {
        Mesh {
            name: "Box".to_owned(),
        }
    }
}

impl MeshProvider for Ellipsoid {
    fn create_mesh() -> Mesh {
        Mesh {
            name: "Ellipsoid".to_owned(),
        }
    }
}

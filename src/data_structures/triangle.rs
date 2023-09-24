use crate::data_structures::vector::Vector;
#[derive(Debug,Clone,Copy)]
pub struct Triangle {
    vertices: [Vector; 3],
}

impl Triangle {
    pub fn new(vertex0: Vector, vertex1: Vector, vertex2: Vector) -> Self {
        Triangle {
            vertices: [vertex0, vertex1, vertex2],
        }
    }

    pub fn p(&self, index: usize) -> Vector {
        self.vertices[index]
    }
}

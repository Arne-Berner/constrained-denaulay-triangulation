use crate::data_structures::vec2::Vec2;
pub struct Triangle {
    vertices: [Vec2; 3],
}

impl Triangle {
    pub fn new(vertex0: Vec2, vertex1: Vec2, vertex2: Vec2) -> Self {
        Triangle {
            vertices: [vertex0, vertex1, vertex2],
        }
    }

    pub fn p(&self, index: usize) -> Vec2 {
        self.vertices[index]
    }
}
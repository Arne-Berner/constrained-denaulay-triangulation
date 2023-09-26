#[derive(Debug, PartialEq)]
pub struct Edge {
    edge_vertex_a: usize,
    edge_vertex_b: usize,
}

impl Edge {
    pub fn new(edge_vertex_a: usize, edge_vertex_b: usize) -> Self {
        Edge {
            edge_vertex_a,
            edge_vertex_b,
        }
    }

    pub fn vertex_a(&self) -> usize {
        self.edge_vertex_a
    }

    pub fn vertex_b(&self) -> usize {
        self.edge_vertex_b
    }
}

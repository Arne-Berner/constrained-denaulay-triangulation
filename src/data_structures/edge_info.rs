pub struct EdgeInfo {
    pub triangle_index: usize,
    pub edge_index: usize,
    edge_vertex_a: usize,
    edge_vertex_b: usize,
}

impl EdgeInfo {
    pub fn new(
        triangle_index: usize,
        edge_index: usize,
        edge_vertex_a: usize,
        edge_vertex_b: usize,
    ) -> Self {
        EdgeInfo {
            triangle_index,
            edge_index,
            edge_vertex_a,
            edge_vertex_b,
        }
    }

    pub fn a(&self) -> usize {
        self.edge_vertex_a
    }

    pub fn b(&self) -> usize {
        self.edge_vertex_b
    }
    pub fn edge(&self) -> (usize, usize){
        (self.edge_vertex_a, self.edge_vertex_b)
    }
}
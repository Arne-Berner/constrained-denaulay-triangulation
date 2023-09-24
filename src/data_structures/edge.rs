pub struct Edge {
    edge_start: Vector,
    edge_end: Vector,
}

impl Edge {
    pub fn new(edge_start: Vector, edge_end: Vector) -> Self {
        Edge {
            edge_start,
            edge_end,
        }
    }
}

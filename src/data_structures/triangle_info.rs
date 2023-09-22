pub struct TriangleInfo {
    pub vertex_indices: [usize; 3],
    pub adjacent_triangle_indices: [Option<usize>; 3],
}
impl TriangleInfo {
    pub fn new(index_vertices: [usize; 3]) -> Self {
        TriangleInfo {
            vertex_indices: index_vertices,
            adjacent_triangle_indices: [None, None, None],
        }
    }

    pub fn with_adjacent(
        mut self,
        adjacent0: Option<usize>,
        adjacent1: Option<usize>,
        adjacent2: Option<usize>,
    ) -> TriangleInfo {
        self.adjacent_triangle_indices[0] = adjacent0;
        self.adjacent_triangle_indices[1] = adjacent1;
        self.adjacent_triangle_indices[2] = adjacent2;
        self
    }
}
use crate::{math_utils::is_point_to_the_right_of_edge, triangulation::triangulate_point};

use super::{
    edge_info::EdgeInfo, error::CustomError, found_or_added::FoundOrAdded, triangle::Triangle,
    triangle_info::TriangleInfo, vector::Vector,
};

#[derive(Debug)]
pub struct TriangleSet {
    pub points: Vec<Vector>,
    pub triangle_infos: Vec<TriangleInfo>,
}

impl TriangleSet {
    /// Constructor that receives the expected number of triangles to store. It will reserve memory accordingly.
    pub fn new(expected_triangles: usize) -> Self {
        TriangleSet {
            points: Vec::with_capacity(expected_triangles),
            triangle_infos: Vec::with_capacity(expected_triangles * 3),
        }
    }

    pub fn add_point(&mut self, point_to_add: Vector) -> FoundOrAdded {
        for (idx, point) in self.points.iter().enumerate() {
            if *point == point_to_add {
                return FoundOrAdded::Found(idx);
            }
        }
        self.points.push(point_to_add);
        FoundOrAdded::Added(self.points.len() - 1)
    }

    pub fn add_triangle(&mut self, triangle: &Triangle) {
        let p0 = self.add_point(triangle.p(0)).value();
        let p1 = self.add_point(triangle.p(1)).value();
        let p2 = self.add_point(triangle.p(2)).value();
        self.triangle_infos.push(TriangleInfo::new([p0, p1, p2]));
    }

    pub fn add_triangle_info(&mut self, triangle_info_to_add: TriangleInfo) -> usize {
        self.triangle_infos.push(triangle_info_to_add);
        self.triangle_infos.len() - 1
    }

    pub fn triangle_count(&self) -> usize {
        self.triangle_infos.len()
    }

    pub fn get_triangle(&self, index: usize) -> Triangle {
        let p0 = self.points[self.triangle_infos[index].vertex_indices[0]];
        let p1 = self.points[self.triangle_infos[index].vertex_indices[1]];
        let p2 = self.points[self.triangle_infos[index].vertex_indices[2]];
        Triangle::new(p0, p1, p2)
    }

    pub fn get_triangle_info(&self, index: usize) -> TriangleInfo {
        self.triangle_infos[index]
    }

    pub fn get_point(&self, index: usize)->Vector{
        self.points[index]
    }

    pub fn get_point_from_index(&self, triangle_index: usize, vertex_index: usize) -> Vector {
        self.points[self.triangle_infos[triangle_index].vertex_indices[vertex_index]]
    }

    pub fn get_adjacent_triangle_index(
        &self,
        triangle_index: usize,
        vertex_index: usize,
    ) -> Option<usize> {
        self.triangle_infos[triangle_index].adjacent_triangle_indices[vertex_index]
    }

    pub fn find_triangle_that_contains_point(
        &self,
        point: Vector,
        start_triangle: usize,
    ) -> Result<usize, CustomError> {
        let mut is_triangle_found = false;
        let mut triangle_index = start_triangle;
        let mut checked_triangles = 0;

        while !is_triangle_found && checked_triangles < self.triangle_count() {
            checked_triangles += 1;
            is_triangle_found = true;
            for vertex_index in 0..3 {
                // if it is outside of the triangle
                if is_point_to_the_right_of_edge(
                    self.get_point_from_index(triangle_index, vertex_index),
                    self.get_point_from_index(triangle_index, (vertex_index + 1) % 3),
                    point,
                ) {
                    // The point is in the exterior of the triangle (vertices are sorted CCW, the right side is always the exterior from the perspective of the A->B edge)
                    // This "path finding" can not form a circle, because it will only be on the right side for max 2 edges
                    is_triangle_found = false;
                    if let Some(index) =
                        self.get_adjacent_triangle_index(triangle_index, vertex_index)
                    {
                        triangle_index = index;
                        break;
                    }
                }
            }
        }

        if checked_triangles >= self.triangle_count() && self.triangle_count() > 1 {
            // NOT CCW?!
            //println!("\n\n triangle_info:{:#?}\n\npoint1:{:#?}, point2:{:#?}, point3:{:#?}, point_to_insert{:#?}", self.triangle_infos[triangle_index], self.get_point_from_index(triangle_index, 0), self.get_point_from_index(triangle_index, 1), self.get_point_from_index(triangle_index, 2), point);
            //println!("\n\n triangle_info:{:#?}\n\nindex:{:?}\n\npoints:{:?}\n\nchecked_triangles{:?}", self.triangle_infos[triangle_index], triangle_index, self.points, checked_triangles);
            //println!("Unable to find a triangle that contains the point ({:?}), starting at triangle {}. Are you generating very small triangles?", point, start_triangle);
            return Err(CustomError::PointNotInTriangle);
        }

        Ok(triangle_index)
    }

    pub fn replace_adjacent(
        &mut self,
        triangle_index: usize,
        old_adjacent_triangle: Option<usize>,
        new_adjacent_triangle: Option<usize>,
    ) {
        for vertex_index in 0..3 {
            if self.get_adjacent_triangle_index(triangle_index, vertex_index)
                == old_adjacent_triangle
            {
                self.triangle_infos[triangle_index].adjacent_triangle_indices[vertex_index] =
                    new_adjacent_triangle;
            }
        }
    }

    pub fn replace_triangle(&mut self, triangle_index: usize, new_triangle: &TriangleInfo) {
        for i in 0..3 {
            self.triangle_infos[triangle_index].vertex_indices[i] = new_triangle.vertex_indices[i];
            self.triangle_infos[triangle_index].adjacent_triangle_indices[i] =
                new_triangle.adjacent_triangle_indices[i];
        }
    }

    pub fn replace_adjacent_vertices(
        &mut self,
        triangle_index: usize,
        new_adjacent_indices: [Option<usize>; 3],
    ) {
        self.triangle_infos[triangle_index].adjacent_triangle_indices = new_adjacent_indices;
    }

    pub fn replace_vertex_with_vertex(
        &mut self,
        triangle_index: usize,
        vertex_position: usize,
        new_vertex: usize,
    ) {
        self.triangle_infos[triangle_index].vertex_indices[vertex_position] = new_vertex;
    }

    /// This method gets all the triangle indices for the triangles in a polygon outline and returns those indices.
    pub fn get_triangles_in_polygon(
        &self,
        polygon_outline: &Vec<usize>,
        triangles_to_remove: &mut Vec<usize>,
    ) -> Result<(), CustomError> {
        // This method assumes that the edges of the triangles to find were created using the same vertex order
        // It also assumes all triangles are inside a supertriangle, so no adjacent triangles are -1
        let mut adjacent_triangle_indices: Vec<usize> = Vec::new();

        // First it gets all the triangles of the outline
        for outline_index in 0..polygon_outline.len() {
            // For every edge, it gets the inner triangle that contains such edge
            if let Some(edge_in_triangle) = self.find_triangle_that_contains_edge(
                polygon_outline[outline_index],
                polygon_outline[(outline_index + 1) % polygon_outline.len()],
            ) {
                // A triangle may form a corner, with 2 consecutive outline edges. This avoids adding it twice
                let last_added_triangle = triangles_to_remove[triangles_to_remove.len() - 1];
                let first_added_triangle = triangles_to_remove[0];
                let current_triangle = edge_in_triangle.triangle_index;
                let current_edge = edge_in_triangle.edge_index;
                if triangles_to_remove.len() > 0
                    && (last_added_triangle == current_triangle
                        || first_added_triangle == current_triangle)
                {
                    continue;
                }

                triangles_to_remove.push(edge_in_triangle.triangle_index);

                let previous_outline_edge_vertex_a = polygon_outline
                    [(outline_index + polygon_outline.len() - 1) % polygon_outline.len()];
                let previous_outline_edge_vertex_b = polygon_outline[outline_index];
                let next_outline_edge_vertex_a =
                    polygon_outline[(outline_index + 1) % polygon_outline.len()];
                let next_outline_edge_vertex_b =
                    polygon_outline[(outline_index + 2) % polygon_outline.len()];

                for adjacent_index in 1..3 {
                    // For the 2 adjacent triangles of the other 2 edges in the current triangle
                    let mut is_adjacent_triangle_in_outline = false;
                    if let Some(adjacent_triangle) = self.triangle_infos[current_triangle]
                        .adjacent_triangle_indices[(current_edge + adjacent_index) % 3]
                    {
                        // Compares the contiguous edges of the outline, to the right and to the left of the current one, flipped and not flipped, with the adjacent triangle's edges
                        for k in 0..3 {
                            let adjacent_triangle_edge_vertex_a =
                                self.triangle_infos[adjacent_triangle].vertex_indices[k];
                            let adjacent_triangle_edge_vertex_b =
                                self.triangle_infos[adjacent_triangle].vertex_indices[(k + 1) % 3];

                            // TODO it seems like the comparism after the first and third || is unnecessary
                            if (adjacent_triangle_edge_vertex_a == previous_outline_edge_vertex_a
                                && adjacent_triangle_edge_vertex_b
                                    == previous_outline_edge_vertex_b)
                                || (adjacent_triangle_edge_vertex_a
                                    == previous_outline_edge_vertex_b
                                    && adjacent_triangle_edge_vertex_b
                                        == previous_outline_edge_vertex_a)
                                || (adjacent_triangle_edge_vertex_a == next_outline_edge_vertex_a
                                    && adjacent_triangle_edge_vertex_b
                                        == next_outline_edge_vertex_b)
                                || (adjacent_triangle_edge_vertex_a == next_outline_edge_vertex_b
                                    && adjacent_triangle_edge_vertex_b
                                        == next_outline_edge_vertex_a)
                            {
                                is_adjacent_triangle_in_outline = true;
                            }
                        }

                        if !is_adjacent_triangle_in_outline
                            && !triangles_to_remove.contains(&adjacent_triangle)
                        {
                            adjacent_triangle_indices.push(adjacent_triangle);
                        }
                    } else {
                        return Err(CustomError::PolygonIsOpen);
                    }
                }
            } else {
                return Err(CustomError::EdgeNotFoundInTriangles(
                    polygon_outline[outline_index],
                    polygon_outline[(outline_index + 1) % polygon_outline.len()],
                ));
            }
        }

        // Then it propagates by adjacency, stopping when an adjacent triangle has already been included in the list
        // Since all the outline triangles have been added previously, it will not propagate outside of the polygon
        while let Some(adjacent_triangle_index) = adjacent_triangle_indices.pop() {
            if triangles_to_remove.contains(&adjacent_triangle_index) {
                continue;
            }
            for i in 0..3 {
                // TODO this unwrap should not cause a panic, but what would be an appropriate error message?
                let adjacent_to_adjacent_triangle = self.triangle_infos[adjacent_triangle_index]
                    .adjacent_triangle_indices[i]
                    .unwrap();
                if !triangles_to_remove.contains(&adjacent_to_adjacent_triangle) {
                    adjacent_triangle_indices.push(adjacent_to_adjacent_triangle);
                }
            }

            triangles_to_remove.push(adjacent_triangle_index);
        }
        Ok(())
    }

    // This will find only one triangle, because edges are directional
    pub fn find_triangle_that_contains_edge(
        &self,
        edge_vertex_a: usize,
        edge_vertex_b: usize,
    ) -> Option<EdgeInfo> {
        for i in 0..self.triangle_count() {
            for j in 0..3 {
                if self.triangle_infos[i].vertex_indices[j] == edge_vertex_a
                    && self.triangle_infos[i].vertex_indices[(j + 1) % 3] == edge_vertex_b
                {
                    return Some(EdgeInfo::new(i, j, edge_vertex_a, edge_vertex_b));
                }
            }
        }
        None
    }

    // TODO because of this function this triangle set might need a vec and adj field
    // instead of what it has right now.
    // then triangle_info would be a function which takes the index..?
    // no. fuck.
    pub fn get_triangles_with_vertex(&self, vertex_index: usize) -> Vec<usize> {
        let mut output_triangles = Vec::new();
        for i in 0..self.triangle_count() {
            for j in 0..3 {
                if self.triangle_infos[i].vertex_indices[j] == vertex_index {
                    output_triangles.push(i);
                    break;
                }
            }
        }
        output_triangles
    }
}

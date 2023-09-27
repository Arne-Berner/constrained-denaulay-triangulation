use std::collections::VecDeque;

use crate::math_utils::{
    intersection_between_lines, is_point_to_the_left_of_edge, is_point_to_the_right_of_edge,
};

use super::{
    edge::Edge, edge_info::EdgeInfo, error::CustomError, found_or_added::FoundOrAdded,
    triangle::Triangle, triangle_info::TriangleInfo, vector::Vector,
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

    pub fn get_point_from_vertex(&self, vertex: usize) -> Vector {
        self.points[vertex]
    }

    pub fn get_point_from_index(&self, triangle_index: usize, vertex_index: usize) -> &Vector {
        &self.points[self.triangle_infos[triangle_index].vertex_indices[vertex_index]]
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
                    &point,
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

    /// This method gets all the triangle indices for the triangles in a polygon outline and returns those indices.
    pub fn get_triangles_in_polygon(
        &self,
        polygon_outline: &Vec<usize>,
        triangles_to_remove: &mut Vec<usize>,
    ) -> Result<(), CustomError> {
        // TODO This function takes triangles in a specific order.
        // This method assumes that the edges of the triangles to find were created using the same vertex order
        // It also assumes all triangles are inside a supertriangle, so no adjacent triangles are -1
        let mut adjacent_triangle_indices: Vec<usize> = Vec::new();

        // First it gets all the triangles of the outline
        for outline_index in 0..polygon_outline.len() {
            // For every edge, it gets the inner triangle that contains such edge
            if let Some(edge_in_triangle) = self.find_edge_info_for_vertices(
                polygon_outline[outline_index],
                polygon_outline[(outline_index + 1) % polygon_outline.len()],
            ) {
                // A triangle may form a corner, with 2 consecutive outline edges. This avoids adding it twice
                let current_triangle = edge_in_triangle.triangle_index;
                let current_edge = edge_in_triangle.edge_index;
                if triangles_to_remove.len() > 0 {
                    let last_added_triangle = triangles_to_remove[triangles_to_remove.len() - 1];
                    let first_added_triangle = triangles_to_remove[0];
                    if (last_added_triangle == current_triangle)
                        || (first_added_triangle == current_triangle)
                    {
                        continue;
                    }
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
                if let Some(adjacent_to_adjacent_triangle) =
                    self.triangle_infos[adjacent_triangle_index].adjacent_triangle_indices[i]
                {
                    if !triangles_to_remove.contains(&adjacent_to_adjacent_triangle) {
                        adjacent_triangle_indices.push(adjacent_to_adjacent_triangle);
                    }
                }
            }

            triangles_to_remove.push(adjacent_triangle_index);
        }
        Ok(())
    }

    // This will find only one edge_info, because edges are directional
    pub fn find_edge_info_for_vertices(
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
    // but not sure, since everything is on the heap as vec
    pub fn get_triangle_indices_with_vertex(&self, vertex_index: usize) -> Vec<usize> {
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

    /// This will find the triangle that contains endpoint a of the polygon and intersects with the a-b edge.
    pub fn find_triangle_that_contains_edge_start_and_intersects(
        &self,
        endpoint_a_index: usize,
        endpoint_b_index: usize,
    ) -> usize {
        let triangles_with_endpoint: Vec<usize> =
            self.get_triangle_indices_with_vertex(endpoint_a_index);

        let mut found_triangle = None;
        let endpoint_a = self.points[endpoint_a_index];
        let endpoint_b = self.points[endpoint_b_index];

        for i in 0..triangles_with_endpoint.len() {
            let mut vertex_position_in_triangle = None;
            for j in 0..3 {
                if self.triangle_infos[triangles_with_endpoint[i]].vertex_indices[j]
                    == endpoint_a_index
                {
                    vertex_position_in_triangle = Some(j);
                    break;
                }
            }
            let triangle_edge_point1 = self.points[self.triangle_infos[triangles_with_endpoint[i]]
                .vertex_indices[(vertex_position_in_triangle.unwrap() + 1) % 3]];
            let triangle_edge_point2 = self.points[self.triangle_infos[triangles_with_endpoint[i]]
                .vertex_indices[(vertex_position_in_triangle.unwrap() + 2) % 3]];

            // Is the line in the angle between the 2 contiguous edges of the triangle?
            if is_point_to_the_left_of_edge(&endpoint_a, &triangle_edge_point1, &endpoint_b)
                && is_point_to_the_left_of_edge(&triangle_edge_point2, &endpoint_a, &endpoint_b)
            {
                found_triangle = Some(triangles_with_endpoint[i]);
                break;
            }
        }

        found_triangle.expect("The beginning should at least be in the super triangle.")
    }

    pub fn get_intersecting_edges(
        &self,
        line_endpoint_a: Vector,
        line_endpoint_b: Vector,
        start_triangle: usize,
    ) -> VecDeque<Edge> {
        let mut intersected_triangle_edges = VecDeque::<Edge>::new();
        let mut is_triangle_containing_b_found = false;
        let mut triangle_index = start_triangle;

        while !is_triangle_containing_b_found {
            let mut has_crossed_edge = false;
            let mut tentative_adjacent_triangle = None;

            for i in 0..3 {
                let edge_vertex_a = self.triangle_infos[triangle_index].vertex_indices[i];
                let edge_vertex_b = self.triangle_infos[triangle_index].vertex_indices[(i + 1) % 3];
                let current_a = self.points[edge_vertex_a];
                let current_b = self.points[edge_vertex_b];

                // if one point it the endpoint, then this is the end triangle
                if current_a == line_endpoint_b || current_b == line_endpoint_b {
                    is_triangle_containing_b_found = true;
                    break;
                }

                if is_point_to_the_right_of_edge(&current_a, &current_b, &line_endpoint_b) {
                    tentative_adjacent_triangle = Some(i);
                    if intersection_between_lines(
                        &current_a,
                        &current_b,
                        &line_endpoint_a,
                        &line_endpoint_b,
                    )
                    .is_some()
                    {
                        let new_edge = Edge::new(edge_vertex_a, edge_vertex_b);

                        // TODO THIS IS SHIT
                        if let Some(temp_edge) = intersected_triangle_edges.pop_back() {
                            if temp_edge == new_edge {
                                intersected_triangle_edges.push_back(new_edge);
                            } else {
                                has_crossed_edge = true;
                                intersected_triangle_edges.push_back(temp_edge);
                                intersected_triangle_edges.push_back(new_edge);
                                triangle_index = self.triangle_infos[triangle_index]
                                    .adjacent_triangle_indices[i]
                                    .unwrap();
                                break;
                            }
                        } else {
                            has_crossed_edge = true;
                            intersected_triangle_edges.push_back(new_edge);
                            triangle_index = self.triangle_infos[triangle_index]
                                .adjacent_triangle_indices[i]
                                .unwrap();
                            break;
                        }
                    }
                }
            }

            // Continue searching at a different adjacent triangle
            if !has_crossed_edge {
                if let Some(tentative_adjacent_triangle) = tentative_adjacent_triangle {
                    triangle_index = self.triangle_infos[triangle_index].adjacent_triangle_indices
                        [tentative_adjacent_triangle]
                        .expect("This would result in an endless loop");
                }
            }
        }
        intersected_triangle_edges
    }
}

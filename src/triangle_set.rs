use bevy::prelude::{Vec2, Color};

/// A 2D triangle.
struct Triangle2D {
    /// The first vertex.
    p0: Vec2,

    /// The second vertex.
    p1: Vec2,

    /// The third vertex.
    p2: Vec2,
}

impl Triangle2D {
    /// Constructor that receives the 3 vertices.
    fn new(point0: Vec2, point1: Vec2, point2: Vec2) -> Self {
        Self {
            p0: point0,
            p1: point1,
            p2: point2,
        }
    }

    /// Gets a vertex by its index.
    fn get_vertex(&self, index: usize) -> Vec2 {
        debug_assert!(
            index < 3,
            "The index of the triangle vertex must be in the range [0, 2]."
        );

        match index {
            0 => self.p0,
            1 => self.p1,
            2 => self.p2,
            _ => panic!("Invalid index"),
        }
    }
}

pub struct DelaunayTriangleEdge {
    pub triangle_index: Option<usize>,
    pub edge_index: Option<usize>,
    pub edge_vertex_a: usize,
    pub edge_vertex_b: usize,
}

impl DelaunayTriangleEdge {
    pub fn new(
        triangle_index: Option<usize>,
        edge_index: Option<usize>,
        edge_vertex_a: usize,
        edge_vertex_b: usize,
    ) -> Self {
        DelaunayTriangleEdge {
            triangle_index,
            edge_index,
            edge_vertex_a,
            edge_vertex_b,
        }
    }
}

/// Data that describes a triangle and its context in a triangulation.
pub struct DelaunayTriangle {
    /// The indices of the points that define the triangle.
    p: [usize; 3],
    /// The indices of the triangles that are adjacent.
    adjacent: [Option<usize>; 3],
}

impl DelaunayTriangle {
    /// Constructor that receives 3 vertex indices.
    pub fn new(point0: usize, point1: usize, point2: usize) -> Self {
        DelaunayTriangle {
            p: [point0, point1, point2],
            adjacent: [None, None, None],
        }
    }

    // TODO use builder pattern instead
    // Original code used -1 for non existent adjacents instead of option
    // Is that more performant?
    /// Constructor that receives all the data.
    pub fn new_with_adjacent(
        point0: usize,
        point1: usize,
        point2: usize,
        adjacent0: Option<usize>,
        adjacent1: Option<usize>,
        adjacent2: Option<usize>,
    ) -> Self {
        DelaunayTriangle {
            p: [point0, point1, point2],
            adjacent: [adjacent0, adjacent1, adjacent2],
        }
    }
}

pub struct DelaunayTriangleSet {
    /// The indices of the adjacent triangles of every triangle, so there are 3 indices per triangle, and each index is the position of the triangle in groups of 3.
    adjacent_triangles: Vec<Option<usize>>,

    /// The indices of the vertices of every triangle, so there are 3 indices per triangle, and each index is the position of the point in the points array.
    triangle_vertices: Vec<usize>,

    /// The real points in the 2D space.
    points: Vec<Vec2>,

    //TODO does this work better as Option?
    // Indicates that the index of a vertex, edge or triangle is not defined or was not found
    //const NOT_FOUND: usize = -1,

    // Indicates that there is no adjacent triangle
    //const NO_ADJACENT_TRIANGLE: usize = -1,
}

impl DelaunayTriangleSet {
    /// Constructor that receives the expected number of triangles to store. It will reserve memory accordingly.
    pub fn new(expected_triangles: usize) -> Self {
        DelaunayTriangleSet {
            adjacent_triangles: Vec::with_capacity(expected_triangles * 3),
            triangle_vertices: Vec::with_capacity(expected_triangles * 3),
            points: Vec::with_capacity(expected_triangles),
        }
    }

    /// Removes all the data stored in the buffers, while keeping the memory.
    pub fn clear(&mut self) {
        self.adjacent_triangles.clear();
        self.triangle_vertices.clear();
        self.points.clear();
    }

    /// Modifies the capacity of the buffer, reserving new memory if necessary, according to the new expected number of triangles.
    /// * `expectedTriangles` - The expected number of triangles to store.
    pub fn set_capacity(&mut self, expected_triangles: usize) {
        if self.adjacent_triangles.capacity() < expected_triangles * 3 {
            self.adjacent_triangles.reserve(expected_triangles * 3);
        }

        if self.triangle_vertices.capacity() < expected_triangles * 3 {
            self.triangle_vertices.reserve(expected_triangles * 3);
        }

        if self.points.capacity() < expected_triangles {
            self.points.reserve(expected_triangles);
        }
    }

    /// Gets all the points of the stored triangles.
    pub fn points(&self) -> &Vec<Vec2> {
        &self.points
    }

    /// Gets the indices of the vertices of all the stored triangles.
    pub fn triangles(&self) -> &Vec<usize> {
        &self.triangle_vertices
    }

    /// Gets the amount of triangles stored.
    pub fn triangle_count(&self) -> usize {
        self.triangle_vertices.len() as usize / 3
    }

    /// Forms a new triangle using the existing points.
    /// Returns the index of the new triangle.
    pub fn add_triangle(&mut self, new_triangle: DelaunayTriangle) -> usize {
        for adjacent_triangle in new_triangle.adjacent {
            self.adjacent_triangles.push(adjacent_triangle)
        }
        for point in new_triangle.p {
            self.triangle_vertices.push(point)
        }

        self.triangle_count() - 1
    }

    /// Adds a new point to the triangle set. This does neither form triangles nor edges.
    /// Returns the index of the point.
    pub fn add_point(&mut self, point: Vec2) -> usize {
        self.points.push(point);
        self.points.len() as usize - 1
    }
    /// Forms a new triangle using new points.
    ///
    /// # Arguments
    ///
    /// * `p0` - The point for the first vertex.
    /// * `p1` - The point for the second vertex.
    /// * `p2` - The point for the third vertex.
    /// * `adjacent_triangle0` - The index of the first adjacent triangle.
    /// * `adjacent_triangle1` - The index of the second adjacent triangle.
    /// * `adjacent_triangle2` - The index of the third adjacent triangle.
    ///
    /// # Returns
    ///
    /// The index of the new triangle.
    pub fn add_triangle_from_points(
        &mut self,
        p0: Vec2,
        p1: Vec2,
        p2: Vec2,
        adjacent_triangle0: Option<usize>,
        adjacent_triangle1: Option<usize>,
        adjacent_triangle2: Option<usize>,
    ) -> usize {
        self.adjacent_triangles.push(adjacent_triangle0);
        self.adjacent_triangles.push(adjacent_triangle1);
        self.adjacent_triangles.push(adjacent_triangle2);
        self.triangle_vertices.push(self.add_point(p0));
        self.triangle_vertices.push(self.add_point(p1));
        self.triangle_vertices.push(self.add_point(p2));

        return self.triangle_count() - 1;
    }

    /// Given the index of a point, it obtains all the existing triangles that share that point.
    ///
    /// # Arguments
    ///
    /// * `vertex_index` - The index of the point that is a vertex of the triangles.
    /// * `output_triangles` - The indices of the triangles that have that point as one of their vertices. No elements will be removed from the list.
    pub fn get_triangles_with_vertex(
        &self,
        vertex_index: usize,
        output_triangles: &mut Vec<usize>,
    ) {
        for i in 0..self.triangle_count() {
            for j in 0..3 {
                if self.triangle_vertices[i * 3 + j] == vertex_index {
                    output_triangles.push(i);
                    break;
                }
            }
        }
    }

    /// Gets the points of a triangle.
    ///
    /// # Arguments
    ///
    /// * `triangle_index` - The index of the triangle.
    ///
    /// # Returns
    ///
    /// The triangle.
    pub fn get_triangle_points(&self, triangle_index: usize) -> Triangle2D {
        Triangle2D::new(
            self.points[self.triangle_vertices[triangle_index * 3]],
            self.points[self.triangle_vertices[triangle_index * 3 + 1]],
            self.points[self.triangle_vertices[triangle_index * 3 + 2]],
        )
    }

    /// Gets the data of a triangle.
    ///
    /// # Arguments
    ///
    /// * `triangle_index` - The index of the triangle.
    ///
    /// # Returns
    ///
    /// The triangle data.
    pub fn get_triangle(&self, triangle_index: usize) -> DelaunayTriangle {
        DelaunayTriangle::new_with_adjacent(
            self.triangle_vertices[triangle_index * 3],
            self.triangle_vertices[triangle_index * 3 + 1],
            self.triangle_vertices[triangle_index * 3 + 2],
            self.adjacent_triangles[triangle_index * 3],
            self.adjacent_triangles[triangle_index * 3 + 1],
            self.adjacent_triangles[triangle_index * 3 + 2],
        )
    }

    /// Given the outline of a closed polygon, expressed as a list of vertices, it finds all the triangles that lay inside of the figure.
    ///
    /// # Arguments
    ///
    /// * `polygon_outline` - The outline, a list of vertex indices sorted counter-clockwise.
    /// * `output_triangles_in_polygon` - The list where the triangles found inside the polygon will be added. No elements are removed from this list.
    pub fn get_triangles_in_polygon(
        &self,
        polygon_outline: &Vec<usize>,
        output_triangles_in_polygon: &mut Vec<usize>,
    ) {
        // This method assumes that the edges of the triangles to find were created using the same vertex order
        // It also assumes all triangles are inside a supertriangle, so no adjacent triangles are -1
        let mut adjacent_triangles: Vec<Option<usize>> = Vec::new();

        // First it gets all the triangles of the outline
        for i in 0..polygon_outline.len() {
            // For every edge, it gets the inner triangle that contains such edge
            let triangle_edge = self.find_triangle_that_contains_edge(
                polygon_outline[i],
                polygon_outline[(i + 1) % polygon_outline.len()],
            );

            // A triangle may form a corner, with 2 consecutive outline edges. This avoids adding it twice
            if output_triangles_in_polygon.len() > 0
                && (output_triangles_in_polygon[output_triangles_in_polygon.len() - 1]
                    == triangle_edge.triangle_index
                    || output_triangles_in_polygon[0] == triangle_edge.triangle_index)
            {
                continue;
            }

            output_triangles_in_polygon.push(triangle_edge.triangle_index);

            let previous_outline_edge_vertex_a =
                polygon_outline[(i + polygon_outline.len() - 1) % polygon_outline.len()];
            let previous_outline_edge_vertex_b = polygon_outline[i];
            let next_outline_edge_vertex_a = polygon_outline[(i + 1) % polygon_outline.len()];
            let next_outline_edge_vertex_b = polygon_outline[(i + 2) % polygon_outline.len()];

            for j in 1..3 {
                // For the 2 adjacent triangles of the other 2 edges
                // TODO should this be if let instead?
                let adjacent_triangle = adjacent_triangles
                    [triangle_edge.triangle_index * 3 + (triangle_edge.edge_index + j) % 3];
                let mut is_adjacent_triangle_in_outline = false;

                // Compares the contiguous edges of the outline, to the right and to the left of the current one, flipped and not flipped, with the adjacent triangle's edges
                for k in 0..3 {
                    let current_triangle_edge_vertex_a =
                        self.triangle_vertices[adjacent_triangle * 3 + k];
                    let current_triangle_edge_vertex_b =
                        self.triangle_vertices[adjacent_triangle * 3 + (k + 1) % 3];

                    if (current_triangle_edge_vertex_a == previous_outline_edge_vertex_a
                        && current_triangle_edge_vertex_b == previous_outline_edge_vertex_b)
                        || (current_triangle_edge_vertex_a == previous_outline_edge_vertex_b
                            && current_triangle_edge_vertex_b == previous_outline_edge_vertex_a)
                        || (current_triangle_edge_vertex_a == next_outline_edge_vertex_a
                            && current_triangle_edge_vertex_b == next_outline_edge_vertex_b)
                        || (current_triangle_edge_vertex_a == next_outline_edge_vertex_b
                            && current_triangle_edge_vertex_b == next_outline_edge_vertex_a)
                    {
                        is_adjacent_triangle_in_outline = true;
                    }
                }

                if !is_adjacent_triangle_in_outline
                    && !output_triangles_in_polygon.contains(&adjacent_triangle)
                {
                    adjacent_triangles.push(adjacent_triangle);
                }
            }
        }

        // Then it propagates by adjacency, stopping when an adjacent triangle has already been included in the list
        // Since all the outline triangles have been added previously, it will not propagate outside of the polygon
        while !adjacent_triangles.is_empty() {
            if let Some(current_triangle) = adjacent_triangles.pop().unwrap() {
                if output_triangles_in_polygon.contains(&current_triangle) {
                    continue;
                }
                for i in 0..3 {
                    if let adjacent_triangle = adjacent_triangles[current_triangle * 3 + i] {
                        if adjacent_triangle != None
                            && !output_triangles_in_polygon.contains(&adjacent_triangle)
                        {
                            adjacent_triangles.push(adjacent_triangle);
                        }
                    }
                }

                output_triangles_in_polygon.push(current_triangle);
            }
        }
    }

    /// <summary>
    /// Finds the intersected edges of a line segment with triangles in a mesh.
    /// </summary>
    /// * `lineEndpointA` - The first point of the line segment.
    /// * `lineEndpointB` - The second point of the line segment.
    /// * `startTriangle` - The index of the triangle from which to start searching for intersections.
    /// * `intersectingEdges` - The list where the intersected triangle edges will be added. No elements will be removed from this list.
    pub fn get_intersecting_edges(
        &self,
        line_endpoint_a: Vec2,
        line_endpoint_b: Vec2,
        // TODO should tirangle index be optional?
        start_triangle: Option<usize>,
        intersecting_edges: &mut Vec<DelaunayTriangleEdge>,
    ) {
        let mut is_triangle_containing_b_found = false;
        let mut triangle_index = start_triangle;

        while !is_triangle_containing_b_found {
            //DrawTriangle(triangle_index, Color.green);

            let mut has_crossed_edge = false;
            let mut tentative_adjacent_triangle = None;

            for i in 0..3 {
                if self.points[self.triangle_vertices[triangle_index * 3 + i]] == line_endpoint_b
                    || self.points[self.triangle_vertices[triangle_index * 3 + (i + 1) % 3]]
                        == line_endpoint_b
                {
                    is_triangle_containing_b_found = true;
                    break;
                }

                if math_utils::is_point_to_the_right_of_edge(
                    self.points[self.triangle_vertices[triangle_index * 3 + i]],
                    self.points[self.triangle_vertices[triangle_index * 3 + (i + 1) % 3]],
                    line_endpoint_b,
                ) {
                    tentative_adjacent_triangle = i;

                    //Debug.DrawLine(points[triangle_vertices[triangle_index * 3 + i]], points[triangle_vertices[triangle_index * 3 + (i + 1) % 3]], Color.green, 10.0f);

                    let mut intersection_point;

                    if math_utils::intersection_between_lines(
                        self.points[self.triangle_vertices[triangle_index * 3 + i]],
                        self.points[self.triangle_vertices[triangle_index * 3 + (i + 1) % 3]],
                        line_endpoint_a,
                        line_endpoint_b,
                        &mut intersection_point,
                    ) {
                        has_crossed_edge = true;

                        intersecting_edges.push(DelaunayTriangleEdge::new(
                            None,
                            None,
                            self.triangle_vertices[triangle_index * 3 + i],
                            self.triangle_vertices[triangle_index * 3 + (i + 1) % 3],
                        ));

                        //Debug.DrawLine(points[triangle_vertices[triangle_index * 3 + i]], points[triangle_vertices[triangle_index * 3 + (i + 1) % 3]], Color.yellow, 10.0f);
                        //const float xline_length = 0.008f;
                        //Debug.DrawRay(intersection_point - new Vec2(xline_length * 0.5f, xline_length * 0.5f), new Vec2(xline_length, xline_length), Color.red, 10.0f);
                        //Debug.DrawRay(intersection_point + new Vec2(-xline_length * 0.5f, xline_length * 0.5f), new Vec2(xline_length, -xline_length), Color.red, 10.0f);

                        // The point is in the exterior of the triangle (vertices are sorted CCW, the right side is always the exterior from the perspective of the A->B edge)
                        triangle_index = self.adjacent_triangles[triangle_index * 3 + i];

                        break;
                    }
                }
            }

            // Continue searching at a different adjacent triangle
            if !has_crossed_edge {
                triangle_index =
                    self.adjacent_triangles[triangle_index * 3 + tentative_adjacent_triangle];
            }
        }
    }
    /// Gets a point by its index.
    ///
    /// # Arguments
    ///
    /// * `point_index` - The index of the point.
    ///
    /// # Returns
    ///
    /// The point that corresponds to the index.
    pub fn get_point_by_index(&self, point_index: usize) -> Vec2 {
        self.points[point_index]
    }

    /// Gets the index of a point, if there is any that coincides with it in the triangulation.
    ///
    /// # Arguments
    ///
    /// * `point` - The point that is expected to exist already.
    ///
    /// # Returns
    ///
    /// The index of the point. If the point does not exist, -1 is returned.
    pub fn get_index_of_point(&self, point: Vec2) -> isize {
        let mut index = 0;

        while index < self.points.len() && self.points[index] != point {
            index += 1;
        }

        if index == self.points.len() {
            -1
        } else {
            index as isize
        }
    }

    /// Given an edge AB, it searches for the triangle that has an edge with the same vertices in the same order.
    /// Remember that the vertices of a triangle are sorted counter-clockwise.
    /// edgeVertexA: The index of the first vertex of the edge.
    /// edgeVertexB: The index of the second vertex of the edge.
    /// Returns the data of the triangle.
    pub fn find_triangle_that_contains_edge(
        &self,
        edge_vertex_a: usize,
        edge_vertex_b: usize,
    ) -> DelaunayTriangleEdge {
        let mut found_triangle =
            DelaunayTriangleEdge::new(None, None, edge_vertex_a, edge_vertex_b);

        for i in 0..self.triangle_count() {
            for j in 0..3 {
                if self.triangle_vertices[i * 3 + j] == edge_vertex_a
                    && self.triangle_vertices[i * 3 + (j + 1) % 3] == edge_vertex_b
                {
                    found_triangle.triangle_index = i;
                    found_triangle.edge_index = j;
                    break;
                }
            }
        }

        found_triangle
    }

    /// Given a point, it searches for a triangle that contains it.
    /// point: The point expected to be contained by a triangle.
    /// start_triangle: The index of the first triangle to check.
    /// Returns the index of the triangle that contains the point.
    pub fn find_triangle_that_contains_point(&self, point: Vec2, start_triangle: usize) -> usize {
        let mut is_triangle_found = false;
        let mut triangle_index = start_triangle;
        let mut checked_triangles = 0;

        while !is_triangle_found && checked_triangles < self.triangle_count() {
            is_triangle_found = true;

            for i in 0..3 {
                if math_utils::is_point_to_the_right_of_edge(
                    self.points[self.triangle_vertices[triangle_index * 3 + i]],
                    self.points[self.triangle_vertices[triangle_index * 3 + (i + 1) % 3]],
                    point,
                ) {
                    // The point is in the exterior of the triangle (vertices are sorted CCW, the right side is always the exterior from the perspective of the A->B edge)
                    triangle_index = self.adjacent_triangles[triangle_index * 3 + i];

                    is_triangle_found = false;
                    break;
                }
            }

            checked_triangles += 1;
        }

        if checked_triangles >= self.triangle_count() && self.triangle_count() > 1 {
            println!("Unable to find a triangle that contains the point ({:?}), starting at triangle {}. Are you generating very small triangles?", point, start_triangle);
        }

        triangle_index
    }
    /// Given an edge AB, it searches for a triangle that contains the first point and the beginning of the edge.
    /// * `endpointAIndex` - The index of the first point.
    /// * `endpointBIndex` - The index of the second point.
    /// <returns>The index of the triangle that contains the first line endpoint.</returns>
    pub fn find_triangle_that_contains_line_endpoint(
        &self,
        endpoint_a_index: usize,
        endpoint_b_index: usize,
    ) -> usize {
        let mut triangles_with_endpoint: Vec<usize> = Vec::new();
        self.get_triangles_with_vertex(endpoint_a_index, &mut triangles_with_endpoint);

        let mut found_triangle = NOT_FOUND;
        let endpoint_a = self.points[endpoint_a_index];
        let endpoint_b = self.points[endpoint_b_index];
        //Debug.DrawLine(endpointA + Vec2.up * 0.01f, endpointB + Vec2.up * 0.01f, Color.yellow, 10.0f);

        for i in 0..triangles_with_endpoint.len() {
            let vertex_position_in_triangle =
                if self.triangle_vertices[triangles_with_endpoint[i] * 3] == endpoint_a_index {
                    0
                } else if self.triangle_vertices[triangles_with_endpoint[i] * 3 + 1]
                    == endpoint_a_index
                {
                    1
                } else {
                    2
                };
            let triangle_edge_point1 = self.points[self.triangle_vertices
                [triangles_with_endpoint[i] * 3 + (vertex_position_in_triangle + 1) % 3]];
            let triangle_edge_point2 = self.points[self.triangle_vertices
                [triangles_with_endpoint[i] * 3 + (vertex_position_in_triangle + 2) % 3]];

            // Is the line in the angle between the 2 contiguous edges of the triangle?
            if math_utils::is_point_to_the_right_of_edge(
                triangle_edge_point1,
                endpoint_a,
                endpoint_b,
            ) && math_utils::is_point_to_the_right_of_edge(
                endpoint_a,
                triangle_edge_point2,
                endpoint_b,
            ) {
                found_triangle = triangles_with_endpoint[i];
                break;
            }
        }

        found_triangle
    }

    /// <summary>
    /// Stores the adjacency data of a triangle.
    /// </summary>
    /// * `triangleIndex` - The index of the triangle whose adjacency data is to be written.
    /// * `adjacentsToTriangle` - The adjacency data, 3 triangle indices sorted counter-clockwise.
    pub fn set_triangle_adjacency(
        &mut self,
        triangle_index: usize,
        adjacents_to_triangle: *mut Option<usize>,
    ) {
        for i in 0..3 {
            // TODO do i need unsafe here?
            self.adjacent_triangles[triangle_index * 3 + i] =
                unsafe { *adjacents_to_triangle.offset(i as isize) };
        }
    }

    /// Given a triangle, it searches for an adjacent triangle and replaces it with another adjacent triangle.
    ///
    /// # Arguments
    ///
    /// * `triangle_index` - The index of the triangle whose adjacency data is to be modified.
    /// * `old_adjacent_triangle` - The index of the adjacent triangle to be replaced.
    /// * `new_adjacent_triangle` - The new index of an adjacent triangle that will replace the existing one.
    pub fn replace_adjacent(
        &mut self,
        triangle_index: usize,
        old_adjacent_triangle: Option<usize>,
        new_adjacent_triangle: Option<usize>,
    ) {
        for i in 0..3 {
            if self.adjacent_triangles[triangle_index * 3 + i] == old_adjacent_triangle {
                self.adjacent_triangles[triangle_index * 3 + i] = new_adjacent_triangle;
            }
        }
    }

    /// Replaces all the data of a given triangle. The index of the triangle will remain the same.
    ///
    /// # Arguments
    ///
    /// * `triangle_index` - The index of the triangle whose data is to be replaced.
    /// * `new_triangle` - The new data that will replace the existing one.
    pub fn replace_triangle(&mut self, triangle_index: usize, new_triangle: DelaunayTriangle) {
        for i in 0..3 {
            self.triangle_vertices[triangle_index * 3 + i] = new_triangle.p[i];
            self.adjacent_triangles[triangle_index * 3 + i] = new_triangle.adjacent[i];
        }
    }

    pub fn draw_triangle(&self, triangle_index: usize, color: Color) {
        for i in 0..3 {
            let start_point = self.points[self.triangle_vertices[triangle_index * 3 + i]];
            let end_point = self.points[self.triangle_vertices[triangle_index * 3 + (i + 1) % 3]];
            // TODO replace every debug with gizmo
            // Debug::draw_line(start_point, end_point, color, 10.0);
        }
    }

    pub fn log_dump(&self) {
        for i in 0..self.triangle_count() {
            let mut log_entry = format!("Triangle {}<color=yellow>(", i);

            for j in 0..3 {
                log_entry += &self.triangle_vertices[i * 3 + j].to_string();

                if j < 2 {
                    log_entry += ", ";
                }
            }

            log_entry += ")</color>-A(";

            for j in 0..3 {
                if let Some(triangle) = self.adjacent_triangles[i * 3 + j]{
                    log_entry += &triangle.to_string();                }

                if j < 2 {
                    log_entry += ", ";
                }
            }

            log_entry += ")-v(";

            for j in 0..3 {
                log_entry += &self.points[self.triangle_vertices[i * 3 + j]].to_string();

                if j < 2 {
                    log_entry += ", ";
                }
            }

            log_entry += ")";

            // Debug::log(log_entry);
        }
    }
}

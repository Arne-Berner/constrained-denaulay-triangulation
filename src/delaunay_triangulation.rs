use std::time::Duration;

use bevy::{render::primitives::Aabb, prelude::Vec2, math::Vec3A};

use crate::{point_bin_grid::PointBinGrid, triangle_set::{DelaunayTriangleSet, Triangle2D, DelaunayTriangle, DelaunayTriangleEdge, Edge}, math_utils};

/// Encapsulates the entire constrained Delaunay triangulation algorithm, according to S. W. Sloan's proposal, and stores the resulting triangulation.
/// Instantiate this struct and call triangulate to obtain the triangulation of a point cloud.
pub struct DelaunayTriangulation {
    // The bin grid used for optimizing the search of triangles that contain a points
    grid: PointBinGrid,

    // The metadata of all the generated triangles
    // TODO Make this a ressource
    triangle_set: Option<DelaunayTriangleSet>,

    // The stack of adjacent triangles, used when checking for the Delaunay constraint
    adjacent_triangles: Option<Vec<usize>>,

    // A stack, parallel to the adjacent triangles stack, that contains the local index [0, 2] of the edge shared among the adjacent triangle
    // of the other stack and the triangle that was processed before it
    adjacent_triangle_edges: Vec<usize>,

    // The list of triangles to be discarded (holes and supertriangle)
    // TODO make this a ressource
    triangles_to_remove: Option<Vec<usize>>,

    // The bounding box of the main point cloud
    // TODO make this a ressource
    main_point_cloud_bounds: Aabb,
}

impl DelaunayTriangulation {
    /// Gets the metadata of all the generated triangles.
    pub fn triangle_set(&self) -> &Option<DelaunayTriangleSet> {
        &self.triangle_set
    }

    /// Gets the triangles generated by the triangulate method that should be discarded, those that are inside holes or exclusively belong to the supertriangle.
    pub fn discarded_triangles(&self) -> Option<&Vec<usize>> {
        match &self.triangles_to_remove{
            Some(triangles) =>return Some(triangles),
            None => None,
        }
    }

    /// Generates the triangulation of a point cloud that fulfills the Delaunay constraint. It allows the creation of holes in the
    /// triangulation, formed by closed polygons that do not overlap each other.
    ///
    /// # Arguments
    ///
    /// * `inputPoints` - The main point cloud. It must contain, at least, 3 points.
    /// * `maximumAreaTesselation` - Optional. When it is greater than zero, all the triangles of the main point cloud will be tessellated until none of them occupies
    /// an area greater than this value.
    /// * `constrainedEdges` - Optional. The list of holes. Each hole must be defined by a closed polygon formed by consecutive points sorted counter-clockwise.
    /// It does not matter if the polugons are convex or concave. It is preferable that holes lay inside the main point cloud.
    pub fn triangulate(
        &self,
        input_points: &Vec<Vec2>,
        maximum_area_tesselation: f32,
        constrained_edges: Option<&Vec<Vec<Vec2>>>,
    ) {
        // Initialize containers
        if let Some(mut triangle_set) = self.triangle_set {
            triangle_set.clear();
            triangle_set.set_capacity(input_points.len() - 2);
        } else {
            self.triangle_set = Some(DelaunayTriangleSet::new(input_points.len() - 2));
        }

        if let Some(adjacent_triangles) = self.adjacent_triangles {
            self.adjacent_triangles = Some(Vec::<usize>::with_capacity(input_points.len() - 2));
            self.adjacent_triangle_edges = Vec::<usize>::with_capacity(input_points.len() - 2);
        } else {
            if let Some(mut adjacent_triangles) = self.adjacent_triangles {
                adjacent_triangles.clear();
            }
            self.adjacent_triangle_edges.clear();
        }

        if let Some(mut triangles_to_remove) = self.triangles_to_remove {
            triangles_to_remove.clear();
        } else {
            self.triangles_to_remove = Some(Vec::<usize>::new());
        }

            // 1: Normalization
        self.main_point_cloud_bounds =
            DelaunayTriangulation::calculate_bounds_with_left_bottom_corner_at_origin(&input_points);

        let mut normalized_points = input_points.clone();
        DelaunayTriangulation::normalize_points(&mut normalized_points, &self.main_point_cloud_bounds);

            // 2: Addition of points to the space partitioning grid
        let normalized_cloud_bounds =
            DelaunayTriangulation::calculate_bounds_with_left_bottom_corner_at_origin(&normalized_points);
        let grid = PointBinGrid::new(
            (input_points.len() as f32).sqrt().sqrt().ceil() as usize,
            Vec2::new(normalized_cloud_bounds.half_extents.x, normalized_cloud_bounds.half_extents.z),
        );

        for point in normalized_points {
            grid.add_point(point);
        }

            // 3: Supertriangle initialization
        let supertriangle = Triangle2D::new(
            Vec2::new(-100.0, -100.0),
            Vec2::new(100.0, -100.0),
            Vec2::new(0.0, 100.0),
        );
        self.triangle_set.unwrap().add_triangle_from_points(
            supertriangle.p0,
            supertriangle.p1,
            supertriangle.p2,
            None,
            None,
            None,
        );

            // 4: Adding points to the Triangle set and Triangulation
            // Points are added one at a time, and points that are close together are inserted together because they are sorted in the grid, 
            // so a later step for finding their containing triangle is faster
        for cell in grid.cells.iter() {
                for point in cell {
                    // All the points in the bin are added together, one by one
                    self.add_point_to_triangulation(*point);
                }
        }

        if maximum_area_tesselation > 0.0 {
            self.tesselate(maximum_area_tesselation);
        }

            // 5: Holes creation (constrained edges)
        if let Some(constrained_edges) = constrained_edges {
                // Adds the points of all the polygons to the triangulation
            let mut constrained_edge_indices = Vec::new();

            for constrained_edge in constrained_edges {
                    // 5.1: Normalize
                let mut normalized_constrained_edges = constrained_edge.clone();
                DelaunayTriangulation::normalize_points(&mut normalized_constrained_edges, &self.main_point_cloud_bounds);

                let mut polygon_edge_indices = Vec::new();

                for i in 0..normalized_constrained_edges.len() {
                    // 5.2: Add the points to the Triangle set
                    if normalized_constrained_edges[i]
                        == normalized_constrained_edges
                            [(i + 1) % normalized_constrained_edges.len()]
                    {
                        println!("The list of constrained edges contains a zero-length edge (2 consecutive coinciding points, indices {} and {}). It will be ignored.", i, (i + 1) % normalized_constrained_edges.len());
                        continue;
                    }

                    let added_point_index =
                        self.add_point_to_triangulation(normalized_constrained_edges[i]);
                    polygon_edge_indices.push(added_point_index);
                }

                constrained_edge_indices.push(polygon_edge_indices);
            }

            for constrained_edge in constrained_edge_indices {
                // TODO no unwrap please
                // 5.3: Create the constrained edges
                for i in 0..constrained_edge.len() {
                    self.add_constrained_edge_to_triangulation(
                        constrained_edge[i].unwrap(),
                        constrained_edge[(i + 1) % constrained_edge.len()].unwrap(),
                    );
                }
            }

                // 5.4: Identify all the triangles in the polygon
            for constrained_edge in constrained_edge_indices {
                let mut unwrapped_edges = Vec::<usize>::new();
                for unwrapped_edge in constrained_edge{
                    unwrapped_edges.push(unwrapped_edge.unwrap())

                }
                self.triangle_set.unwrap().
                    get_triangles_in_polygon(&unwrapped_edges, &mut self.triangles_to_remove.unwrap());
            }
        }

        self.get_supertriangle_triangles(&mut self.triangles_to_remove.unwrap());

        self.triangles_to_remove.unwrap().sort();

        DelaunayTriangulation::denormalize_points(&mut self.triangle_set.unwrap().points, &self.main_point_cloud_bounds);
    }

/// Reads the triangles generated by the Triangulate method, discarding all those triangles that are inside a hole or belong to the supertriangle.
/// 
/// # Arguments
/// 
/// * `outputTriangles` - The list to which the triangles will be added. No elements will be removed from this list.
pub fn get_triangles_discarding_holes(&self, output_triangles: &mut Vec<Triangle2D>) {
    if output_triangles.capacity() < self.triangle_set.unwrap().triangle_count() {
        output_triangles.reserve(self.triangle_set.unwrap().triangle_count() - output_triangles.capacity());
    }

    // Output filtering
    for i in 0..self.triangle_set.unwrap().triangle_count() {
        let mut is_triangle_to_be_removed = false;

        // Is the triangle in the "To Remove" list?
        for j in 0..self.triangles_to_remove.unwrap().len() {
            if self.triangles_to_remove.unwrap()[j] >= i {
                is_triangle_to_be_removed = self.triangles_to_remove.unwrap()[j] == i;
                break;
            }
        }

        if !is_triangle_to_be_removed {
            let triangle = self.triangle_set.unwrap().get_triangle(i);
            output_triangles.push(Triangle2D::new(self.triangle_set.unwrap().points[triangle.p[0]], self.triangle_set.unwrap().points[triangle.p[1]], self.triangle_set.unwrap().points[triangle.p[2]]));
        }
    }
}

/// Reads all the triangles generated by the Triangulate method, without discarding any.
/// 
/// # Arguments
/// 
/// * `output_triangles` - The list to which the triangles will be added. No elements will be removed from this list.
pub fn get_all_triangles(&self, output_triangles: &mut Vec<Triangle2D>) {
    if output_triangles.capacity() < self.triangle_set.unwrap().triangle_count() {
        output_triangles.reserve(self.triangle_set.unwrap().triangle_count() - output_triangles.capacity());
    }

    for i in 0..self.triangle_set.unwrap().triangle_count() {
        let triangle = self.triangle_set.unwrap().get_triangle(i);
        output_triangles.push(Triangle2D::new(
            self.triangle_set.unwrap().points[triangle.p[0]],
            self.triangle_set.unwrap().points[triangle.p[1]],
            self.triangle_set.unwrap().points[triangle.p[2]],
        ));
    }
}

/// Adds a point to the triangulation, which implies splitting a triangle into 3 pieces and checking that all triangles still fulfill the Delaunay constraint.
/// If the point coincides in space with an existing point, nothing will be done and the index of the existing point will be returned.
/// 
/// # Arguments
///
/// * `pointToInsert` - The point to add to the triangulation.
///
/// # Returns
///
/// The index of the new point in the triangle set.
fn add_point_to_triangulation(&self, point_to_insert: Vec2) -> Option<usize> {
            let adjacent_triangles = self.adjacent_triangles.expect("adjacent triangles should be initialized.");
            // Note: Adjacent triangle, opposite to the inserted point, is always at index 1
            // Note 2: Adjacent triangles are stored CCW automatically, their index matches the index of the first vertex in every edge, and it is known that vertices are stored CCW
            if let Some(triangle_set) = self.triangle_set{

            // 4.1: Check point existence
            let existing_point_index = triangle_set.get_index_of_point(point_to_insert);

            if let Some(index) = existing_point_index
            {
                return Some(index);
            }

            // 4.2: Search containing triangle
// Start at the last added triangle
            let containing_triangle_index = triangle_set.find_triangle_that_contains_point(point_to_insert, triangle_set.triangle_count()-1);
            let containing_triangle = triangle_set.get_triangle(containing_triangle_index);

            // 4.3: Store the point
            // Inserting a new point into a triangle splits it into 3 pieces, 3 new triangles
            let inserted_point = triangle_set.add_point(point_to_insert);

            // 4.4: Create 2 triangles
            let first_triangle = DelaunayTriangle::new(inserted_point, containing_triangle.p[0], containing_triangle.p[1]);
            first_triangle.adjacent[0] = None;
            first_triangle.adjacent[1] = containing_triangle.adjacent[0];
            first_triangle.adjacent[2] = Some(containing_triangle_index);
            let first_triangle_index = triangle_set.add_triangle(first_triangle);

            let second_triangle = DelaunayTriangle::new(inserted_point, containing_triangle.p[2], containing_triangle.p[0]);
            second_triangle.adjacent[0] = Some(containing_triangle_index);
            second_triangle.adjacent[1] = containing_triangle.adjacent[2];
            second_triangle.adjacent[2] = None;
            let second_triangle_index = triangle_set.add_triangle(second_triangle);

            // Sets adjacency between the 2 new triangles
            first_triangle.adjacent[0] = Some(second_triangle_index);
            first_triangle.adjacent[2] = Some(first_triangle_index);
            triangle_set.set_triangle_adjacency(first_triangle_index, &first_triangle.adjacent);
            triangle_set.set_triangle_adjacency(second_triangle_index, &second_triangle.adjacent);

            // Sets the adjacency of the triangles that were adjacent to the original containing triangle
            if let Some(adjacent_triangle) = first_triangle.adjacent[1]{
                triangle_set.replace_adjacent(adjacent_triangle, Some(containing_triangle_index), Some(first_triangle_index))
            }
            if let Some(adjacent_triangle) = second_triangle.adjacent[1]{
                triangle_set.replace_adjacent(adjacent_triangle, Some(containing_triangle_index), Some(second_triangle_index))
            }

            // 4.5: Transform containing triangle into the third
            // Original triangle is transformed into the third triangle after the point has split the containing triangle into 3
            containing_triangle.p[0] = inserted_point;
            containing_triangle.adjacent[0] = Some(first_triangle_index);
            containing_triangle.adjacent[2] = Some(second_triangle_index);
            triangle_set.replace_triangle(containing_triangle_index, containing_triangle);

            // 4.6: Add new triangles to a stack
            // Triangles that contain the inserted point are added to the stack for them to be processed by the Delaunay swapping algorithm
            if let Some(_) = containing_triangle.adjacent[1]{

                adjacent_triangles.push(containing_triangle_index);
                self.adjacent_triangle_edges.push(1);
            }

            if let Some(_) = first_triangle.adjacent[1]{
                adjacent_triangles.push(first_triangle_index);
                self.adjacent_triangle_edges.push(1);
            }

            if let Some(_) = second_triangle.adjacent[1]{
                adjacent_triangles.push(second_triangle_index);
                self.adjacent_triangle_edges.push(1);
            }
            // 4.7: Check Delaunay constraint
            self.fullfill_delaunay_constraint(self.adjacent_triangles, self.adjacent_triangle_edges);

            return Some(inserted_point);
        } else {
            return None;
        }
        }

/// Process a stack of triangles checking whether they fulfill the Delaunay constraint with respect to their adjacents, swapping edges if they do not.
/// The adjacent triangles of the processed triangles are added to the stack too, so the check propagates until they all fulfill the condition.
/// 
/// Parameters:
/// - adjacent_triangles_to_process: Initial set of triangles to check.
/// - adjacent_triangle_edges: The local index (0 to 2) of the edges shared among the triangles in adjacent_triangles_to_process and the triangles that preceded
/// them at the moment they were added. There is one edge per triangle.
fn fulfill_delaunay_constraint(&self, adjacent_triangles_to_process: &mut Vec<usize>, adjacent_triangle_edges: &mut Vec<usize>) {
    let triangle_set = self.triangle_set.expect("The triangle set should not be empty.");

            while adjacent_triangles_to_process.len() > 0
            {
                let current_triangle_to_swap = adjacent_triangles_to_process.pop().expect("Since we checked for the triangles to process before, there should be some here");
                let triangle = triangle_set.get_triangle(current_triangle_to_swap);

                let opposite_triangle_index  = adjacent_triangle_edges.pop().expect("Every triangle should have an adjacent triangle");

                if triangle.adjacent[opposite_triangle_index].is_none(){
                    continue;
                }

                let not_in_edge_vertex_index = (opposite_triangle_index + 2) % 3;
                let triangle_not_in_edge_vertex = triangle_set.get_point_by_index(triangle.p[not_in_edge_vertex_index]);

                let opposite_triangle = triangle_set.get_triangle(triangle.adjacent.[opposite_triangle_index].unwrap());
                let opposite_triangle_points = triangle_set.get_triangle_points(triangle.adjacent[opposite_triangle_index].unwrap());

                if math_utils::is_point_inside_circumcircle(opposite_triangle_points.p0, opposite_triangle_points.p1, opposite_triangle_points.p2, triangle_not_in_edge_vertex)
                {
                    // Finds the edge of the opposite triangle that is shared with the other triangle, this edge will be swapped
                    let shared_edge_vertex_index = DelaunayTriangulation::get_shared_edge(&opposite_triangle, current_triangle_to_swap);

                    // Adds the 2 triangles that were adjacent to the opposite triangle, to be processed too
                    if let Some(adjacent) = opposite_triangle.adjacent[(shared_edge_vertex_index + 1) & 3] {
                        let opposite_adjacent0 = adjacent.clone();
                        if !adjacent_triangles_to_process.contains(opposite_adjacent0){
                        adjacent_triangles_to_process.push(opposite_adjacent0);
                        let neighbor_edge = DelaunayTriangulation::get_shared_edge(&triangle_set.get_triangle(opposite_adjacent0), triangle.adjacent[opposite_triangle_index].unwrap()).unwrap();
                        adjacent_triangle_edges.push(neighbor_edge);
                        }
                    }

                    if let Some(adjacent) = opposite_triangle.adjacent[(shared_edge_vertex_index + 2) & 3] {
                        let opposite_adjacent1 = adjacent.clone();
                        if !adjacent_triangles_to_process.contains(opposite_adjacent1){
                        adjacent_triangles_to_process.push(opposite_adjacent1);
                        let neighbor_edge = DelaunayTriangulation::get_shared_edge(&triangle_set.get_triangle(opposite_adjacent1), triangle.adjacent[opposite_triangle_index].unwrap());
                        adjacent_triangle_edges.push(neighbor_edge.unwrap());
                        }
                    }

                    if let Some(adjacent) = triangle.adjacent[not_in_edge_vertex_index] {
                        let triangle_adjacent0 = adjacent.clone();
                        if !adjacent_triangles_to_process.contains(&triangle_adjacent0){
                        adjacent_triangles_to_process.push(triangle_adjacent0);
                        let neighbor_edge = DelaunayTriangulation::get_shared_edge(&triangle_set.get_triangle(triangle_adjacent0), triangle.adjacent[current_triangle_to_swap].unwrap());
                        adjacent_triangle_edges.push(neighbor_edge);
                        }

                    }

                    if let Some(adjacent) = triangle.adjacent[(not_in_edge_vertex_index + 2) % 3]{
                        let triangle_adjacent1 = adjacent.clone();
                        if !adjacent_triangles_to_process.contains(&triangle_adjacent1){
                        adjacent_triangles_to_process.push(triangle_adjacent1);
                        let neighbor_edge = self.get_shared_edge(triangle_set.get_triangle(triangle_adjacent1), triangle.adjacent[current_triangle_to_swap].unwrap());
                        adjacent_triangle_edges.push(neighbor_edge);
                        }

                    }
                    
                    // 4.8: Swap edges
                    swap_edges(current_triangle_to_swap, triangle, not_in_edge_vertex_index, opposite_triangle, shared_edge_vertex_index);
                }
            }
        }

/// Finds the index of the edge (0 to 2) of a triangle that is shared with another triangle.
/// 
/// # Arguments
/// 
/// * `triangle` - The triangle whose edge is to be returned.
/// * `adjacent_triangle` - The index of the adjacent triangle.
/// 
/// # Returns
/// 
/// The index of the shared edge in the first triangle, from 0 to 2.
fn get_shared_edge(triangle: &DelaunayTriangle, adjacent_triangle: usize) -> Option<usize> {
    for shared_edge_vertex_local_index in 0..3 {
        if let Some(adjacent) = triangle.adjacent[shared_edge_vertex_local_index] {
            if adjacent == adjacent_triangle{
            return Some(shared_edge_vertex_local_index);
            }
        }
    }

    return None;
}

/// Given 2 adjacent triangles, it replaces the shared edge with a new edge that joins both opposite vertices. For example, triangles ABC-CBD would become ADC-ABD.
/// For the main triangle, its shared edge vertex is moved so the new shared edge vertex is 1 position behind / or 2 forward (if it was 1, now the shared edge is 0).
/// Parameters:
/// - main_triangle_index: The index of the main triangle.
/// - triangle: Data about the main triangle.
/// - not_in_edge_triangle_vertex: The local index of the vertex that is not in the shared edge, in the main triangle.
/// - opposite_triangle: Data about the triangle that opposes the main triangle.
/// - opposite_triangle_shared_edge_vertex_local_index: The local index of the vertex where the shared edge begins, in the opposite triangle.
fn swap_edges(&self, main_triangle_index: usize, main_triangle: DelaunayTriangle, not_in_edge_vertex_local_index: usize, opposite_triangle: &mut DelaunayTriangle, opposite_triangle_shared_edge_vertex_local_index: usize) {
    let opposite_vertex = (opposite_triangle_shared_edge_vertex_local_index + 2) % 3;

            //           2 _|_ a
            //       A2 _   |   _
            //       _      |      _
            //   0 _     A1 |         _  c (opposite vertex)
            //       _      |      _
            //          _   |   _
            //       A0   _ |_
            //              |
            //            1    b

            //           2 _|_ 
            //       A2 _       _ A1
            //       _             _
            //   0 _________A0_______ 1
            //   a   _             _  c
            //          _       _
            //             _ _
            //              | b
            //            

    // Only one vertex of each triangle is moved
    let opposite_triangle_index = main_triangle.adjacent[(not_in_edge_vertex_local_index + 1) % 3];
    main_triangle.p[(not_in_edge_vertex_local_index + 1) % 3] = opposite_triangle.p[opposite_vertex];
    opposite_triangle.p[opposite_triangle_shared_edge_vertex_local_index] = main_triangle.p[not_in_edge_vertex_local_index];
    opposite_triangle.adjacent[opposite_triangle_shared_edge_vertex_local_index] = main_triangle.adjacent[not_in_edge_vertex_local_index];
    main_triangle.adjacent[not_in_edge_vertex_local_index] = opposite_triangle_index;
    main_triangle.adjacent[(not_in_edge_vertex_local_index + 1) % 3] = opposite_triangle.adjacent[opposite_vertex];
    opposite_triangle.adjacent[opposite_vertex] = Some(main_triangle_index);

    let triangle_set = self.triangle_set.expect("There should be a triangle set at this point or this function has been called too early.");
    triangle_set.replace_triangle(main_triangle_index, main_triangle);
    triangle_set.replace_triangle(opposite_triangle_index, opposite_triangle);

    // Adjacent triangles are updated too
    if let Some(main_adjacent) = main_triangle.adjacent[(not_in_edge_vertex_local_index + 1) % 3]{
        triangle_set.replace_adjacent(main_adjacent, opposite_triangle_index, Some(main_triangle_index));
    }

    if let Some(opposite_adjacent) = opposite_triangle.adjacent[opposite_triangle_shared_edge_vertex_local_index]{
        triangle_set.replace_adjacent(opposite_adjacent, Some(main_triangle_index), opposite_triangle_index);
    }
}

/// Adds an edge to the triangulation in such a way that it keeps there even if it forms triangles that do not fulfill the Delaunay constraint.
/// If the edge already exists, nothing will be done.
/// The order in which the vertices of the edges are provided is important, as the edge may be part of a polygon whose vertices are sorted counterclockwise.
///
/// # Arguments
///
/// * `endpointAIndex` - The index of the first vertex of the edge, in the existing triangulation.
/// * `endpointBIndex` - The index of the second vertex of the edge, in the existing triangulation.
fn add_constrained_edge_to_triangulation(&self, endpoint_a_index: usize, endpoint_b_index: usize) {
    let triangle_set = self.triangle_set.unwrap();
            // Detects if the edge already exists
            if let Some(_) = triangle_set.find_triangle_that_contains_edge(endpoint_a_index, endpoint_b_index)
                {
                return;
            }

            let edge_endpoint_a = triangle_set.get_point_by_index(endpoint_a_index);
            let edge_endpoint_b = triangle_set.get_point_by_index(endpoint_b_index);

            // 5.3.1: Search for the triangle that contains the beginning of the new edge
            let triangle_containing_a = triangle_set.find_triangle_that_contains_line_endpoint(endpoint_a_index, endpoint_b_index);

            // 5.3.2: Get all the triangle edges intersected by the constrained edge
            let intersected_triangle_edges = Vec::<Edge>::new();
            triangle_set.get_intersecting_edges(edge_endpoint_a, edge_endpoint_b, triangle_containing_a, &mut intersected_triangle_edges);

            let new_edges = Vec::<DelaunayTriangleEdge>::new();

            while (intersected_triangle_edges.len() > 0){
                // wird eine copy erstellt?
                let current_intersected_triangle_edge = intersected_triangle_edges[intersected_triangle_edges.len() - 1];
                intersected_triangle_edges.remove(intersected_triangle_edges.len() -1);

                // 5.3.3: Form quadrilaterals and swap intersected edges
                // Deduces the data for both triangles
                if let Some(current_intersected_triangle_edge) = triangle_set.find_triangle_that_contains_edge(current_intersected_triangle_edge.edge_vertex_a, current_intersected_triangle_edge.edge_vertex_b){
                    let intersected_triangle = triangle_set.get_triangle(current_intersected_triangle_edge.triangle_index);
                    // TODO This should probably be checked for None, I think there are cases it is None.
                    let opposite_triangle = triangle_set.get_triangle(intersected_triangle.adjacent[current_intersected_triangle_edge.edge_index].unwrap());
                    let triangle_points = triangle_set.get_triangle_points(current_intersected_triangle_edge.triangle_index);

                // Gets the opposite vertex of adjacent triangle, knowing the fisrt vertex of the shared edge
                let opposite_vertex = None;

                //List<int> debugP = intersectedTriangle.DebugP;
                //List<int> debugA = intersectedTriangle.DebugAdjacent;
                //List<int> debugP2 = oppositeTriangle.DebugP;
                //List<int> debugA2 = oppositeTriangle.DebugAdjacent;

                let opposite_shared_edge_vertex = None;

                for j in 0..3
                {
                    if opposite_triangle.p[j] == intersected_triangle.p[(current_intersected_triangle_edge.edge_index + 1) % 3]
                    {
                        opposite_vertex = Some(opposite_triangle.p[(j + 2) % 3]);
                        opposite_shared_edge_vertex = j;
                        break;
                    }
                }

                let opposite_point = triangle_set.get_point_by_index(opposite_vertex.unwrap());

                if math_utils::is_quadrilateral_convex(triangle_points.p0, triangle_points.p1, triangle_points.p2, opposite_point)
                {
                    // Swap
                    let not_in_edge_triangle_vertex = (current_intersected_triangle_edge.edge_index + 2) % 3;
                    self.swap_edges(current_intersected_triangle_edge.triangle_index, intersected_triangle, not_in_edge_triangle_vertex, &mut opposite_triangle, opposite_shared_edge_vertex)                   int notInEdgeTriangleVertex = (currentIntersectedTriangleEdge.EdgeIndex + 2) % 3;;


                    // Refreshes triangle data after swapping
                    intersected_triangle = triangle_set.get_triangle(current_intersected_triangle_edge.triangle_index);

                    // Check new diagonal against the intersecting edge
                    let new_triangle_shared_edge_vertex = (current_intersected_triangle_edge.edge_index + 2) % 3;
                    let new_triangle_shared_point_a = triangle_set.get_point_by_index(intersected_triangle.p[new_triangle_shared_edge_vertex]);
                    let new_triangle_shared_point_b = triangle_set.get_point_by_index(intersected_triangle.p[(new_triangle_shared_edge_vertex + 1) % 3]);


                    let new_edge = Edge::new(intersected_triangle.p[new_triangle_shared_edge_vertex], intersected_triangle.p[(new_triangle_shared_edge_vertex + 1) % 3]);

                    if let Some(_) = math_utils::intersection_between_lines(edge_endpoint_a, edge_endpoint_b, new_triangle_shared_point_a, new_triangle_shared_point_b){
                    if new_triangle_shared_point_a != edge_endpoint_b && new_triangle_shared_point_b != edge_endpoint_b &&
                    new_triangle_shared_point_a != edge_endpoint_a && new_triangle_shared_point_b != edge_endpoint_a{ 
                        // New triangles edge still intersects with the constrained edge, so it is returned to the list
                        intersected_triangle_edges.insert(0, new_edge);
                    }
                    else {
                        new_edges.push(new_edge);
                    }
                }
                else
                {
                    // Back to the list
                    intersected_triangle_edges.insert(0, current_intersected_triangle_edge);
                }
            }
            }

            // 5.3.4. Check Delaunay constraint and swap edges
            for i in 0..new_edges.len(){
            {
                // Checks if the constrained edge coincides with the new edge
                let triangle_edge_point_a = triangle_set.get_point_by_index(new_edges[i].a());
                let triangle_edge_point_a = triangle_set.get_point_by_index(new_edges[i].b());

                if (triangle_edge_point_a == edge_endpoint_a && triangle_edge_point_b == edge_endpoint_b) || (triangle_edge_point_b == edge_endpoint_a && triangle_edge_point_a == edge_endpoint_b) {
                    continue;
                }

                // Deduces the data for both triangles
                let current_edge = triangle_set.find_triangle_that_contains_edge(new_edges[i].a(), new_edges[i].b()).expect("Those edges were just created and should contain an edge");
                let current_edge_triangle = triangle_set.get_triangle(current_edge.triangle_index);
                let triangle_vertex_not_shared = (current_edge.edge_index + 2) % 3;
                let triangle_point_not_shared = triangle_set.get_point_by_index(current_edge_triangle.p[triangle_vertex_not_shared]);
                let opposite_triangle = triangle_set.get_triangle(current_edge_triangle.adjacent[current_edge.edge_index]);
                let opposite_triangle_points = triangle_set.get_triangle_points(current_edge_triangle.adjacentj[current_edge.edge_index]);

                if math_utils::is_point_inside_circumcircle(opposite_triangle_points.p0, opposite_triangle_points.p1, opposite_triangle_points.p2, triangle_point_not_shared){
                    // Finds the edge of the opposite triangle that is shared with the other triangle, this edge will be swapped

                    for index in 0..3{
                        if opposite_triangle.adjacent[index].unwrap() == current_edge.triangle_index
                        {
                            break;
                        }
                    }

                    // Swap
                    self.swap_edges(current_edge.triangle_index, current_edge_triangle, triangle_vertex_not_shared, &mut opposite_triangle, index);
                }
            }
        }
    }
}

/// Gets all the triangles that contain any of the vertices of the supertriangle.
/// 
/// # Arguments
/// 
/// * `output_triangles` - The triangles of the supertriangle.
// TODO remove ugly sideeffect
fn get_supertriangle_triangles(&mut self, output_triangles: &mut Vec<usize>) {
    for i in 0..3 { // Vertices of the supertriangle
        let mut triangles_that_share_vertex = self.triangle_set.unwrap().get_triangles_with_vertex(i);

        for j in 0..triangles_that_share_vertex.len() {
            // if the triangles that share the vertex of the super triangles are not in there, put them in there
            if !output_triangles.contains(&triangles_that_share_vertex[j]) {
                output_triangles.push(triangles_that_share_vertex[j]);
            }
        }
    }
}

/// Calculates the bounds of a point cloud, in such a way that the minimum position becomes the center of the box.
/// 
/// # Arguments
/// 
/// * `points` - The points whose bound is to be calculated.
/// 
/// # Returns
/// 
/// The bounds that contains all the points.
fn calculate_bounds_with_left_bottom_corner_at_origin(points: &Vec<Vec2>) -> Aabb {
    // TODO is min and max switched, because of the reversed coordinates?
    let mut new_min = Vec3A::new(f32::MAX, 0., f32::MAX);
    let mut new_max = Vec3A::new(f32::MIN, 0., f32::MIN);

    for i in 0..points.len() {
        if points[i].x > new_max.x {
            new_max.x = points[i].x;
        }

        if points[i].y > new_max.y {
            new_max.y = points[i].y;
        }

        if points[i].x < new_min.x {
            new_min.x = points[i].x;
        }

        if points[i].y < new_min.y {
            new_min.y = points[i].y;
        }
    }

    let size = Vec3A::new((new_max.x - new_min.x).abs(), 0., (new_max.y - new_min.y).abs());
    let extends = size / 2.;

    Aabb{center:extends + new_min, half_extents:extends}
}

/// Normalizes a list of points according to a bounding box so all of them lay between the coordinates [0,0] and [1,1], while they conserve their 
/// relative position with respect to the others.
///
/// # Arguments
///
/// * `input_output_points` - The input points to normalize. The points in the list will be updated.
/// * `bounds` - The bounding box in which the normalization is based.
fn normalize_points(input_output_points: &mut Vec<Vec2>, bounds: &Aabb) {
    let maximum_dimension = f32::max(bounds.half_extents.x*2., bounds.half_extents.z *2.);

    for point in input_output_points {
        *point = (*point - Vec2::new(bounds.min().x, bounds.min().z)) / maximum_dimension;
    }
}

/// Denormalizes a list of points according to a bounding box so all of them lay between the coordinates determined by such box, while they conserve their 
/// relative position with respect to the others.
///
/// # Arguments
///
/// * `input_output_points` - The points to denormalize. They are expected to be previously normalized. The points in the list will be updated.
/// * `bounds` - The bounding box in which the denormalization is based.
fn denormalize_points(input_output_points: &mut Vec<Vec2>, bounds: &Aabb) {
    let maximum_dimension = f32::max(bounds.half_extents.x*2., bounds.half_extents.z *2.);

    for point in input_output_points {
        *point = *point * maximum_dimension + Vec2::new(bounds.min().x, bounds.min().z);
    }
}

/// <summary>
/// For each triangle, it splits its edges in 2 pieces, generating 4 subtriangles. The operation is repeated until none of them has an area greater than the desired value.
/// </summary>
/// <remarks>
/// The triangles that exclusively belong to the supertriangle will be ignored.
/// </remarks>
/// <param name="maximumTriangleArea">The maximum area all the triangles will have after the tessellation.</param>
fn tesselate(&mut self, maximum_triangle_area: f32) {
    let triangle_set = self.triangle_set.expect("Tesselation without triangles");
    let mut i = 2; // Skips supertriangle

    while i < triangle_set.triangle_count() - 1 {
        i += 1;

        // Skips all the Supertriangle triangles
        let mut is_supertriangle = false;
        let triangle_data = triangle_set.get_triangle(i);

        for j in 0..3 {
            if triangle_data.p[j] == 0 || triangle_data.p[j] == 1 || triangle_data.p[j] == 2 { // 0, 1 and 2 are vertices of the supertriangle
                is_supertriangle = true;
                break;
            }
        }

        if is_supertriangle {
            continue;
        }

        let triangle_points = triangle_set.get_triangle_points(i);
        let triangle_area = math_utils::calculate_triangle_area(triangle_points.p0, triangle_points.p1, triangle_points.p2);

        if triangle_area > maximum_triangle_area {
            self.add_point_to_triangulation(triangle_points.p0 + (triangle_points.p1 - triangle_points.p0) * 0.5);
            self.add_point_to_triangulation(triangle_points.p1 + (triangle_points.p2 - triangle_points.p1) * 0.5);
            self.add_point_to_triangulation(triangle_points.p2 + (triangle_points.p0 - triangle_points.p2) * 0.5);

            i = 2; // The tesselation restarts
        }
    }
}

pub fn draw_points(points: &Vec<Vec2>, duration: Duration) {
    for point in points {
        //debug::draw_ray(point, Vec2::up() * 0.2, Color::red(), duration);
        //debug::draw_ray(point, Vec2::right() * 0.2, Color::green(), duration);
    }
}


}
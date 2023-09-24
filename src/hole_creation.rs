use crate::{
    data_structures::{error::CustomError, triangle_set::TriangleSet, vector::Vector, edge_info::EdgeInfo} ,
    normalize::{normalize_points, Bounds},
    triangulation::triangulate_point,
};

/// returns triangles to remove
pub fn create_holes(
    mut triangle_set: &mut TriangleSet,
    holes: &Vec<Vec<Vector>>,
    bounds: Bounds,
) -> Result<Vec<usize>, CustomError> {
    // 8: Holes creation (constrained edges)
    // Adds the points of all the polygons to the triangulation
    let mut hole_indices = Vec::new();

    for mut hole in holes {
        // 5.1: Normalize
        let (normalized_hole, new_bounds) = normalize_points(&mut hole, Some(bounds));
        // TODO remove assert
        assert_eq!(bounds, new_bounds);

        let mut polygon_indices = Vec::new();

        for point_to_insert in normalized_hole {
            // 5.2: Add the points to the Triangle set
            let point_index: usize;
            polygon_indices.push(triangulate_point(&mut triangle_set, point_to_insert)?.value())
        }

        hole_indices.push(polygon_indices);
    }

    for constraint_edge_indices in &hole_indices {
        // 5.3: create the constrained edges
        for j in 0..constraint_edge_indices.len() {
            add_constrained_edge_to_triangulation(
                &mut triangle_set,
                constraint_edge_indices[j],
                constraint_edge_indices[(j + 1) % constraint_edge_indices.len()],
            );
        }
    }

    let mut triangles_to_remove = Vec::<usize>::new();
    // 5.4: Identify all the triangles in the polygon
    for constraint_edge_indices in &hole_indices {
        triangle_set.get_triangles_in_polygon(&constraint_edge_indices, &mut triangles_to_remove);
    }

    get_supertriangle_triangles(&mut triangle_set, &mut triangles_to_remove);

    triangles_to_remove.sort();

    return Ok(triangles_to_remove);
}

fn add_constrained_edge_to_triangulation(
    triangle_set: &mut TriangleSet,
    endpoint_a_index: usize,
    endpoint_b_index: usize,
) {
    // Detects if the edge already exists
    if let Some(_) =
        triangle_set.find_triangle_that_contains_edge(endpoint_a_index, endpoint_b_index)
    {
        return;
    }

    let edge_endpoint_a = triangle_set.get_point(endpoint_a_index);
    let edge_endpoint_b = triangle_set.get_point(endpoint_b_index);

    // 5.3.1: Search for the triangle that contains the beginning of the new edge
    let triangle_containing_a =
        triangle_set.find_triangle_that_contains_edge_start_and_intersects(endpoint_a_index, endpoint_b_index);

    // 5.3.2: Get all the triangle edges intersected by the constrained edge
    let intersected_triangle_edges = triangle_set.get_intersecting_edges(
        edge_endpoint_a,
        edge_endpoint_b,
        triangle_containing_a,
    );

    let mut new_edges = Vec::<EdgeInfo>::new();

    for intersected_triangle_edge in intersected_triangle_edges.iter().rev(){
        // 5.3.3: Form quadrilaterals and swap intersected edges
        // Deduces the data for both triangles
        if let Some(current_intersected_triangle_edge) = triangle_set
            .find_triangle_that_contains_edge(
                intersected_triangle_edge.a(),
                intersected_triangle_edge.b()
            )
        {
            let mut intersected_triangle =
                triangle_set.get_triangle(current_intersected_triangle_edge.triangle_index);
            // TODO This should probably be checked for None, I think there are cases it is None.
            let mut opposite_triangle = triangle_set.get_triangle(
                intersected_triangle.adjacent[current_intersected_triangle_edge.edge_index]
                    .unwrap(),
            );
            let triangle_points =
                triangle_set.get_triangle_points(current_intersected_triangle_edge.triangle_index);

            // Gets the opposite vertex of adjacent triangle, knowing the fisrt vertex of the shared edge
            let mut opposite_vertex = None;

            let mut opposite_shared_edge_vertex = None;

            for j in 0..3 {
                if opposite_triangle.p[j]
                    == intersected_triangle.p
                        [(current_intersected_triangle_edge.edge_index + 1) % 3]
                {
                    opposite_vertex = Some(opposite_triangle.p[(j + 2) % 3]);
                    opposite_shared_edge_vertex = Some(j);
                    break;
                }
            }

            let opposite_point = triangle_set.get_point_from_index(opposite_vertex.unwrap());

            if math_utils::is_quadrilateral_convex(
                triangle_points.p0,
                triangle_points.p1,
                triangle_points.p2,
                opposite_point,
            ) {
                // Swap
                let not_in_edge_triangle_vertex =
                    (current_intersected_triangle_edge.edge_index + 2) % 3;
                DelaunayTriangulation::swap_edges(
                    triangle_set,
                    current_intersected_triangle_edge.triangle_index,
                    &mut intersected_triangle,
                    not_in_edge_triangle_vertex,
                    &mut opposite_triangle,
                    opposite_shared_edge_vertex.unwrap(),
                );

                // Refreshes triangle data after swapping
                intersected_triangle =
                    triangle_set.get_triangle(current_intersected_triangle_edge.triangle_index);

                // Check new diagonal against the intersecting edge
                let new_triangle_shared_edge_vertex =
                    (current_intersected_triangle_edge.edge_index + 2) % 3;
                let new_triangle_shared_point_a = triangle_set
                    .get_point_from_index(intersected_triangle.p[new_triangle_shared_edge_vertex]);
                let new_triangle_shared_point_b = triangle_set.get_point_from_index(
                    intersected_triangle.p[(new_triangle_shared_edge_vertex + 1) % 3],
                );

                let new_edge = Edge::new(
                    intersected_triangle.p[new_triangle_shared_edge_vertex],
                    intersected_triangle.p[(new_triangle_shared_edge_vertex + 1) % 3],
                );

                if let Some(_) = math_utils::intersection_between_lines(
                    edge_endpoint_a,
                    edge_endpoint_b,
                    new_triangle_shared_point_a,
                    new_triangle_shared_point_b,
                ) {
                    if new_triangle_shared_point_a != edge_endpoint_b
                        && new_triangle_shared_point_b != edge_endpoint_b
                        && new_triangle_shared_point_a != edge_endpoint_a
                        && new_triangle_shared_point_b != edge_endpoint_a
                    {
                        // New triangles edge still intersects with the constrained edge, so it is returned to the list
                        intersected_triangle_edges.insert(0, new_edge);
                    } else {
                        new_edges.push(new_edge);
                    }
                } else {
                    // Back to the list
                    intersected_triangle_edges.insert(0, current_intersected_triangle_edge.edge);
                }
            }
        }

        // 5.3.4. Check Delaunay constraint and swap edges
        for i in 0..new_edges.len() {
            {
                // Checks if the constrained edge coincides with the new edge
                let triangle_edge_point_a = triangle_set.get_point_from_index(new_edges[i].a());
                let triangle_edge_point_b = triangle_set.get_point_from_index(new_edges[i].b());

                if (triangle_edge_point_a == edge_endpoint_a
                    && triangle_edge_point_b == edge_endpoint_b)
                    || (triangle_edge_point_b == edge_endpoint_a
                        && triangle_edge_point_a == edge_endpoint_b)
                {
                    continue;
                }

                // Deduces the data for both triangles
                let current_edge = triangle_set
                    .find_triangle_that_contains_edge(new_edges[i].a(), new_edges[i].b())
                    .expect("Those edges were just created and should contain an edge");
                let mut current_edge_triangle =
                    triangle_set.get_triangle(current_edge.triangle_index);
                let triangle_vertex_not_shared = (current_edge.edge_index + 2) % 3;
                let triangle_point_not_shared = triangle_set
                    .get_point_from_index(current_edge_triangle.p[triangle_vertex_not_shared]);
                let mut opposite_triangle = triangle_set
                    .get_triangle(current_edge_triangle.adjacent[current_edge.edge_index].unwrap());
                let opposite_triangle_points = triangle_set.get_triangle_points(
                    current_edge_triangle.adjacent[current_edge.edge_index].unwrap(),
                );

                if math_utils::is_point_inside_circumcircle(
                    opposite_triangle_points.p0,
                    opposite_triangle_points.p1,
                    opposite_triangle_points.p2,
                    triangle_point_not_shared,
                ) {
                    // Finds the edge of the opposite triangle that is shared with the other triangle, this edge will be swapped

                    let mut index = 0;
                    for i in 0..3 {
                        if opposite_triangle.adjacent[i].unwrap() == current_edge.triangle_index {
                            index = i;
                            break;
                        }
                    }

                    // Swap
                    DelaunayTriangulation::swap_edges(
                        triangle_set,
                        current_edge.triangle_index,
                        &mut current_edge_triangle,
                        triangle_vertex_not_shared,
                        &mut opposite_triangle,
                        index,
                    );
                }
            }
        }
    }
}

fn get_supertriangle_triangles(triangle_set: &mut TriangleSet, output_triangles: &mut Vec<usize>) {
    for i in 0..3 {
        // Vertices of the supertriangle
        let triangles_that_share_vertex = triangle_set.get_triangle_indices_with_vertex(i);

        for j in 0..triangles_that_share_vertex.len() {
            // if the triangles that share the vertex of the super triangles are not in there, put them in there
            if !output_triangles.contains(&triangles_that_share_vertex[j]) {
                output_triangles.push(triangles_that_share_vertex[j]);
            }
        }
    }
}

use std::collections::VecDeque;

use crate::{
    data_structures::{
        edge_info::EdgeInfo, error::CustomError, triangle_set::TriangleSet, vector::Vector,
    },
    math_utils::{
        intersection_between_lines, is_point_inside_circumcircle, is_quadrilateral_convex,
    },
    normalize::{normalize_points, Bounds},
    triangulation::{swap_edges, triangulate_point, TriangleIndexPair},
};

/// returns triangles to remove
pub fn create_holes(
    mut triangle_set: &mut TriangleSet,
    holes: &mut Vec<Vec<Vector>>,
    bounds: Bounds,
) -> Result<Vec<usize>, CustomError> {
    // 8: Holes creation (constrained edges)
    // Adds the points of all the polygons to the triangulation
    let mut hole_indices = Vec::new();

    for mut hole in holes {
        // 5.1: Normalize
        let (normalized_hole, _) = normalize_points(&mut hole, Some(bounds));
        let mut polygon_indices = Vec::new();

        for point_to_insert in normalized_hole {
            // 5.2: Add the points to the Triangle set
            polygon_indices.push(triangulate_point(&mut triangle_set, point_to_insert)?.value());
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
            )?;
        }
    }

    let mut triangles_to_remove = Vec::<usize>::new();
    // 5.4: Identify all the triangles in the polygon
    for constraint_edge_indices in &hole_indices {
        triangle_set
            .get_triangles_in_polygon(&constraint_edge_indices, &mut triangles_to_remove)?;
    }

    get_supertriangle_triangles(&mut triangle_set, &mut triangles_to_remove);

    triangles_to_remove.sort();

    return Ok(triangles_to_remove);
}

fn add_constrained_edge_to_triangulation(
    triangle_set: &mut TriangleSet,
    endpoint_a_index: usize,
    endpoint_b_index: usize,
) -> Result<(), CustomError> {
    // Detects if the edge already exists
    if let Some(_) = triangle_set.find_edge_info_for_triangle(endpoint_a_index, endpoint_b_index) {
        return Ok(());
    }
    // pseudocode
    // find all intersecting edges
    // starting with the first one, create a quadrileteral, with the intersecting edge vertex as the start
    // next is the endpoint_a
    // other intersecting edge vertex
    // last vertex of adjacent triangle
    // if is convex: replace
    // else: don't care??'
    // add new edge to a list
    // repeat for every one
    // check delaunay constraint for endpoint a triangle with the added new edges
    // repeat with every edge
    // can there be new triangles, that are not delaunay that way? i don't think so

    // 5.3.1: Search for the triangle that contains the beginning of the new edge
    let triangle_containing_a = triangle_set
        .find_triangle_that_contains_edge_start_and_intersects(endpoint_a_index, endpoint_b_index);
    let edge_endpoint_a = triangle_set.get_point(endpoint_a_index);
    let edge_endpoint_b = triangle_set.get_point(endpoint_b_index);

    // 5.3.2: Get all the triangle edges intersected by the constrained edge
    // TODO rewrite to VecDeque and use push front instead of insert 0?
    let intersected_triangle_edges = triangle_set.get_intersecting_edges(
        edge_endpoint_a,
        edge_endpoint_b,
        triangle_containing_a,
    );
    let mut intersected_triangle_edges = VecDeque::from(intersected_triangle_edges);
    println!(
        "intersected_triangle_edges: {:#?}",
        intersected_triangle_edges
    );

    let mut new_edges = Vec::<EdgeInfo>::new();

    while let Some(intersected_triangle_edge) = intersected_triangle_edges.pop_front() {
        // 5.3.3: Form quadrilaterals and swap intersected edges
        // Deduces the data for both triangles

        let current_triangle_index = intersected_triangle_edge.triangle_index;
        let current_intersected_edge_index = intersected_triangle_edge.edge_index;
        let mut current_triangle_info = triangle_set.get_triangle_info(current_triangle_index);
        // if we are only checking intersected edges, then there must be an adjacent triangle (at least the one containing edgepoint b)
        let opposite_triangle_index = current_triangle_info.adjacent_triangle_indices
            [current_intersected_edge_index]
            .unwrap();
        let opposite_triangle_info = triangle_set.get_triangle_info(opposite_triangle_index);
        let triangle_points = triangle_set.get_triangle(current_triangle_index);

        // Gets the opposite vertex of adjacent triangle, knowing the first vertex of the shared edge
        let mut opposite_vertex = None;

        for j in 0..3 {
            if opposite_triangle_info.vertex_indices[j]
                == current_triangle_info.vertex_indices
                    [(intersected_triangle_edge.edge_index + 1) % 3]
            {
                opposite_vertex = Some(opposite_triangle_info.vertex_indices[(j + 2) % 3]);
                break;
            }
        }

        let opposite_point = &triangle_set.get_point(opposite_vertex.unwrap());

        if is_quadrilateral_convex(
            &triangle_points.p(0),
            &triangle_points.p(1),
            &triangle_points.p(2),
            opposite_point,
        ) {
            let index_pair = TriangleIndexPair {
                adjacent: opposite_triangle_index,
                current: current_triangle_index,
            };
            swap_edges(&index_pair, triangle_set, current_intersected_edge_index)?;

            // Refreshes triangle data after swapping
            current_triangle_info = triangle_set.get_triangle_info(current_triangle_index);

            // Check new diagonal against the intersecting edge
            // the indices should always be 2 and 0, since swap edges always sets the vertices in a specific order
            let shared_vertex_a = current_triangle_info.vertex_indices[2];
            let shared_vertex_b = current_triangle_info.vertex_indices[0];
            let new_triangle_shared_point_a = triangle_set.get_point(shared_vertex_a);
            let new_triangle_shared_point_b = triangle_set.get_point(shared_vertex_b);

            // only need to create this, if it gets pushed
            let new_edge =
                EdgeInfo::new(current_triangle_index, 2, shared_vertex_a, shared_vertex_b);

            // if it still intersects after swapping, it needs to be removed
            if let Some(_) = intersection_between_lines(
                edge_endpoint_a,
                edge_endpoint_b,
                new_triangle_shared_point_a,
                new_triangle_shared_point_b,
            ) {
                // this seems wrong
                //// if none of the new shared_edge_points are edge_endpoints
                //// this is always true, in the first loop, because we are swapping with endpoint a as the start
                if new_triangle_shared_point_a != edge_endpoint_b
                    && new_triangle_shared_point_b != edge_endpoint_b
                    && new_triangle_shared_point_a != edge_endpoint_a
                    && new_triangle_shared_point_b != edge_endpoint_a
                {
                    // New triangles edge still intersects with the constrained edge, so it is returned to the list
                    intersected_triangle_edges.push_back(new_edge);
                }
            } else {
                // otherwise it needs to be checked for the delaunay constraint
                new_edges.push(new_edge);
            }
        } else {
            // If they do not form a convex quadrilateral, then they need to be checked again later
            intersected_triangle_edges.push_back(intersected_triangle_edge);
        }
    }

    // 5.3.4. Check Delaunay constraint and swap edges
    for i in 0..new_edges.len() {
        {
            // Checks if the constrained edge coincides with the new edge
            let triangle_edge_point_a = triangle_set.get_point(new_edges[i].a());
            let triangle_edge_point_b = triangle_set.get_point(new_edges[i].b());


            if (triangle_edge_point_a == edge_endpoint_a)
                && (triangle_edge_point_b == edge_endpoint_b)
            {
                continue;
            }

            //this should not happen, since the swap is always in the same order
            if (triangle_edge_point_b == edge_endpoint_a)
                && (triangle_edge_point_a == edge_endpoint_b)
            {
                continue;
            }

            // Deduces the data for both triangles
            let current_edge = triangle_set
                .find_edge_info_for_triangle(new_edges[i].a(), new_edges[i].b())
                .expect("Those edges were just created and should contain an edge");
            // the comment is right, but I had some wrong assumptions above i guess.
            // also the new edge should contain all the information needed

            let current_edge_triangle = triangle_set.get_triangle_info(current_edge.triangle_index);

            let triangle_vertex_not_shared = (current_edge.edge_index + 2) % 3;
            let triangle_point_not_shared = triangle_set
                .get_point(current_edge_triangle.vertex_indices[triangle_vertex_not_shared]);

            let opposite_triangle_index =
                current_edge_triangle.adjacent_triangle_indices[current_edge.edge_index].unwrap();

            let opposite_triangle = triangle_set.get_triangle(
                current_edge_triangle.adjacent_triangle_indices[current_edge.edge_index].unwrap(),
            );

            if is_point_inside_circumcircle(opposite_triangle, triangle_point_not_shared) {
                // Swap
                swap_edges(
                    &TriangleIndexPair {
                        adjacent: opposite_triangle_index,
                        current: current_edge.triangle_index,
                    },
                    triangle_set,
                    current_edge.edge_index,
                )?;
            }
        }
    }
    return Ok(());
}

pub fn get_supertriangle_triangles(
    triangle_set: &mut TriangleSet,
    output_triangles: &mut Vec<usize>,
) {
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

use std::collections::VecDeque;

use crate::{
    data_structures::{
        edge::Edge, edge_info::EdgeInfo, error::CustomError, triangle_set::TriangleSet,
        vector::Vector,
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
    for constraint_edge_indices in &mut hole_indices {
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
    if let Some(_) = triangle_set.find_edge_info_for_vertices(endpoint_a_index, endpoint_b_index) {
        return Ok(());
    }
    // 5.3.1: Search for the triangle that contains the beginning of the new edge
    let triangle_containing_a = triangle_set
        .find_triangle_that_contains_edge_start_and_intersects(endpoint_a_index, endpoint_b_index);
    let edge_endpoint_a = triangle_set.get_point_from_vertex(endpoint_a_index);
    let edge_endpoint_b = triangle_set.get_point_from_vertex(endpoint_b_index);

    // 5.3.2: Get all the triangle edges intersected by the constrained edge
    println!("triangles 0: {:?}, triangle 1: {:?}", triangle_set.triangle_infos[0], triangle_set.triangle_infos[1]);
    let mut intersected_triangle_edges: VecDeque<Edge> = triangle_set.get_intersecting_edges(
        edge_endpoint_a,
        edge_endpoint_b,
        triangle_containing_a,
    );

    let mut new_edges = Vec::<Edge>::new();

    while let Some(intersected_triangle_edge) = intersected_triangle_edges.pop_back() {
        println!("edge: {:?}", intersected_triangle_edge);
        let current_edge_info = triangle_set
            .find_edge_info_for_vertices(
                intersected_triangle_edge.vertex_a(),
                intersected_triangle_edge.vertex_b(),
            )
            .unwrap();
        let opposite_triangle_index = triangle_set.triangle_infos[current_edge_info.triangle_index]
            .adjacent_triangle_indices[current_edge_info.edge_index]
            .unwrap();
        // for loop to get index
        let mut opposite_vertex_index = None;
        for i in 0..3 {
            if triangle_set.triangle_infos[opposite_triangle_index].vertex_indices[i]
                == current_edge_info.vertex_a()
            {
                opposite_vertex_index = Some((i + 1) % 3);
                break;
            }
        }
        let opposite_point = triangle_set
            .get_point_from_index(opposite_triangle_index, opposite_vertex_index.unwrap());

        if is_quadrilateral_convex(
            &triangle_set.points[current_edge_info.vertex_b()],
            &edge_endpoint_a,
            &triangle_set.points[current_edge_info.vertex_a()],
            opposite_point,
        ) {
            let index_pair = TriangleIndexPair {
                current: current_edge_info.triangle_index,
                adjacent: opposite_triangle_index,
            };
            swap_edges(&index_pair, triangle_set, current_edge_info.edge_index)?;
            let new_triangle_shared_point_a =
                triangle_set.get_point_from_index(current_edge_info.triangle_index, 2);
            let new_triangle_shared_point_b =
                triangle_set.get_point_from_index(current_edge_info.triangle_index, 0);

            let new_edge = Edge::new(
                triangle_set.triangle_infos[current_edge_info.triangle_index].vertex_indices[2],
                triangle_set.triangle_infos[current_edge_info.triangle_index].vertex_indices[0],
            );

            if let Some(_) = intersection_between_lines(
                &edge_endpoint_a,
                &edge_endpoint_b,
                new_triangle_shared_point_a,
                new_triangle_shared_point_b,
            ) {
                // if it still intersects after swapping, it needs to be put into the vec again
                if *new_triangle_shared_point_a != edge_endpoint_b
                    && *new_triangle_shared_point_b != edge_endpoint_b
                    && *new_triangle_shared_point_a != edge_endpoint_a
                    && *new_triangle_shared_point_b != edge_endpoint_a
                {
                    intersected_triangle_edges.push_front(new_edge);
                } else {
                    // except if it is the polygon edge
                    new_edges.push(new_edge);
                }
            } else {
                // otherwise it needs to be checked for the delaunay constraint
                new_edges.push(new_edge);
            }
        } else {
            // If they do not form a convex quadrilateral, then they need to be checked again later
            intersected_triangle_edges.push_front(intersected_triangle_edge);
        }
    }

    // 5.3.4. Check Delaunay constraint and swap edges
    for i in 0..new_edges.len() {
        {
            // Checks if the constrained edge coincides with the new edge
            let triangle_edge_point_a = triangle_set.get_point_from_vertex(new_edges[i].vertex_a());
            let triangle_edge_point_b = triangle_set.get_point_from_vertex(new_edges[i].vertex_b());

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
                .find_edge_info_for_vertices(new_edges[i].vertex_a(), new_edges[i].vertex_b())
                .expect("Those edges were just created and the triangulation should contain them");

            let current_edge_triangle = triangle_set.get_triangle_info(current_edge.triangle_index);

            let triangle_vertex_not_shared = (current_edge.edge_index + 2) % 3;
            let triangle_point_not_shared = triangle_set.get_point_from_vertex(
                current_edge_triangle.vertex_indices[triangle_vertex_not_shared],
            );

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

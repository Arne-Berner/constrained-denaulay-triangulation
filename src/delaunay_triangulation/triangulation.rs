use bevy::{prelude::Gizmos, prelude::Vec2};
//TODO ADD TESTS FOR EVERY FUNCTION (in docs)

use crate::{
    delaunay_triangulation::normalize,
    point_bin_grid::PointBinGrid,
    triangle_set::{Triangle2D, TriangleSet, DelaunayTriangle},
};

pub fn triangulate(
    // might need to be a ref, for it to work with other systems
    input_points: Vec<Vec2>,
    maximum_area_tesselation: f32,
    constrained_edges: Option<&Vec<Vec<Vec2>>>,
    mut gizmos: Gizmos,
) {
    // Initialize containers
    let mut triangle_set = TriangleSet::new(input_points.len() - 2);
    let mut adjacent_triangles = Some(Vec::<usize>::with_capacity(input_points.len() - 2));
    let adjacent_triangle_edges = Vec::<usize>::with_capacity(input_points.len() - 2);
    let mut triangles_to_remove = Vec::<usize>::new();

    let normalized_points = normalize::normalize_points(input_points);

    println!("normalized points: {:?}", &normalized_points);

    // 2: Addition of points to the space partitioning grid
    let mut grid = PointBinGrid::new(
        //with 100 points that would result in 3 cells per side thus ~10 points per grid
        // which is the proposed overall_points^1/2 points per grid
        (input_points.len() as f32).powf(1. / 4.).round() as usize,
    );

    for point in normalized_points {
        grid.add_point(point);
    }
    println!("grid with points: {:?}", grid);

    // 3: Supertriangle initialization
    let supertriangle = Triangle2D::new(
        Vec2::new(-100.0, -100.0),
        Vec2::new(100.0, -100.0),
        Vec2::new(0.0, 100.0),
    );
    triangle_set.add_triangle_from_points(
        supertriangle.get_vertex(0),
        supertriangle.get_vertex(1),
        supertriangle.get_vertex(2),
        None,
        None,
        None,
    );

    // 4: (loop over each point) For each point P in the list of sorted points, do steps 5-7
    // Points are added one at a time, and points that are close together are inserted together because they are sorted in the grid,
    // so a later step for finding their containing triangle is faster
    for cell in grid.cells().iter() {
        for point in cell {
            // All the points in the bin are added together, one by one
            add_point_to_triangulation(
                &mut adjacent_triangles,
                &mut triangle_set,
                &mut adjacent_triangle_edges,
                *point,
            );
        }
    }

    if maximum_area_tesselation > 0.0 {
        DelaunayTriangulation::tesselate(
            &mut adjacent_triangles,
            &mut triangle_set,
            &mut self.adjacent_triangle_edges,
            maximum_area_tesselation,
        );
    }

    println!("before creating holes");
    // 5: Holes creation (constrained edges)
    if let Some(constrained_edges) = constrained_edges {
        // Adds the points of all the polygons to the triangulation
        let mut constrained_edge_indices = Vec::new();

        for constrained_edge in constrained_edges {
            // 5.1: Normalize
            let mut normalized_constrained_edges = constrained_edge.clone();
            DelaunayTriangulation::normalize_points(
                &mut normalized_constrained_edges,
                &main_point_cloud_bounds,
            );

            let mut polygon_edge_indices = Vec::new();

            for i in 0..normalized_constrained_edges.len() {
                // 5.2: Add the points to the Triangle set
                if normalized_constrained_edges[i]
                    == normalized_constrained_edges[(i + 1) % normalized_constrained_edges.len()]
                {
                    println!("The list of constrained edges contains a zero-length edge (2 consecutive coinciding points, indices {} and {}). It will be ignored.", i, (i + 1) % normalized_constrained_edges.len());
                    continue;
                }

                let added_point_index = add_point_to_triangulation(
                    &mut adjacent_triangles,
                    &mut triangle_set,
                    &mut self.adjacent_triangle_edges,
                    normalized_constrained_edges[i],
                );
                polygon_edge_indices.push(added_point_index);
            }

            constrained_edge_indices.push(polygon_edge_indices);
        }

        for edges in &constrained_edge_indices {
            // todo no unwrap please
            // 5.3: create the constrained edges
            for j in 0..edges.len() {
                DelaunayTriangulation::add_constrained_edge_to_triangulation(
                    &mut triangle_set,
                    edges[j].unwrap(),
                    edges[(j + 1) % edges.len()].unwrap(),
                );
            }
        }

        // 5.4: Identify all the triangles in the polygon
        for constrained_edge in &constrained_edge_indices {
            let mut unwrapped_edges = Vec::<usize>::new();
            for unwrapped_edge in constrained_edge {
                unwrapped_edges.push(unwrapped_edge.unwrap())
            }
            triangle_set.get_triangles_in_polygon(&unwrapped_edges, &mut triangles_to_remove);
        }
    }

    DelaunayTriangulation::get_supertriangle_triangles(&mut triangle_set, &mut triangles_to_remove);

    triangles_to_remove.sort();

    DelaunayTriangulation::denormalize_points(&mut triangle_set.points, &main_point_cloud_bounds);
}

fn add_point_to_triangulation(
    adjacent_triangles: &mut Option<Vec<usize>>,
    triangle_set: &mut TriangleSet,
    adjacent_triangle_edges: &mut Vec<usize>,
    point_to_insert: Vec2,
) -> Option<usize> {
    // Note: Adjacent triangle, opposite to the inserted point, is always at index 1
    // Note 2: Adjacent triangles are stored CCW automatically, their index matches the index of the first vertex in every edge, and it is known that vertices are stored CCW

    // 4.1: Check point existence
    let existing_point_index = triangle_set.get_index_of_point(point_to_insert);

    if let Some(index) = existing_point_index {
        return Some(index);
    }

    // 4.2: Search containing triangle
    // Start at the last added triangle
    if let Some(containing_triangle_index) = triangle_set
        .find_triangle_that_contains_point(point_to_insert, triangle_set.triangle_count() - 1)
    {
        let mut containing_triangle = triangle_set.get_triangle(containing_triangle_index);

        // 4.3: Store the point
        // Inserting a new point into a triangle splits it into 3 pieces, 3 new triangles
        let inserted_point_index = triangle_set.add_point(point_to_insert);
        let original_points = (containing_triangle.p[0], containing_triangle.p[1], containing_triangle.p[2]);

        // 4.4: Create 2 triangles
        let mut first_triangle = DelaunayTriangle::new(
            original_points.0,
            original_points.1,
            inserted_point_index,
        );
        first_triangle.adjacent[0] = containing_triangle.adjacent[0]; // the originals adjacent
        first_triangle.adjacent[1] = Some(triangle_set.triangle_count() + 1); // the second triangle
        first_triangle.adjacent[2] = Some(containing_triangle_index); // this is the original triangle, that will get changed a bit
        let first_triangle_index = triangle_set.add_triangle(&first_triangle);
        
        let mut second_triangle = DelaunayTriangle::new(
            inserted_point_index,
            original_points.1,
            original_points.2,
        );

        second_triangle.adjacent[0] = Some(first_triangle_index); 
        second_triangle.adjacent[1] = containing_triangle.adjacent[1];
        second_triangle.adjacent[2] = Some(containing_triangle_index);
        let second_triangle_index = triangle_set.add_triangle(&second_triangle);

        // Sets the adjacency of the triangles that were adjacent to the original containing triangle
        if let Some(adjacent_triangle) = first_triangle.adjacent[0] {
            triangle_set.replace_adjacent(
                adjacent_triangle,
                Some(containing_triangle_index),
                Some(first_triangle_index),
            )
        }
        if let Some(adjacent_triangle) = second_triangle.adjacent[1] {
            triangle_set.replace_adjacent(
                adjacent_triangle,
                Some(containing_triangle_index),
                Some(second_triangle_index),
            )
        }

        // 4.5: Transform containing triangle into the third
        // Original triangle is transformed into the third triangle after the point has split the containing triangle into 3
        containing_triangle.p[0] = original_points.2;
        containing_triangle.p[1] = original_points.0;
        containing_triangle.p[2] = inserted_point_index;
        containing_triangle.adjacent[0] = containing_triangle.adjacent[2];
        containing_triangle.adjacent[1] = Some(first_triangle_index);
        containing_triangle.adjacent[2] = Some(second_triangle_index);
        triangle_set.replace_triangle(containing_triangle_index, &containing_triangle);

        // GOT TO HERE
        // 4.6: Add new triangles to a stack
        // Triangles that contain the inserted point are added to the stack for them to be processed by the Delaunay swapping algorithm
        if containing_triangle.adjacent[1].is_some() {
            adjacent_triangles.push(containing_triangle_index);
            adjacent_triangle_edges.push(1);
        }

        if first_triangle.adjacent[1].is_some() {
            adjacent_triangles.push(first_triangle_index);
            adjacent_triangle_edges.push(1);
        }

        if second_triangle.adjacent[1].is_some() {
            adjacent_triangles.push(second_triangle_index);
            adjacent_triangle_edges.push(1);
        }
        // 4.7: Check Delaunay constraint
        DelaunayTriangulation::fulfill_delaunay_constraint(
            triangle_set,
            adjacent_triangles,
            adjacent_triangle_edges,
        );

        return Some(inserted_point_index);
    } else {
        return None;
    }
}

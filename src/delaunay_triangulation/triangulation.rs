use bevy::{prelude::Gizmos, prelude::Vec2};
//TODO ADD TESTS FOR EVERY FUNCTION (in docs)

use crate::{
    delaunay_triangulation::{
        data_structure::{Triangle, TriangleSet},
        normalize,
    },
    math_utils,
    point_bin_grid::PointBinGrid,
    triangle_set::{DelaunayTriangle, Triangle2D, TriangleSetOld},
};

use super::{data_structure::{FoundOrAdded, TriangleInfo}, math_utils::is_point_inside_circumcircle};

pub fn triangulate(
    // might need to be a ref, for it to work with other systems
    input_points: Vec<Vec2>,
    maximum_area_tesselation: f32,
    constrained_edges: Option<&Vec<Vec<Vec2>>>,
    mut gizmos: Gizmos,
) {
    // Initialize containers
    let mut triangle_set = TriangleSet::new(input_points.len() - 2);
    let mut triangles_to_remove = Vec::<usize>::new();

    let (normalized_points, bounds) = normalize::normalize_points(input_points);

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
    let supertriangle = Triangle::new(
        Vec2::new(-100.0, -100.0),
        Vec2::new(100.0, -100.0),
        Vec2::new(0.0, 100.0),
    );
    triangle_set.add_triangle(supertriangle);

    // 4: (loop over each point) For each point P in the list of sorted points, do steps 5-7
    // Points are added one at a time, and points that are close together are inserted together because they are sorted in the grid,
    // so a later step for finding their containing triangle is faster
    for cell in grid.cells().iter() {
        for point in cell {
            // All the points in the bin are added together, one by one
            triangulate_point(&mut triangle_set, *point);
        }
    }

    if maximum_area_tesselation > 0.0 {
        DelaunayTriangulation::tesselate(
            &mut adjacent_triangles_with_edge,
            &mut triangle_set,
            &mut adjacent_triangle_edges,
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

                let added_point_index = triangulate_point(
                    &mut adjacent_triangles_with_edge,
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

fn triangulate_point(triangle_set: &mut TriangleSet, point_to_insert: Vec2) -> Option<usize> {
    // Note: Adjacent triangle, opposite to the inserted point, is always at index 1
    // Note 2: Adjacent triangles are stored CCW automatically, their index matches the index of the first vertex in every edge, and it is known that vertices are stored CCW

    // 4.1: Check point existence
    let (inserted_point_index, found) = triangle_set.add_point(point_to_insert);
    if found == FoundOrAdded::Found {
        return Some(inserted_point_index);
    }

    // 4.2: Search containing triangle
    // Start at the last added triangle
    if let Some(containing_triangle_index) = triangle_set
        .find_triangle_that_contains_point(point_to_insert, triangle_set.triangle_count() - 1)
    {
        let mut containing_triangle = triangle_set.get_triangle_info(containing_triangle_index);

        // 5. Insert new point in triangulation and create 2 new triangles off of it
        // all the triangles take inserted point as there vertex 0, so that adjacent is 1
        let mut first_triangle = TriangleInfo::new([
            inserted_point_index,
            containing_triangle.vertex_indices[0],
            containing_triangle.vertex_indices[1],
        ])
        .with_adjacent(
            Some(triangle_set.triangle_count() + 1), // the second triangle
            containing_triangle.adjacent_triangle_indices[0], // the originals adjacent
            Some(containing_triangle_index), // this is the original triangle, that will get changed a bit
        );
        let first_triangle_index = triangle_set.add_triangle_info(&first_triangle);

        let mut second_triangle = TriangleInfo::new(
            [inserted_point_index,
            containing_triangle.vertex_indices[2],
            containing_triangle.vertex_indices[1]],
        )
        .with_adjacent(
            Some(containing_triangle_index),
            containing_triangle.adjacent_triangle_indices[2],
            Some(first_triangle_index),
        );

        let second_triangle_index = triangle_set.add_triangle_info(&second_triangle);

        // Sets the adjacency of the triangles that were adjacent to the original containing triangle
        if let Some(adjacent_triangle) = first_triangle.adjacent_triangle_indices[1] {
            triangle_set.replace_adjacent(
                adjacent_triangle,
                Some(containing_triangle_index),
                Some(first_triangle_index),
            )
        }
        if let Some(adjacent_triangle) = second_triangle.adjacent_triangle_indices[1] {
            triangle_set.replace_adjacent(
                adjacent_triangle,
                Some(containing_triangle_index),
                Some(second_triangle_index),
            )
        }

        // 5.1: Transform containing triangle into the third
        // Original triangle is transformed into the third triangle after the point has split the containing triangle into 3
        // using that triangle to keep main, so that the least has to change
        containing_triangle.vertex_indices[0] = inserted_point_index;
        containing_triangle.adjacent_triangle_indices[0] = Some(first_triangle_index);
        containing_triangle.adjacent_triangle_indices[2] = Some(second_triangle_index);
        triangle_set.replace_triangle(containing_triangle_index, &containing_triangle);

        // TODO there might be a good capacity to choose here
        let adjacent_triangle = Vec::<Triangle>::new();
        // REWORK THIS
        // 6: Add new triangles to a stack
        if let Some(adjacent_index) = containing_triangle.adjacent_triangle_indices[1] {
            adjacent_triangle.push(triangle_set.get_triangle(adjacent_index));
        }

        if let Some(adjacent_index) = first_triangle.adjacent_triangle_indices[1] {
            adjacent_triangle.push(triangle_set.get_triangle(adjacent_index));
        }

        if let Some(adjacent_index) = second_triangle.adjacent_triangle_indices[1] {
            adjacent_triangle.push(triangle_set.get_triangle(adjacent_index));
        }
        // 7.1: Check Delaunay constraint
        while let Some(triangle_to_check) = adjacent_triangle.pop() {
            if is_point_inside_circumcircle(triangle_to_check, point_to_insert) {
                triangle_set.get_triangle(idx)
                // get adjacent_triangles vom adjacent triangle, welche nicht am anliegenden edge sind und packe sie auf den stapel
                // speicher die jeweiligen kanten mit dazu zu den adjacent triangles (oder nur die kanten)
                // swap diagonal (diagnal edge + 2 ist der gegenüberliegende punkt... also eigentlich immer?)
                // repeat
            }
        }

        DelaunayTriangulation::fulfill_delaunay_constraint(
            triangle_set,
            adjacent_triangle,
            adjacent_triangle_edges,
        );

        return Some(inserted_point_index);
    } else {
        return None;
    }
}

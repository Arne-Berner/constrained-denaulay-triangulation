//TODO ADD TESTS FOR EVERY FUNCTION (in docs)
use crate::{
    data_structures::{
        error::CustomError, found_or_added::FoundOrAdded, point_bin_grid::PointBinGrid,
        triangle::Triangle, triangle_info::TriangleInfo, triangle_set::TriangleSet, vector::Vector,
    },
    hole_creation::{create_holes, get_supertriangle_triangles},
    math_utils::is_point_inside_circumcircle,
    normalize::{self, denormalize_points, normalize_points, Bounds},
};

pub struct TriangleIndexPair {
    pub adjacent: usize,
    pub current: usize,
}
impl TriangleIndexPair {
    fn new(adjacent: usize, current: usize) -> Self {
        TriangleIndexPair { adjacent, current }
    }
}

// TODO create a wrapper for the triangulate function, so that only that is visible to users of the library

/// This will triangulate any polygon using the delaunay constraint
///
/// You may provide input points in the given vector type, which will be used to create the triangulated polygon.
/// Then you can use optionally a vec of holes to create holes in the polygon mentioned above.
/// At least you can tesselate the area so that it may only contain triangles of the maximum area size given.
/// # Examples
/// This example uses an easy convex polygon.
/// ```
/// let mut input_points = Vec::new();
/// input_points.push(Vector::new(-0., 7.0));
/// input_points.push(Vector::new(-5., 5.));
/// input_points.push(Vector::new(5., 5.));
/// input_points.push(Vector::new(-2., 3.));
/// input_points.push(Vector::new(3., 1.));
/// input_points.push(Vector::new(-4., -1.));
/// input_points.push(Vector::new(1., -2.));
/// input_points.push(Vector::new(-6., -4.));
/// input_points.push(Vector::new(5., -4.));
/// let (triangle_set, bounds) = match triangulation::triangulate(&mut input_points, None, None){
///     Ok(result) => result,
///     Err(err) => panic!("triangulation failed!{:?}", err),
/// };
/// assert!(triangle_set.triangle_count() > 0);
/// ```
/// Even more complex are no problem either. (such as with collinear lines to the super triangle and each other.)
/// ```
/// let mut input_points = Vec::new();
/// input_points.push(Vector::new(1., 1.));
/// input_points.push(Vector::new(3., 4.));
/// input_points.push(Vector::new(-2., 3.));
/// input_points.push(Vector::new(-2., 3.));
/// input_points.push(Vector::new(-2.,-2.));
/// input_points.push(Vector::new(-1., -1.));
/// input_points.push(Vector::new(-2.,-3.));
/// input_points.push(Vector::new(4.,-2.));
/// let (triangle_set, bounds) = match triangulation::triangulate(&mut input_points, None, None){
///     Ok(result) => result,
///     Err(err) => panic!("triangulation failed!{:?}", err),
/// };
/// assert!(triangle_set.triangle_count() > 0);
/// ```
/// # Panics
/// The triangulation might panic if the holes are 50x the size of the polygon to be triangulated.
pub fn triangulate(
    input_points: &mut Vec<Vector>,
    holes: Option<&mut Vec<Vec<Vector>>>,
    maximum_triangle_area: Option<f32>,
) -> Result<Vec<Triangle>, CustomError> {
    // Initialize containers
    let mut triangle_set = TriangleSet::new(input_points.len() - 2);

    let (normalized_points, bounds) = normalize_points(input_points, None);

    // 2: Addition of points to the space partitioning grid
    let mut grid = PointBinGrid::new(
        //with 100 points that would result in 3 cells per side thus ~10 points per grid
        // which is the proposed overall_points^1/2 points per grid
        (input_points.len() as f32).powf(1. / 4.).round() as usize,
    );

    for point in &normalized_points {
        grid.add_point(*point);
    }

    // 3: Supertriangle initialization
    let supertriangle = Triangle::new(
        Vector::new(-100.0, -100.0),
        Vector::new(100.0, -100.0),
        Vector::new(0.0, 100.0),
    );
    triangle_set.add_triangle(&supertriangle);

    // 4: (loop over each point) For each point P in the list of sorted points, do steps 5-7
    // Points are added one at a time, and points that are close together are inserted together because they are sorted in the grid,
    // so a later step for finding their containing triangle is faster
    for cell in grid.cells().iter() {
        for point in cell {
            // All the points in the bin are added together, one by one
            match triangulate_point(&mut triangle_set, *point) {
                Ok(_) => (),
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }
    if let Some(maximum_triangle_area) = maximum_triangle_area {
        tesselate(&mut triangle_set, maximum_triangle_area)?;
    }

    let triangles;
    if let Some(holes) = holes {
        let triangles_to_remove = create_holes(&mut triangle_set, holes, bounds)?;
        triangle_set.points = denormalize_points(&mut triangle_set.points, &bounds);
        triangles = get_triangles_discarding_holes(&triangle_set, triangles_to_remove);
    } else {
        println!("{:#?}", triangle_set.triangle_infos);
        let mut triangles_to_remove = Vec::new();
        //get_supertriangle_triangles(&mut triangle_set, &mut triangles_to_remove);
        triangle_set.points = denormalize_points(&mut triangle_set.points, &bounds);
        triangles_to_remove.sort();
        triangles = get_triangles_discarding_holes(&triangle_set, triangles_to_remove);
    }

    return Ok(triangles);
}

fn tesselate(
    mut triangle_set: &mut TriangleSet,
    maximum_triangle_area: f32,
) -> Result<(), CustomError> {
    // skip Supertriangle
    let mut triangle_index = 2;
    while triangle_index < triangle_set.triangle_count() {
        // Skips  triangles sharing vertices with the Supertriangle
        let mut is_supertriangle = false;
        let triangle_info = triangle_set.get_triangle_info(triangle_index);

        for j in 0..3 {
            if triangle_info.vertex_indices[j] == 0
                || triangle_info.vertex_indices[j] == 1
                || triangle_info.vertex_indices[j] == 2
            {
                // 0, 1 and 2 are vertices of the supertriangle
                is_supertriangle = true;
                break;
            }
        }

        if is_supertriangle {
            continue;
        }

        let triangle = triangle_set.get_triangle(triangle_index);
        let triangle_area = crate::math_utils::calculate_triangle_area(&triangle);

        if triangle_area > maximum_triangle_area {
            if let Err(_) = triangulate_point(
                &mut triangle_set,
                triangle.p(0) + (triangle.p(1) - triangle.p(0)) * 0.5,
            ) {
                return Err(CustomError::TesselationFailed);
            }

            if let Err(_) = triangulate_point(
                &mut triangle_set,
                triangle.p(1) + (triangle.p(2) - triangle.p(1)) * 0.5,
            ) {
                return Err(CustomError::TesselationFailed);
            }

            if let Err(_) = triangulate_point(
                &mut triangle_set,
                triangle.p(2) + (triangle.p(0) - triangle.p(2)) * 0.5,
            ) {
                return Err(CustomError::TesselationFailed);
            }
            triangle_index = 2; // The tesselation restarts
        }
        triangle_index += 1;
    }
    return Ok(());
}

pub fn triangulate_point(
    triangle_set: &mut TriangleSet,
    point_to_insert: Vector,
) -> Result<FoundOrAdded, CustomError> {
    // Note: Adjacent triangle, opposite to the inserted point, is always at index 1
    // Note 2: Adjacent triangles are stored CCW automatically, their index matches the index of the first vertex in every edge, and it is known that vertices are stored CCW

    // 4.1: Check point existence
    let inserted_point_index;
    match triangle_set.add_point(point_to_insert) {
        FoundOrAdded::Found(idx) => return Ok(FoundOrAdded::Found(idx)),
        FoundOrAdded::Added(idx) => inserted_point_index = idx,
    }

    // 4.2: Search containing triangle
    // Start at the last added triangle
    if let Ok(containing_triangle_index) = triangle_set
        .find_triangle_that_contains_point(point_to_insert, triangle_set.triangle_count() - 1)
    {
        let mut containing_triangle = triangle_set.get_triangle_info(containing_triangle_index);

        // 5. Insert new point in triangulation and create 2 new triangles off of it
        // all the triangles take inserted point as there vertex 0, so that adjacent is 1
        let first_triangle = TriangleInfo::new([
            inserted_point_index,
            containing_triangle.vertex_indices[0],
            containing_triangle.vertex_indices[1],
        ])
        .with_adjacent(
            None,                                             // the second triangle
            containing_triangle.adjacent_triangle_indices[0], // the originals adjacent
            Some(containing_triangle_index), // this is the original triangle, that will get changed a bit
        );
        let first_triangle_index = triangle_set.add_triangle_info(first_triangle);

        let second_triangle = TriangleInfo::new([
            inserted_point_index,
            containing_triangle.vertex_indices[2],
            containing_triangle.vertex_indices[0],
        ])
        .with_adjacent(
            Some(containing_triangle_index),
            containing_triangle.adjacent_triangle_indices[2],
            Some(first_triangle_index),
        );

        let second_triangle_index = triangle_set.add_triangle_info(second_triangle);
        triangle_set.triangle_infos[first_triangle_index].adjacent_triangle_indices[0] =
            Some(second_triangle_index);

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
        triangle_set.triangle_infos[containing_triangle_index].vertex_indices[0] =
            inserted_point_index;
        triangle_set.triangle_infos[containing_triangle_index].adjacent_triangle_indices[0] =
            Some(first_triangle_index);
        triangle_set.triangle_infos[containing_triangle_index].adjacent_triangle_indices[2] =
            Some(second_triangle_index);

        // TODO there might be a good capacity to choose here
        let mut index_pairs = Vec::<TriangleIndexPair>::new();
        // 6: Add new triangles to a stack
        if let Some(adjacent_index) = containing_triangle.adjacent_triangle_indices[1] {
            index_pairs.push(TriangleIndexPair {
                adjacent: adjacent_index,
                current: containing_triangle_index,
            });
        }

        if let Some(adjacent_index) = first_triangle.adjacent_triangle_indices[1] {
            index_pairs.push(TriangleIndexPair {
                adjacent: adjacent_index,
                current: first_triangle_index,
            });
        }

        if let Some(adjacent_index) = second_triangle.adjacent_triangle_indices[1] {
            index_pairs.push(TriangleIndexPair {
                adjacent: adjacent_index,
                current: second_triangle_index,
            });
        }
        // 7.1: Check Delaunay constraint
        while let Some(index_pair) = index_pairs.pop() {
            if is_point_inside_circumcircle(
                triangle_set.get_triangle(index_pair.adjacent),
                point_to_insert,
            ) {
                // delaunay constraint not fullfilled
                if let Ok((first_new_adjacent, second_new_adjacent)) =
                    // 7.2
                    // TODO rewrite to Option<(usize, usize)>
                    swap_edges(&index_pair, triangle_set, 1)
                {
                    // 7.3 push new adjacents on stack
                    if let Some(new_oppositve_index) = second_new_adjacent {
                        index_pairs.push(TriangleIndexPair::new(
                            new_oppositve_index,
                            index_pair.adjacent,
                        ))
                    }
                    if let Some(new_opposite_index) = first_new_adjacent {
                        index_pairs.push(TriangleIndexPair::new(
                            new_opposite_index,
                            index_pair.current,
                        ))
                    }
                } else {
                    return Err(CustomError::SwappingFailed);
                }
            }
        }
        return Ok(FoundOrAdded::Added(inserted_point_index));
    } else {
        return Err(CustomError::PointNotInTriangle);
    }
}

pub fn swap_edges(
    index_pair: &TriangleIndexPair,
    triangle_set: &mut TriangleSet,
    shared_vertex_index: usize,
) -> Result<(Option<usize>, Option<usize>), CustomError> {
    let current_info = triangle_set.get_triangle_info(index_pair.current);
    let adjacent_info = triangle_set.get_triangle_info(index_pair.adjacent);
    let p = current_info.vertex_indices[(shared_vertex_index + 2) % 3];
    let p2 = current_info.vertex_indices[(shared_vertex_index + 1) % 3];
    // assumption (needs the FIRST shared vertex of current)
    let shared_vertex = current_info.vertex_indices[shared_vertex_index];
    let mut adj_shared_vertex_index = 4; // out of bounds
    for idx in 0..3 {
        if shared_vertex == adjacent_info.vertex_indices[idx] {
            adj_shared_vertex_index = idx;
            break;
        }
    }
    if adj_shared_vertex_index > 2 {
        return Err(CustomError::TrianglesDontShareIndex);
    }
    let first_new_adjacent = adjacent_info.adjacent_triangle_indices[adj_shared_vertex_index];
    let second_new_adjacent =
        adjacent_info.adjacent_triangle_indices[(adj_shared_vertex_index + 1) % 3];

    let opposite_vertex = adjacent_info.vertex_indices[(adj_shared_vertex_index + 1) % 3];
    let a2 = current_info.adjacent_triangle_indices[(shared_vertex_index + 1) % 3];
    let new_adjacent = TriangleInfo::new([
        // Assumption
        p,
        opposite_vertex,
        p2,
    ])
    .with_adjacent(Some(index_pair.current), second_new_adjacent, a2);
    triangle_set.replace_triangle(index_pair.adjacent, &new_adjacent);
    let new_current = TriangleInfo::new([
        // Assumption
        p,
        shared_vertex,
        opposite_vertex,
    ])
    .with_adjacent(
        current_info.adjacent_triangle_indices[(shared_vertex_index + 2) % 3],
        first_new_adjacent,
        Some(index_pair.adjacent),
    );
    triangle_set.replace_triangle(index_pair.current, &new_current);
    // change the adjacent triangles of the changed adjacent triangles
    
    if let Some(needs_replacement_index) = first_new_adjacent {
        triangle_set.replace_adjacent(
            needs_replacement_index,
            Some(index_pair.adjacent),
            Some(index_pair.current),
        );
    }
    if let Some(needs_replacement_index) = a2 {
        triangle_set.replace_adjacent(
            needs_replacement_index,
            Some(index_pair.current),
            Some(index_pair.adjacent),
        );
    }
    Ok((first_new_adjacent, second_new_adjacent))
}

fn get_triangles(triangle_set: &TriangleSet) -> Vec<Triangle> {
    let mut output_triangles = Vec::with_capacity(triangle_set.triangle_count() - 1);
    for triangle_info in &triangle_set.triangle_infos {
        output_triangles.push(Triangle::new(
            triangle_set.get_point(triangle_info.vertex_indices[0]),
            triangle_set.get_point(triangle_info.vertex_indices[1]),
            triangle_set.get_point(triangle_info.vertex_indices[2]),
        ))
    }
    output_triangles
}

fn get_triangles_discarding_holes(
    triangle_set: &TriangleSet,
    triangles_to_remove: Vec<usize>,
) -> Vec<Triangle> {
    let mut output_triangles = Vec::with_capacity(triangle_set.triangle_count() - 1);

    // Output filtering
    let mut idxs_i = 0;

    for (idx, triangle_info) in triangle_set.triangle_infos.iter().enumerate() {
        if !(triangles_to_remove.get(idxs_i) == Some(&idx)) {
            output_triangles.push(Triangle::new(
                triangle_set.get_point(triangle_info.vertex_indices[0]),
                triangle_set.get_point(triangle_info.vertex_indices[1]),
                triangle_set.get_point(triangle_info.vertex_indices[2]),
            ));
        } else {
            idxs_i += 1;
        }
    }
    output_triangles
}

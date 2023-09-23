use crate::data_structures::{triangle::Triangle, vector::Vector};
/// Calculates the determinant of a 3 columns x 3 rows matrix.
///
/// # Arguments
///
/// * `m00` - The element at position (0, 0).
/// * `m10` - The element at position (1, 0).
/// * `m20` - The element at position (2, 0).
/// * `m01` - The element at position (0, 1).
/// * `m11` - The element at position (1, 1).
/// * `m21` - The element at position (2, 1).
/// * `m02` - The element at position (0, 2).
/// * `m12` - The element at position (1, 2).
/// * `m22` - The element at position (2, 2).
///
/// # Returns
///
/// The determinant.
pub fn calculate_matrix3x3_determinant(
    m00: f32,
    m10: f32,
    m20: f32,
    m01: f32,
    m11: f32,
    m21: f32,
    m02: f32,
    m12: f32,
    m22: f32,
) -> f32 {
    m00 * m11 * m22 + m10 * m21 * m02 + m20 * m01 * m12
        - m20 * m11 * m02
        - m10 * m01 * m22
        - m00 * m21 * m12
}

/// Checks whether a point lies on the right side of an edge.
///
/// # Arguments
///
/// * `edge_endpoint_a` - The first point of the edge.
/// * `edge_endpoint_b` - The second point of the edge.
/// * `point` - The point to check.
///
/// # Returns
///
/// `true` if the point is on the right side; `false` if the point is on the left side or is contained in the edge.
pub fn is_point_to_the_right_of_edge(
    edge_endpoint_a: Vector,
    edge_endpoint_b: Vector,
    point: Vector,
) -> bool {
    let p1 = edge_endpoint_b.x - edge_endpoint_a.x;
    let p2 = point.y - edge_endpoint_a.y;
    let p3 = edge_endpoint_b.y - edge_endpoint_a.y;
    let p4 = point.x - edge_endpoint_a.x;
    let determinante = p1 * p2 - p3 * p4;
    determinante < -0.00000001 // Note: Due to extremely small negative values causing wrong results, a tolerance is used instead of zero
}

/// Checks whether a point lies on the left side of an edge.
///
/// # Arguments
///
/// * `edge_endpoint_a` - The first point of the edge.
/// * `edge_endpoint_b` - The second point of the edge.
/// * `point` - The point to check.
///
/// # Returns
///
/// `true` if the point is on the left side; `false` if the point is on the right side or is contained in the edge.
pub fn is_point_to_the_left_of_edge(
    edge_endpoint_a: Vector,
    edge_endpoint_b: Vector,
    point: Vector,
) -> bool {
    !is_point_to_the_right_of_edge(edge_endpoint_a, edge_endpoint_b, point)
}

/// Checks whether a point is contained in a triangle. The vertices of the triangle must be sorted counter-clockwise.
///
/// # Arguments
///
/// * `triangle_p0` - The first vertex of the triangle.
/// * `triangle_p1` - The second vertex of the triangle.
/// * `triangle_p2` - The third vertex of the triangle.
/// * `point_to_check` - The point that may be contained.
///
/// # Returns
///
/// Returns true if the point is contained in the triangle; false otherwise.
pub fn is_point_inside_triangle(
    triangle_p0: Vector,
    triangle_p1: Vector,
    triangle_p2: Vector,
    point_to_check: Vector,
) -> bool {
    is_point_to_the_left_of_edge(triangle_p0, triangle_p1, point_to_check)
        && is_point_to_the_left_of_edge(triangle_p1, triangle_p2, point_to_check)
        && is_point_to_the_left_of_edge(triangle_p2, triangle_p0, point_to_check)
}

// https://gamedev.stackexchange.com/questions/71328/how-can-i-add-and-subtract-convex-polygons
pub fn is_point_inside_circumcircle(triangle: Triangle, point_to_check: Vector) -> bool {
    // sloan algorithm
    let x02 = triangle.p(0).x - triangle.p(2).x;
    let x12 = triangle.p(1).x - triangle.p(2).x;
    let x0p = triangle.p(0).x - point_to_check.x;
    let x1p = triangle.p(1).x - point_to_check.x;
    let y02 = triangle.p(0).y - triangle.p(2).y;
    let y12 = triangle.p(1).y - triangle.p(2).y;
    let y0p = triangle.p(0).y - point_to_check.y;
    let y1p = triangle.p(1).y - point_to_check.y;

    let cosa = x02 * x12 + y02 * y12;
    let cosb = x0p * x1p + y0p * y1p;

    if cosa >= 0. && cosb >= 0. {
        return false;
    }
    if cosa < 0. && cosb < 0. {
        return true;
    }

    let sina = x02 * y12 - x12 * y02;
    let sinb = x1p * y0p - x0p * y1p;
    if sina * cosb + sinb * cosa < 0. {
        return true;
    }
    false
}

/// Calculates whether 2 line segments intersect and the intersection point.
///
/// # Arguments
///
/// * `endpointA1` - The first point of the first segment.
/// * `endpointB1` - The second point of the first segment.
/// * `endpointA2` - The first point of the second segment.
/// * `endpointB2` - The second point of the second segment.
/// * `intersectionPoint` - The intersection point, if any.
///
/// # Returns
///
/// Returns true if the line segments intersect; false otherwise.
pub fn intersection_between_lines(
    endpoint_a1: Vector,
    endpoint_b1: Vector,
    endpoint_a2: Vector,
    endpoint_b2: Vector,
) -> Option<Vector> {
    // https://stackoverflow.com/questions/4543506/algorithm-for-intersection-of-2-lines
    let mut intersection_point = Vector::new(f32::MAX, f32::MAX);

    let is_line1_vertical = endpoint_b1.x == endpoint_a1.x;
    let is_line2_vertical = endpoint_b2.x == endpoint_a2.x;

    let mut x = f32::MAX;
    let mut y = f32::MAX;

    if is_line1_vertical && !is_line2_vertical {
        // First it calculates the standard form (Ax + By = C)
        let m2 = (endpoint_b2.y - endpoint_a2.y) / (endpoint_b2.x - endpoint_a2.x);

        let c2 = endpoint_a2.x * m2 - endpoint_a2.y;

        x = endpoint_a1.x;
        y = m2 * endpoint_a1.x - c2;
    } else if is_line2_vertical && !is_line1_vertical {
        // First it calculates the standard form (Ax + By = C)
        let m1 = (endpoint_b1.y - endpoint_a1.y) / (endpoint_b1.x - endpoint_a1.x);

        let c1 = endpoint_a1.x * m1 - endpoint_a1.y;

        x = endpoint_a2.x;
        y = m1 * endpoint_a2.x - c1;
    } else if !is_line1_vertical && !is_line2_vertical {
        let m1 = (endpoint_b1.y - endpoint_a1.y) / (endpoint_b1.x - endpoint_a1.x);

        let b1 = -1.0;
        let c1 = endpoint_a1.x * m1 - endpoint_a1.y;

        let m2 = (endpoint_b2.y - endpoint_a2.y) / (endpoint_b2.x - endpoint_a2.x);

        let b2 = -1.0;
        let c2 = endpoint_a2.x * m2 - endpoint_a2.y;

        let determinant = m1 * b2 - m2 * b1;

        if determinant == 0.0 {
            // Lines do not intersect
            return None;
        }

        x = (b2 * c1 - b1 * c2) / determinant;
        y = (m1 * c2 - m2 * c1) / determinant;
    }

    // Checks whether the point is in the segment determined by the endpoints of both lines
    if x <= endpoint_a1.x.max(endpoint_b1.x)
        && x >= endpoint_a1.x.min(endpoint_b1.x)
        && y <= endpoint_a1.y.max(endpoint_b1.y)
        && y >= endpoint_a1.y.min(endpoint_b1.y)
        && x <= endpoint_a2.x.max(endpoint_b2.x)
        && x >= endpoint_a2.x.min(endpoint_b2.x)
        && y <= endpoint_a2.y.max(endpoint_b2.y)
        && y >= endpoint_a2.y.min(endpoint_b2.y)
    {
        intersection_point.x = x;
        intersection_point.y = y;
        Some(intersection_point)
    } else {
        None
    }
}

// TODO this is never used
pub fn is_triangle_vertices_cw(point0: Vector, point1: Vector, point2: Vector) -> bool {
    calculate_matrix3x3_determinant(
        point0.x, point0.y, 1.0, point1.x, point1.y, 1.0, point2.x, point2.y, 1.0,
    ) < 0.0
}

// TODO this is never used
pub fn is_quadrilateral_convex(a: Vector, b: Vector, c: Vector, d: Vector) -> bool {
    let abc = is_triangle_vertices_cw(a, b, c);
    let abd = is_triangle_vertices_cw(a, b, d);
    let bcd = is_triangle_vertices_cw(b, c, d);
    let cad = is_triangle_vertices_cw(c, a, d);

    let mut is_convex = false;

    if abc && abd && bcd && !cad {
        is_convex = true;
    } else if abc && abd && !bcd && cad {
        is_convex = true;
    } else if abc && !abd && bcd && cad {
        is_convex = true;
    } else if !abc && !abd && !bcd && cad {
        is_convex = true;
    } else if !abc && !abd && bcd && !cad {
        is_convex = true;
    } else if !abc && abd && !bcd && !cad {
        is_convex = true;
    }

    is_convex
}

/// Calculates the area of a triangle, according to its 3 vertices.
///
/// It does not matter whether the vertices are sorted counter-clockwise.
///
/// # Arguments
///
/// * `p0` - The first vertex.
/// * `p1` - The second vertex.
/// * `p2` - The third vertex.
///
/// # Returns
///
/// The area of the triangle.
pub fn calculate_triangle_area(triangle: &Triangle) -> f32 {
    (triangle.p(1) - triangle.p(0)).cross_product(triangle.p(2) - triangle.p(0)) * 0.5
}

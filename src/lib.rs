#[warn(missing_docs)]
// do they need pub use?
use data_structures::vector::Vector;
use data_structures::{error::CustomError, triangle::Triangle};

mod triangulation;
mod data_structures;
mod math_utils;
mod normalize;
mod hole_creation;

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
/// let triangles = match triangulate(&mut input_points, None, None){
///     Ok(result) => result,
///     Err(err) => panic!("triangulation failed!{:?}", err),
/// };
/// assert!(triangle.len() > 0);
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
/// let triangles = match triangulate(&mut input_points, None, None){
///     Ok(result) => result,
///     Err(err) => panic!("triangulation failed!{:?}", err),
/// };
/// assert!(triangles.len() > 0);
/// ```
/// # Panics
/// The triangulation might panic if the holes are 50x the size of the polygon to be triangulated.
pub fn triangulate(input_points:&mut Vec<Vector>, holes:Option<&mut Vec<Vec<Vector>>>, maximum_triangle_area:Option<f32>)-> Result<Vec<Triangle>, CustomError> {
    Ok(triangulation::triangulate(input_points, holes, maximum_triangle_area)?)
}

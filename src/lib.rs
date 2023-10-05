#[warn(missing_docs)]
// do they need pub use?
pub use data_structures::vector::Vector;
pub use data_structures::{error::CustomError, triangle::Triangle};

mod data_structures;
mod hole_creation;
mod math_utils;
mod normalize;
mod triangulation;

/// This will triangulate any polygon using the delaunay constraint
///
/// You may provide input points in the given vector type, which will be used to create the triangulated polygon.
/// Then you can use optionally a vec of holes to create holes in the polygon mentioned above.
/// At least you can tesselate the area so that it may only contain triangles of the maximum area size given.
/// # Examples
/// This example uses an easy convex polygon.
/// ```
/// use constrained_denaulay_triangulation::{triangulate, Vector};
///
/// fn main() {
/// let mut input_points = vec![
///     (0., 7.),
///     (-5., 5.),
///     (5., 5.),
///     (-1., 3.),
///     (3., 1.),
///     (-4., -1.),
///     (1., -2.),
///     (-6., -4.),
///     (5., -4.),
/// ]
/// .iter()
/// .map(|x| Vector::from(x))
/// .collect::<Vec<Vector>>();

/// let mut holes: Vec<Vec<Vector>> = vec![];
/// let minihole = vec![(-1.5, 3.5), (-0.5, 3.5), (-1., 2.5)]
///     .iter()
///     .map(|x| Vector::from(x))
///     .collect::<Vec<Vector>>();
/// holes.push(minihole);

/// let bighole = vec![(-4., 4.), (0., -2.), (4., 4.)]
///     .iter()
///     .map(|x| Vector::from(x))
///     .collect::<Vec<Vector>>();
/// holes.push(bighole);

/// let input_hole = Some(&mut holes);

/// let a = match triangulate(&mut input_points, input_hole, None) {
///     Ok(result) => result,
///     Err(err) => panic!("triangulation failed!{:?}", err),
/// };
/// assert!(a.len() > 0);
/// }
///
/// ```
/// Even more complex are no problem either. (such as with collinear lines to the super triangle and each other.)
/// ```
/// use constrained_denaulay_triangulation::{triangulate, Vector};
///
/// fn main() {
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
/// }
/// ```
/// # Panics
/// The triangulation might panic if the holes are 50x the size of the polygon to be triangulated.
/// # Known limitations
/// The function will not work with holes that are bigger than the point cloud or outside of the point cloud
pub fn triangulate(
    input_points: &mut Vec<Vector>,
    holes: Option<&mut Vec<Vec<Vector>>>,
    maximum_triangle_area: Option<f32>,
) -> Result<Vec<Triangle>, CustomError> {
    Ok(triangulation::triangulate(
        input_points,
        holes,
        maximum_triangle_area,
    )?)
}

fn test() {
/// let mut input_points = vec![
///     (0., 7.),
///     (-5., 5.),
///     (5., 5.),
///     (-1., 3.),
///     (3., 1.),
///     (-4., -1.),
///     (1., -2.),
///     (-6., -4.),
///     (5., -4.),
/// ]
/// .iter()
/// .map(|x| Vector::from(x))
/// .collect::<Vec<Vector>>();

/// let mut holes: Vec<Vec<Vector>> = vec![];
/// let minihole = vec![(-1.5, 3.5), (-0.5, 3.5), (-1., 2.5)]
///     .iter()
///     .map(|x| Vector::from(x))
///     .collect::<Vec<Vector>>();
/// holes.push(minihole);

/// let bighole = vec![(-4., 4.), (0., -2.), (4., 4.)]
///     .iter()
///     .map(|x| Vector::from(x))
///     .collect::<Vec<Vector>>();
/// holes.push(bighole);

/// let input_hole = Some(&mut holes);

/// let a = match triangulate(&mut input_points, input_hole, None) {
///     Ok(result) => result,
///     Err(err) => panic!("triangulation failed!{:?}", err),
/// };
/// assert!(a.len() > 0);
}

#[warn(missing_docs)]
use data_structures::vector::Vector;

mod data_structures;
mod math_utils;
mod normalize;
mod triangulation;
mod hole_creation;

fn main() {
    let mut input_points = Vec::new();
    input_points.push(Vector::new(1., 1.));
    input_points.push(Vector::new(3., 4.));
    input_points.push(Vector::new(-2., 3.));
    input_points.push(Vector::new(-2., 3.));
    input_points.push(Vector::new(-2., -2.));
    input_points.push(Vector::new(-1., -1.));
    input_points.push(Vector::new(-2., -3.));
    input_points.push(Vector::new(4., -2.));
    let (triangle_set, bounds) = match triangulation::triangulate(&mut input_points, None, None) {
        Ok(result) => result,
        Err(err) => panic!("triangulation failed!{:?}", err),
    };
    assert!(triangle_set.triangle_count() > 0);
    println!("Bounds: {:?}", bounds);
}

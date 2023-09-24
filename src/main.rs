#[warn(missing_docs)]
use data_structures::vector::Vector;

mod data_structures;
mod hole_creation;
mod math_utils;
mod normalize;
mod triangulation;

fn main() {
    let mut input_points = Vec::new();
    input_points.push(Vector::new(-0., 7.0));
    input_points.push(Vector::new(-5., 5.));
    input_points.push(Vector::new(5., 5.));
    input_points.push(Vector::new(-2., 3.));
    input_points.push(Vector::new(3., 1.));
    input_points.push(Vector::new(-4., -1.));
    input_points.push(Vector::new(1., -2.));
    input_points.push(Vector::new(-6., -4.));
    input_points.push(Vector::new(5., -4.));

    let mut holes = vec![Vec::<Vector>::new()];
    let mut hole = vec![];
    hole.push(Vector::new(-1., -1.));
    hole.push(Vector::new(1., -1.));
    hole.push(Vector::new(0., 1.));

    let (triangle_set, bounds) = match triangulation::triangulate(&mut input_points, Some(&mut holes), None) {
        Ok(result) => result,
        Err(err) => panic!("triangulation failed!{:?}", err),
    };
    assert!(triangle_set.triangle_count() > 0);
    println!("Bounds: {:?}", bounds);
}

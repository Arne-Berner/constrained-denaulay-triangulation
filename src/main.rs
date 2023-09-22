use data_structures::vec2::Vec2;

mod data_structures;
mod math_utils;
mod normalize;
mod triangulation;

fn main() {
    let mut input_points = Vec::new();
    input_points.push(Vec2::new(-0., 7.0));
    input_points.push(Vec2::new(-5., 5.));
    input_points.push(Vec2::new(5., 5.));
    input_points.push(Vec2::new(-2., 3.));
    input_points.push(Vec2::new(3., 1.));
    input_points.push(Vec2::new(-4., -1.));
    input_points.push(Vec2::new(1., -2.));
    input_points.push(Vec2::new(-6., -4.));
    input_points.push(Vec2::new(5., -4.));
    let (triangle_set, bounds) = match triangulation::triangulate(&mut input_points, None, None){
        Ok(result) => result,
        Err(err) => panic!("triangulation failed!{:?}", err),
    };
    assert!(triangle_set.triangle_count() > 0);
    println!("Bounds: {:?}", bounds);
}

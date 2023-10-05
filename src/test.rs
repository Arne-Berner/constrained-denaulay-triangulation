fn main() {
    let input_points = vec![
        (0., 7.),
        (-5., 5.),
        (5., 5.),
        (-1., 3.),
        (3., 1.),
        (-4., -1.),
        (1., -2.),
        (-6., -4.),
        (5., -4.),
    ].iter().map(|x| Vec::new(x)).collect::<Vec<Vector>>();
    let mut input_points = Vec::new();
    input_points.push(Vector::new(-0., 7.0) * 10.); //
    input_points.push(Vector::new(-5., 5.) * 10.); //
    input_points.push(Vector::new(5., 5.) * 10.); //
    input_points.push(Vector::new(-1., 3.) * 10.); //
    input_points.push(Vector::new(3., 1.) * 10.); //
    input_points.push(Vector::new(-4., -1.) * 10.); //
    input_points.push(Vector::new(1., -2.) * 10.); //
    input_points.push(Vector::new(-6., -4.) * 10.); //
    input_points.push(Vector::new(5., -4.) * 10.); //
    let mut holes: Vec<Vec<Vector>> = vec![];
    let mut minihole = Vec::<Vector>::new();
    minihole.push(Vector::new(-1.5, 3.5) * 10.);
    minihole.push(Vector::new(-0.5, 3.5) * 10.);
    minihole.push(Vector::new(-1., 2.5) * 10.);
    holes.push(minihole);
    let mut bighole = Vec::<Vector>::new();
    bighole.push(Vector::new(-4., 4.) * 10.);
    bighole.push(Vector::new(0., -2.) * 10.);
    bighole.push(Vector::new(4., 4.) * 10.);
    holes.push(bighole);
    let input_hole = Some(&mut holes);

    let a = match triangulate(&mut input_points, input_hole, None) {
        Ok(result) => result,
        Err(err) => panic!("triangulation failed!{:?}", err),
    };
    assert!(a.len() > 0);
}

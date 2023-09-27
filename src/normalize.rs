use crate::data_structures::vector::Vector;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Bounds {
    min: Vector,
    max: Vector,
}

/// Takes vectors and normalizes them, either using their own bounds or the given bounds. Also outputs their original minimal x and y vector as a value and their maximum x and y vector. 
/// ```
/// 
pub fn normalize_points(points: &mut Vec<Vector>, bounds: Option<Bounds>) -> (Vec<Vector>, Bounds) {
    let bounds = if let Some(bounds) = bounds {
        bounds
    } else {
        let mut min = Vector::new(f32::MAX, f32::MAX);
        let mut max = Vector::new(f32::MIN, f32::MIN);

        for i in 0..points.len() {
            if points[i].x > max.x {
                max.x = points[i].x;
            }

            if points[i].y > max.y {
                max.y = points[i].y;
            }

            if points[i].x < min.x {
                min.x = points[i].x;
            }

            if points[i].y < min.y {
                min.y = points[i].y;
            }
        }
        Bounds { min, max }
    };

    let points = points
        .iter()
        .map(|point| (*point - bounds.min) / (bounds.max - bounds.min))
        .collect::<Vec<_>>();
    (points, bounds)
}

pub fn denormalize_points(input_points: &mut Vec<Vector>, bounds: &Bounds)->Vec<Vector>{
    input_points.iter().map(|point| (*point * (bounds.max - bounds.min) + bounds.min)).collect()
}

#[test]
fn test_normalize(){
    let mut input_points = Vec::new();
    input_points.push(Vector::new(-0., 5.0)); 
    input_points.push(Vector::new(-5., 0.)); 
    input_points.push(Vector::new(5., -5.)); 
    let output = normalize_points(&mut input_points, None);

    let expected_bounds = Bounds{min: Vector::new(-5., -5.), max:Vector::new(5.,5.)};
    let mut expected_points= Vec::new();
    expected_points.push(Vector::new(0.5, 1.)); 
    expected_points.push(Vector::new(0., 0.5)); 
    expected_points.push(Vector::new(1., 0.)); 
    assert_eq!(output, (expected_points, expected_bounds));
}

#[test]
fn test_normalize_with_bounds(){
    let bounds = Bounds{min: Vector::new(-10., -10.), max:Vector::new(10.,10.)};
    let mut input_points = Vec::new();
    input_points.push(Vector::new(-0., 5.0)); 
    input_points.push(Vector::new(-5., 0.)); 
    input_points.push(Vector::new(5., -5.)); 
    let output = normalize_points(&mut input_points, Some(bounds));

    let expected_bounds = Bounds{min: Vector::new(-10., -10.), max:Vector::new(10.,10.)};
    let mut expected_points= Vec::new();
    expected_points.push(Vector::new(0.5, 0.75)); 
    expected_points.push(Vector::new(0.25, 0.5)); 
    expected_points.push(Vector::new(0.75, 0.25)); 
    assert_eq!(output, (expected_points, expected_bounds));
}
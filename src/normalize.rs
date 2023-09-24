use crate::data_structures::vector::Vector;

// TODO add tests that check bounds
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Bounds {
    min: Vector,
    max: Vector,
}
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

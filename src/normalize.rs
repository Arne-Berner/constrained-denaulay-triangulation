use crate::data_structures::vec2::Vec2;

pub struct Bounds {
    min: Vec2,
    max: Vec2,
}
pub fn normalize_points(points: &mut Vec<Vec2>, bounds: Option<Bounds>) -> (&Vec<Vec2>, Bounds) {
    let bounds = if let Some(bounds) = bounds {
        bounds
    } else {
        let mut min = Vec2::new(f32::MAX, f32::MAX);
        let mut max = Vec2::new(f32::MIN, f32::MIN);

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

    let points = points.iter().map(|point| (*point-bounds.min) / (bounds.max -bounds.min)).collect();
    (points, bounds)
}

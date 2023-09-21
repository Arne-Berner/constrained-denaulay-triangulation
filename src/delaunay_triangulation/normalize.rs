use bevy::prelude::Vec2;

pub fn normalize_points(points: Vec<Vec2>) -> Vec<Vec2>{
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

    for point in &points {
        *point = (*point - min) / (max - min)
    }
    points
}

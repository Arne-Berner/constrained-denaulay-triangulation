pub mod point_bin_grid;
pub mod triangle_set;
pub mod math_utils;
pub mod delaunay_triangulation;
use bevy::{prelude::*, render::primitives::Aabb};
use delaunay_triangulation::DelaunayTriangulation;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (system, update_config))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    // text
    commands.spawn(TextBundle::from_section(
        "Hold 'Left' or 'Right' to change the line width",
        TextStyle {
            font_size: 24.,
            color: Color::WHITE,
            ..default()
        },
    ));
    let triangulation = DelaunayTriangulation::new(100, Vec2::new(1., 1.)* 50000., None, None, vec![], None, Aabb::from_min_max(Vec3::new(-1.,1.,-1.)*10000., Vec3::new(1., 1., 1.)*10000.));
let p0 = Vec2::new(-6189.595, 8209.541);
let p1 = Vec2::new(-5733.924, 8823.252);
let p2 = Vec2::new(-5748.702, 8231.538);
let p3 = Vec2::new(-5687.935, 7984.462);
let p4 = Vec2::new(-6189.595, 7709.541);
let p5 = Vec2::new(-6893.049, 7682.856);
let p6 = Vec2::new(-6759.087, 8353.894);
let p7 = Vec2::new(-6763.313, 8504.048);
let p8 = Vec2::new(-6284.938, 8771.748);
let points = vec![p0, p1, p2, p3, p4, p5, p6, p7, p8];
let hole1 = vec![Vec2::new(1., 1.), Vec2::new(1.,2.),Vec2::new(2.,2.), Vec2::new(2., 1.)];
let holes = vec![hole1];
    triangulation.triangulate(&points, 0., None);
}


fn system(mut gizmos: Gizmos) {
    let _adjacent_triangles = ();
    let _adjacent_edges = ();
    let points = [
        (Vec2::new(1., 0.) * 100., Color::WHITE),
        (Vec2::new(0., 1.5) * 100., Color::WHITE),
        (Vec2::new(-1., 1.) * 100., Color::WHITE),
        (Vec2::new(-1., -1.) * 100., Color::WHITE),
        (Vec2::new(1., 0.) * 100., Color::WHITE),
    ];
    gizmos.line_gradient_2d(Vec2::Y, Vec2::splat(-80.), Color::RED, Color::SILVER);

    //pointcloud
    gizmos.linestrip_gradient_2d(points);
}

fn update_config(mut config: ResMut<GizmoConfig>, keyboard: Res<Input<KeyCode>>, time: Res<Time>) {
    if keyboard.pressed(KeyCode::Right) {
        config.line_width += 5. * time.delta_seconds();
    }
    if keyboard.pressed(KeyCode::Left) {
        config.line_width -= 5. * time.delta_seconds();
    }
}

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
    let triangulation = DelaunayTriangulation::new(10, Vec2::new(10., 10.), None, None, vec![], None, Aabb::from_min_max(Vec3::new(-10.,0.,-10.), Vec3::new(10., 0., 10.)));
let points = vec![Vec2::new(0.,0.), Vec2::new(0., 4.), Vec2::new(4.,4.), Vec2::new(4.,0.)];
let hole1 = vec![Vec2::new(1., 1.), Vec2::new(1.,2.),Vec2::new(2.,2.), Vec2::new(2., 1.)];
let holes = vec![hole1];
    triangulation.triangulate(&points, 0., Some(&holes));
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

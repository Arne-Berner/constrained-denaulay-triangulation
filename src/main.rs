pub mod point_bin_grid;
pub mod triangle_set;
pub mod math_utils;
pub mod delaunay_triangulation;
use bevy::prelude::*;

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

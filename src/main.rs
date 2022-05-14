use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_startup_system(spawn_camera)
        .add_system(mouse_position)
        .add_system(spawn_rectangle)
        .add_system(mouse_motion)
        //.add_system(camera_movement)
        .add_system(camera_zoom)
        .init_resource::<MouseMovement>()
        .run();
}

#[derive(Default)]
struct MouseMovement {
    position: Vec2,
    normalized: Vec2,
}

#[derive(Component)]
struct Moving {
    origin: Vec2,
}
fn spawn_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}
fn spawn_rectangle(
    mut commands: Commands,
    mut windows: ResMut<Windows>,
    mouse_input: Res<Input<MouseButton>>,
    mut query: Query<(&mut DrawMode, Entity), With<Moving>>,
    mouse: Res<MouseMovement>,
) {
    let window = windows.get_primary_mut().unwrap();
    if mouse_input.just_pressed(MouseButton::Left) {
        let rect = shapes::Rectangle {
            extents: Vec2::ZERO,
            ..default()
        };
        commands
            .spawn_bundle(GeometryBuilder::build_as(
                &rect,
                DrawMode::Fill(FillMode::color(Color::WHITE)),
                Transform::from_translation(mouse.position.extend(0.0)),
            ))
            .insert(Moving {
                origin: mouse.position,
            });
        window.set_cursor_lock_mode(true);
    }

    if mouse_input.just_released(MouseButton::Left) {
        let (mut rectangle_draw_mode, id) = query.get_single_mut().unwrap();
        if let DrawMode::Fill(ref mut fill_mode) = *rectangle_draw_mode {
            fill_mode.color = Color::RED;
        }
        commands.entity(id).remove::<Moving>();
        window.set_cursor_lock_mode(false);
    }
}

fn mouse_motion(mouse: Res<MouseMovement>, mut query: Query<(&mut Path, &Moving)>) {
    if let Ok((mut path, moving)) = query.get_single_mut() {
        let rect = shapes::Rectangle {
            extents: (mouse.position - moving.origin) * Vec2::new(1.0, -1.0),
            origin: RectangleOrigin::TopLeft,
        };
        *path = ShapePath::build_as(&rect);
    }
}

fn mouse_position(
    windows: Res<Windows>,
    mut mouse: ResMut<MouseMovement>,
    camera: Query<(&Transform, &OrthographicProjection)>,
) {
    let window = windows.get_primary().unwrap();
    let window_size = Vec2::new(window.width(), window.height());
    let (pos, cam) = camera.get_single().unwrap();
    if let Some(cursor_position) = window.cursor_position() {
        let mouse_normalized_screen_pos = (cursor_position / window_size) * 2. - Vec2::ONE;
        mouse.position = pos.translation.truncate()
            + mouse_normalized_screen_pos * Vec2::new(cam.right, cam.top) * cam.scale;
        mouse.normalized = mouse_normalized_screen_pos;
    }
}

fn camera_zoom(
    mut whl: EventReader<MouseWheel>,
    mut mouse_movement: EventReader<MouseMotion>,
    mut cam: Query<(&mut Transform, &mut OrthographicProjection)>,
    mouse: Res<MouseMovement>,
    mouse_button: Res<Input<MouseButton>>,
) {
    let delta_zoom: f32 = whl.iter().map(|e| e.y).sum();
    let mut delta_movement = Vec2::ZERO;
    for i in mouse_movement
        .iter()
        .map(|e| Vec2::new(e.delta.x, -e.delta.y))
    {
        delta_movement -= i;
    }
    if (delta_movement != Vec2::ZERO && mouse_button.pressed(MouseButton::Middle))
        || delta_zoom != 0.
    {
        let (mut pos, mut cam) = cam.single_mut();
        cam.scale -= 0.5 * delta_zoom * cam.scale;
        cam.scale = cam.scale.clamp(0.1, 1000.0);

        pos.translation = (mouse.position
            - mouse.normalized * Vec2::new(cam.right, cam.top) * cam.scale
            + delta_movement * cam.scale)
            .extend(pos.translation.z);
    }
}

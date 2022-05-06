use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
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
    mut query: Query<(&mut Sprite, Entity), With<Moving>>,
    mouse: Res<MouseMovement>,
) {
    let window = windows.get_primary_mut().unwrap();
    if mouse_input.just_pressed(MouseButton::Left) {
        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(1.0, 1.0)),
                    ..default()
                },
                transform: Transform::from_translation(mouse.position.extend(0.0)),
                ..default()
            })
            .insert(Moving {
                origin: mouse.position,
            });
        window.set_cursor_lock_mode(true);
    }

    if mouse_input.just_released(MouseButton::Left) {
        let (mut rectangle_sprite, id) = query.get_single_mut().unwrap();
        rectangle_sprite.color = Color::Rgba {
            red: 1.0,
            green: 0.0,
            blue: 0.0,
            alpha: 1.0,
        };
        commands.entity(id).remove::<Moving>();
        window.set_cursor_lock_mode(false);
    }
}

fn mouse_motion(
    mouse: Res<MouseMovement>,
    mut query: Query<(&mut Sprite, &mut Transform, &Moving)>,
) {
    if let Ok((mut item_sprite, mut item_transform, moving)) = query.get_single_mut() {
        let size = mouse.position - moving.origin;
        item_sprite.custom_size = Some(size);
        item_transform.translation = Vec3::new(
            moving.origin.x + size.x / 2.0,
            moving.origin.y + size.y / 2.0,
            0.0,
        );
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
    for i in mouse_movement.iter().map(|e| Vec2::new(e.delta.x, -e.delta.y)) {
        delta_movement-=i;
    }
    if (delta_movement != Vec2::ZERO && mouse_button.pressed(MouseButton::Middle)) || delta_zoom != 0. {
        let (mut pos, mut cam) = cam.single_mut();
        cam.scale -= 0.5 * delta_zoom * cam.scale;
        cam.scale = cam.scale.clamp(0.1, 1000.0);

        pos.translation = (mouse.position
            - mouse.normalized * Vec2::new(cam.right, cam.top) * cam.scale + delta_movement*cam.scale)
            .extend(pos.translation.z);

    }

}

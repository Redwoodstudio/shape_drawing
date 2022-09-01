use crate::KeyCode::{Delete, Escape, PageDown, PageUp};
use crate::{Moving, ToolChanged};
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy_mod_picking::{PickableBundle, Selection};
use bevy_prototype_lyon::prelude::tess::math::Point;

pub fn point_from_positions(mouse: Vec2, origin: Vec2) -> Point {
    Point::new(mouse.x - origin.x, mouse.y - origin.y)
}

pub fn rotate_around_pivot(mouse: Vec2, origin: Vec2, pivot: Vec2) -> Point {
    Point::new(
        -mouse.x - origin.x + 2.0 * pivot.x,
        -mouse.y - origin.y + 2.0 * pivot.y,
    )
}

pub fn handle_tool_change(
    mut commands: Commands,
    objects: Query<Entity, With<Moving>>,
    mut reader: EventReader<ToolChanged>,
) {
    for _ in reader.iter() {
        for e in objects.iter() {
            commands
                .entity(e)
                .remove::<Moving>()
                .insert_bundle(PickableBundle::default());
        }
    }
}

pub fn handle_keyboard_input(
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    removal_query: Query<(Entity, &Selection)>,
    cancel_query: Query<Entity, With<Moving>>,
) {
    if input.just_pressed(Delete) {
        for e in removal_query.iter().filter_map(|(e, n)| {
            if n.selected() {
                return Some(e);
            }
            None
        }) {
            commands.entity(e).despawn();
        }
    }
    if input.just_pressed(Escape) {
        for e in cancel_query.iter() {
            commands.entity(e).despawn();
        }
    }
}

pub fn handle_layer_change(
    input: Res<Input<KeyCode>>,
    mut layer_query: Query<(&Selection, &mut Transform)>,
) {
    let factor = input.just_pressed(PageUp) as i8 - input.just_pressed(PageDown) as i8;
    if factor != 0 {
        for mut transform in layer_query.iter_mut().filter_map(|(n, t)| {
            if n.selected() {
                return Some(t);
            }
            None
        }) {
            *transform = transform
                .with_translation(transform.translation + Vec3::new(0.0, 0.0, 0.1 * factor as f32));
        }
    }
}

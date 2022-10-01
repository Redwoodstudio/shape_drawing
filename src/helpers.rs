use crate::KeyCode::{Delete, Escape, PageDown, PageUp};
use crate::{ChangedOrderEvent, Moving, ToolChanged};
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
    mut changed: EventWriter<ChangedOrderEvent>,
) {
    if input.just_pressed(Delete) {
        for e in removal_query.iter().filter_map(|(e, n)| {
            if n.selected() {
                return Some(e);
            }
            None
        }) {
            changed.send(ChangedOrderEvent {
                entity: e,
                change_up: false,
                removed: true,
            });
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
    layer_query: Query<(&Selection, Entity)>,
    mut changed: EventWriter<ChangedOrderEvent>,
) {
    let factor = input.just_pressed(PageUp) as i8 - input.just_pressed(PageDown) as i8;
    if factor != 0 {
        if let Some(e) = layer_query
            .iter()
            .filter_map(|(s, e)| if s.selected() { Some(e) } else { None })
            .next()
        {
            changed.send(ChangedOrderEvent {
                entity: e,
                change_up: factor == 1,
                removed: false,
            })
        }
    }
}

pub fn global_vec_to_local(vec: Vec2, rotation: f32) -> Vec2 {
    let len = vec.length();
    let angle = Vec2::new(1.0, 0.0).angle_between(vec);
    let local_angle = angle - rotation;
    let y = local_angle.sin() * len;
    let x = local_angle.cos() * len;
    if !y.is_finite() || !x.is_finite() {
        return Vec2::ZERO;
    }
    Vec2::new(x, y)
}
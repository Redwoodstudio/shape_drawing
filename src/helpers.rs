use crate::{Moving, ToolChanged};
use bevy::prelude::*;
use bevy_mod_picking::{PickableBundle};
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

use crate::{Moving, ToolChanged};
use bevy::prelude::*;
use bevy_mod_picking::PickableBundle;
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

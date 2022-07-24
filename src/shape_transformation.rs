use crate::picking_helpers::TransformScalePick;
use crate::{MouseMovement, PrimitiveShape, ShapeBase};
use bevy::prelude::*;
use bevy_mod_picking::{PickingCamera, Selection};

pub struct ShapeTransformPlugin;

#[derive(Default)]
struct OverEntity {
    entity: Option<Entity>,
}

impl OverEntity {
    fn over(&self, e: Entity) -> bool {
        if let Some(entity) = self.entity {
            entity == e
        } else {
            false
        }
    }
}
impl Plugin for ShapeTransformPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OverEntity>()
            .add_system(move_shape)
            .add_system(scale_shape)
            .add_system(update_focused_shape)
            .add_system(debug_scale);
    }
}

#[derive(Default)]
struct Moved {
    e: Option<Entity>,
    pos_pressed: Vec2,
}
fn move_shape(
    mut moved: Local<Moved>,
    mut query: Query<&mut Transform, (With<Selection>, With<ShapeBase>)>,
    over_entity: Res<OverEntity>,
    mouse_input: Res<Input<MouseButton>>,
    mouse: Res<MouseMovement>,
) {
    if mouse_input.just_released(MouseButton::Left) {
        moved.e = None;
        return;
    }
    if mouse_input.just_pressed(MouseButton::Left) {
        if let Some(e) = over_entity.entity {
            if let Ok(transform) = query.get_mut(e) {
                moved.pos_pressed = mouse.position - transform.translation.truncate();
                moved.e = Some(e);
            }
        }
    }
    if mouse_input.pressed(MouseButton::Left) {
        if let Some(e) = moved.e {
            if let Ok(mut transform) = query.get_mut(e) {
                *transform = transform.with_translation(
                    (mouse.position - moved.pos_pressed).extend(transform.translation.z),
                );
            }
        }
    }
}
#[derive(Default)]
struct Scaled {
    e: Option<Entity>,
    factor: (f32, f32),
    pos_pressed: Vec2,
    orig_scale: Vec3,
    orig_size: Vec2,
}
fn scale_shape(
    mut scaled: Local<Scaled>,
    mut query: Query<&mut Transform, (With<Selection>, With<ShapeBase>)>,
    mut selector_query: Query<&TransformScalePick>,
    over_entity: Res<OverEntity>,
    mouse_input: Res<Input<MouseButton>>,
    mouse: Res<MouseMovement>,
) {
    if mouse_input.just_released(MouseButton::Left) {
        scaled.e = None;
        return;
    }
    if mouse_input.just_pressed(MouseButton::Left) {
        if let Some(e) = over_entity.entity {
            if let Ok(transform_pick) = selector_query.get_mut(e) {
                scaled.pos_pressed = mouse.position;
                scaled.e = transform_pick.entity;
                scaled.factor = transform_pick.location;
                scaled.orig_size = transform_pick.size;
                if let Ok(transform) = query.get(scaled.e.unwrap()) {
                    scaled.orig_scale = transform.scale;
                }
            }
        }
    }
    if mouse_input.pressed(MouseButton::Left) {
        if let Some(e) = scaled.e {
            if let Ok(mut transform) = query.get_mut(e) {
                let whole = scaled.orig_size / scaled.orig_scale.truncate();
                let scale = ((mouse.position - scaled.pos_pressed) * Vec2::from(scaled.factor)
                    + scaled.orig_size)
                    / whole;
                *transform = Transform {
                    translation: transform.translation,
                    rotation: transform.rotation,
                    scale: scale.extend(1.0),
                }
            }
        }
    }
}

fn debug_scale(
    mut q: Query<&mut Transform, With<ShapeBase>>,
    mouse_input: Res<Input<MouseButton>>,
) {
    if mouse_input.pressed(MouseButton::Right) {
        for mut transform in q.iter_mut() {
            *transform = transform.with_scale(Vec3::new(1.5, 1.5, 1.0));
        }
    }
}

fn update_focused_shape(query: Query<&PickingCamera>, mut over_entity: ResMut<OverEntity>) {
    if let Ok(cam) = query.get_single() {
        match cam.intersect_top() {
            Some((e, _)) => over_entity.entity = Some(e),
            None => over_entity.entity = None,
        };
    }
}

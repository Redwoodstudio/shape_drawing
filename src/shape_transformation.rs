use crate::custom_shape::CustomShapeRaw;
use crate::picking_helpers::{TransformRotationPick, TransformScalePick};
use crate::{global_vec_to_local, MouseMovement, Moving, ShapeBase};
use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;
use bevy_mod_picking::{PickingCamera, Selection};
use bevy_prototype_lyon::prelude::{Path, ShapePath};

pub struct ShapeTransformPlugin;

#[derive(Default, Resource)]
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
            .add_system(rotate_shape)
            .add_system(update_focused_shape)
            .add_system(debug_scale);
    }
}

#[derive(Default)]
struct Moved {
    e: Option<Entity>,
    pos_pressed: Vec2,
}

#[derive(Default)]
struct Rotated {
    e: Option<Entity>,
    pos_pressed: Vec2,
    base_rotation: f32,
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
    orig_translat: Vec3,
    global_picker: Vec3,
}
fn scale_shape(
    mut scaled: Local<Scaled>,
    mut query: Query<&mut Transform, (With<Selection>, With<ShapeBase>)>,
    mut selector_query: Query<(&TransformScalePick, &GlobalTransform)>,
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
            if let Ok((transform_pick, pick_transform)) = selector_query.get_mut(e) {
                scaled.e = transform_pick.entity;
                scaled.factor = transform_pick.location;
                scaled.orig_size = transform_pick.size;
                scaled.global_picker = pick_transform.translation();
                if let Ok(transform) = query.get(scaled.e.unwrap()) {
                    scaled.pos_pressed = mouse.position;
                    scaled.orig_scale = transform.scale;
                    scaled.orig_translat = transform.translation
                }
            }
        }
    }
    if mouse_input.pressed(MouseButton::Left) {
        if let Some(e) = scaled.e {
            if let Ok(mut transform) = query.get_mut(e) {
                let not_rotated_position = mouse.position - scaled.pos_pressed;
                let whole = scaled.orig_size / scaled.orig_scale.truncate();
                let r = transform.rotation.to_axis_angle();
                let rotation = r.0.z * r.1;
                let local_position = global_vec_to_local(not_rotated_position, rotation);
                let fact = Vec2::from(scaled.factor);
                let f = local_position * fact + scaled.orig_size;
                let scale = f / whole;
                let offset = global_vec_to_local(local_position * fact.abs(), -rotation);
                *transform = Transform {
                    translation: scaled.orig_translat + offset.extend(0.0) / 2.0,
                    rotation: transform.rotation,
                    scale: scale.extend(1.0),
                }
            }
        }
    }
}

fn rotate_shape(
    mut rotated: Local<Rotated>,
    rot_pickers: Query<&TransformRotationPick>,
    mut transformable_objects: Query<&mut Transform, With<ShapeBase>>,
    over_entity: Res<OverEntity>,
    mouse_input: Res<Input<MouseButton>>,
    mouse: Res<MouseMovement>,
) {
    if mouse_input.just_released(MouseButton::Left) {
        rotated.e = None;
        return;
    }

    if mouse_input.just_pressed(MouseButton::Left) {
        if let Some(e) = over_entity.entity {
            if let Ok(picker) = rot_pickers.get(e) {
                rotated.e = picker.entity;
                rotated.pos_pressed = mouse.position;
                if let Some(ent) = rotated.e {
                    if let Ok(transform) = transformable_objects.get(ent) {
                        let r = transform.rotation.to_axis_angle();
                        rotated.base_rotation = r.0.z * r.1;
                    }
                }
            }
        }
    }

    if mouse_input.pressed(MouseButton::Left) {
        if let Some(e) = rotated.e {
            if let Ok(mut transform) = transformable_objects.get_mut(e) {
                let angle = (rotated.pos_pressed - transform.translation.truncate())
                    .angle_between(mouse.position - transform.translation.truncate());
                *transform =
                    transform.with_rotation(Quat::from_rotation_z(angle + rotated.base_rotation));
            }
        }
    }
}
fn debug_scale(
    mut q: Query<&mut Transform, With<ShapeBase>>,
    mouse_input: Res<Input<MouseButton>>,
) {
    if mouse_input.just_pressed(MouseButton::Right) {
        for mut transform in q.iter_mut() {
            let (_, z) = transform.rotation.to_axis_angle();

            *transform =
                transform.with_rotation(Quat::from_rotation_z(z + std::f32::consts::FRAC_PI_4));
        }
    }
}

fn update_focused_shape(query: Query<&PickingCamera>, mut over_entity: ResMut<OverEntity>) {
    if let Ok(cam) = query.get_single() {
        match cam.get_nearest_intersection() {
            Some((e, _)) => over_entity.entity = Some(e),
            None => over_entity.entity = None,
        };
    }
}

pub fn update_origin(
    points: Res<Assets<Mesh>>,
    mut query: ParamSet<(
        Query<
            (
                &mut CustomShapeRaw,
                &mut Path,
                &mut Transform,
                &Mesh2dHandle,
            ),
            Without<Moving>,
        >,
        Query<Entity, Changed<Path>>,
    )>,
    removed_moving: RemovedComponents<Moving>,
) {
    let mut ents = query
        .p1()
        .iter()
        .chain(removed_moving.iter())
        .collect::<Vec<Entity>>();
    ents.dedup();
    for item in ents {
        if let Ok((mut custom_shape, mut path, mut transform, handle)) = query.p0().get_mut(item) {
            if let Some(mesh) = points.get(&handle.0.clone()) {
                if let Some(aabb) = mesh.compute_aabb() {
                    let old = custom_shape.origin;
                    custom_shape.origin = -(Vec3::from(aabb.center)).truncate() + old;
                    *path = ShapePath::build_as(&custom_shape.clone());
                    *transform = transform.with_translation(
                        transform.translation - (custom_shape.origin - old).extend(0.0),
                    );
                }
            }
        }
    }
}

use crate::{ChangedOrderEvent, OrderedShapes, ShapeBase};
use bevy::prelude::*;

pub fn apply_overlap_order(
    mut objects: Query<&mut Transform, With<ShapeBase>>,
    ordered: Res<OrderedShapes>,
) {
    for (index, e) in ordered.0.iter().enumerate() {
        let mut entity_transform = objects
            .get_mut(*e)
            .expect("Tried to change order for entity, that doesn't exist.");
        *entity_transform = entity_transform.with_translation(Vec3::new(
            entity_transform.translation.x,
            entity_transform.translation.y,
            index as f32,
        ));
    }
}

pub fn calculate_overlap_order(
    new_objects: Query<Entity, Added<ShapeBase>>,
    mut changed_objects: EventReader<ChangedOrderEvent>,
    mut ordered_objects: ResMut<OrderedShapes>,
) {
    for e in new_objects.iter() {
        ordered_objects.0.push(e);
    }
    for (e, top, removed) in changed_objects
        .iter()
        .map(|c| (c.entity, c.change_up, c.removed))
    {
        let i = ordered_objects.0.iter().position(|ent| ent == &e).unwrap();

        if removed {
            ordered_objects.0.remove(i);
        } else if (i + top as usize) < ordered_objects.0.len() && (i as isize - !top as isize) >= 0
        {
            ordered_objects.0.swap(i - !top as usize, i + top as usize);
        }
    }
}

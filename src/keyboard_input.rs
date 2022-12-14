use crate::custom_shape::CustomShapeRaw;
use crate::KeyCode::{Delete, Escape, PageDown, PageUp, S};
use crate::{ChangedOrderEvent, Moving, ShapeSegment};
use bevy::prelude::*;
use bevy_mod_picking::Selection;
use std::fs;

pub struct KeyboardInputPlugin;

impl Plugin for KeyboardInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(handle_save_input)
            .add_system(handle_keyboard_input)
            .add_system(handle_layer_change);
    }
}
fn handle_keyboard_input(
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

fn handle_layer_change(
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

fn handle_save_input(input: Res<Input<KeyCode>>, shapes: Query<&CustomShapeRaw>) {
    if input.pressed(KeyCode::LControl) && input.just_pressed(S) {
        info!("saving");
        let items = shapes
            .iter()
            .map(|c| &c.segments)
            .collect::<Vec<&Vec<ShapeSegment>>>();
        let encoded: Vec<u8> = bincode::serialize(&items).unwrap();
        fs::write("foo.txt", encoded).expect("Unable to write file");
    }
}

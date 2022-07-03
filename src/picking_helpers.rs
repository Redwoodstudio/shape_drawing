use bevy::prelude::*;
use crate::MouseMovement;
use bevy::ui::FocusPolicy;
use bevy::app::PluginGroupBuilder;
use bevy::ecs::schedule::ShouldRun;
use bevy_mod_picking::{
    mesh_events_system, mesh_focus, mesh_selection, pause_for_picking_blockers, Hover,
    PausedForBlockers, PickableMesh, PickingEvent, PickingPlugin,
    PickingPluginsState, PickingSystem
};
fn pause_for_egui(
    mouse: Res<MouseMovement>,
    mut paused: ResMut<PausedForBlockers>,
    mut interaction: Query<
        (
            &mut Interaction,
            Option<&mut Hover>,
            Option<&FocusPolicy>,
            Entity,
        ),
        With<PickableMesh>,
    >,
) {
    if mouse.over_ui {
        for (mut interaction, hover, _, _) in &mut interaction.iter_mut() {
            if *interaction != Interaction::None {
                *interaction = Interaction::None;
            }

            if let Some(mut hover) = hover {
                if hover.hovered {
                    hover.hovered = false;
                }
            }
        }
        paused.0 = true;
        return;
    }
}

pub struct CustomInteractablePickingPlugin;
impl Plugin for CustomInteractablePickingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PausedForBlockers>()
            .add_event::<PickingEvent>()
            .add_system_set_to_stage(
                CoreStage::First,
                SystemSet::new()
                    .with_run_criteria(|state: Res<PickingPluginsState>| {
                        simple_criteria(state.enable_interacting)
                    })
                    .with_system(
                        pause_for_picking_blockers
                            .label(PickingSystem::PauseForBlockers)
                            .after(PickingSystem::UpdateIntersections),
                    )
                    .with_system(
                        pause_for_egui
                            .label(PickingSystem::PauseForEgui)
                            .after(PickingSystem::PauseForBlockers),
                    )
                    .with_system(
                        mesh_focus
                            .label(PickingSystem::Focus)
                            .after(PickingSystem::PauseForEgui),
                    )
                    .with_system(
                        mesh_selection
                            .label(PickingSystem::Selection)
                            .after(PickingSystem::Focus),
                    )
                    .with_system(
                        mesh_events_system
                            .label(PickingSystem::Events)
                            .after(PickingSystem::Selection),
                    ),
            );
    }
}

pub struct CustomPickingPlugins;
impl PluginGroup for CustomPickingPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(PickingPlugin);
        group.add(CustomInteractablePickingPlugin);
    }
}
fn simple_criteria(flag: bool) -> ShouldRun {
    if flag {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}
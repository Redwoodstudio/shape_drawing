use crate::{MouseMovement, PrimitiveType, ShapeBase, Tool, ToolType};
use bevy::app::PluginGroupBuilder;
use bevy::ecs::schedule::ShouldRun;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy_egui::egui::Color32;
use bevy_egui::{egui, EguiContext};
use bevy_mod_picking::{
    mesh_events_system, mesh_focus, mesh_selection, pause_for_picking_blockers, Hover,
    PausedForBlockers, PickableMesh, PickingEvent, PickingPlugin,
    PickingPluginsState, PickingSystem, Selection,
};
use bevy_prototype_lyon::draw::{DrawMode, FillMode, StrokeMode};
use bevy_prototype_lyon::draw::DrawMode::Outlined;

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(ui_example.label("egui"))
            .add_system(objects_list.label("egui"))
            .add_system(edit_style.label("egui"));
    }
}

fn ui_example(
    mut egui_context: ResMut<EguiContext>,
    mut current: ResMut<Tool>,
    mut mouse: ResMut<MouseMovement>,
) {
    egui::Window::new("Tool Options").show(egui_context.ctx_mut(), |ui| {
        let mut prim_type = PrimitiveType::Rectangle;
        match current.tool {
            ToolType::Primitive(mut sh) => {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut sh, PrimitiveType::Rectangle, "Rectangle");
                    ui.selectable_value(&mut sh, PrimitiveType::Ellipse, "Ellipse");
                });
                prim_type = sh;
                current.tool = ToolType::Primitive(sh);
            }
            _ => (),
        }
        ui.label("Choose drawing mode");
        ui.horizontal(|ui| {
            ui.selectable_value(&mut current.tool, ToolType::None, "None");
            ui.selectable_value(
                &mut current.tool,
                ToolType::Primitive(prim_type),
                "Primitive",
            );
            ui.selectable_value(&mut current.tool, ToolType::CustomShape, "Custom Shape");
        });
        ui.end_row();
        ui.label("Choose shape color");
        ui.color_edit_button_srgba_premultiplied(&mut current.color);
        ui.end_row();
    });
    mouse.over_ui = egui_context.ctx_mut().wants_pointer_input();
}

fn objects_list(
    mut egui_context: ResMut<EguiContext>,
    mut query: Query<(&ShapeBase, &mut Selection, Entity)>,
    mut mouse: ResMut<MouseMovement>,
) {
    let mut selected = None;
    egui::Window::new("Objects").show(egui_context.ctx_mut(), |ui| {
        ui.vertical(|ui| {
            let mut ents: Vec<(&ShapeBase, &Selection, Entity)> = query.iter().collect();
            ents.sort_by(|a, b| b.2.cmp(&a.2));
            ents.iter().for_each(|(_name, selection, e)| {
                if ui
                    .selectable_label(selection.selected(), format!("{:?}", e))
                    .clicked()
                {
                    selected = Some(*e);
                }
            });
        });
    });
    if let Some(e) = selected {
        if let Ok(mut comp) = query.get_component_mut::<Selection>(e) {
            comp.set_selected(true);
        }
    }

    mouse.over_ui = egui_context.ctx_mut().wants_pointer_input();
}
fn edit_style(
    mut egui_context: ResMut<EguiContext>,
    mut query: Query<&mut DrawMode>,
    mut mouse: ResMut<MouseMovement>,
    query2: Query<(&Selection, Entity)>,
) {
    let x: Vec<Entity> = query2
        .iter()
        .filter_map(|(n, e)| {
            if n.selected() {
                return Some(e);
            }
            None
        })
        .collect();
    if x.len() == 1 {
        if let Ok(mut draw_mode) = query.get_component_mut::<DrawMode>(x[0]) {
            egui::Window::new("Edit Style").show(egui_context.ctx_mut(), |ui| match *draw_mode {
                DrawMode::Fill(fill_mode) => {
                    let mut edited = false;
                    ui.horizontal(|ui| {
                        let _ = ui.selectable_label(true, "Fill");
                        if ui.selectable_label(false, "Stroke").clicked() {
                            *draw_mode = DrawMode::Stroke(StrokeMode::new(Color::BLACK, 5.0));
                            edited = true;
                            return;
                        }
                        if ui.selectable_label(false, "Outlined").clicked() {
                            *draw_mode = DrawMode::Outlined{fill_mode, outline_mode: StrokeMode::new(Color::BLACK, 5.0)};
                            edited = true;
                            return;
                        }
                    });
                    if edited {
                        return;
                    }
                    let col = fill_mode.color.as_rgba_f32();
                    let mut color = Color32::from_rgba_premultiplied(
                        (col[0] * 256.0) as u8,
                        (col[1] * 256.0) as u8,
                        (col[2] * 256.0) as u8,
                        (col[3] * 256.0) as u8,
                    );
                    ui.color_edit_button_srgba(&mut color);
                    *draw_mode = DrawMode::Fill(FillMode::color(Color::rgba_u8(
                        color.r(),
                        color.g(),
                        color.b(),
                        color.a(),
                    )));
                }
                DrawMode::Stroke(stroke_mode) => {
                    let mut edited = false;
                    ui.horizontal(|ui| {
                        if ui.selectable_label(false, "Fill").clicked() {
                            *draw_mode = DrawMode::Fill(FillMode::color(Color::BLACK));
                            edited = true;
                            return;
                        }
                        let _ =  ui.selectable_label(true, "Stroke");
                        if ui.selectable_label(false, "Outlined").clicked() {
                            *draw_mode = DrawMode::Outlined{fill_mode: FillMode::color(Color::BLACK), outline_mode: stroke_mode};
                            edited = true;
                            return;
                        }
                    });
                    if edited {
                        return;
                    }
                },
                DrawMode::Outlined{outline_mode, fill_mode} => {
                    let mut edited = false;
                    ui.horizontal(|ui| {
                        if ui.selectable_label(false, "Fill").clicked() {
                            *draw_mode = DrawMode::Fill(fill_mode);
                            edited = true;
                            return;
                        }
                        if ui.selectable_label(false, "Stroke").clicked() {
                            *draw_mode = DrawMode::Stroke(outline_mode);
                            edited = true;
                            return;
                        }
                        let _ =  ui.selectable_label(true, "Outlined");
                    });
                    if edited {
                        return;
                    }
                    let col = fill_mode.color.as_rgba_f32();
                    let mut color = Color32::from_rgba_premultiplied(
                        (col[0] * 256.0) as u8,
                        (col[1] * 256.0) as u8,
                        (col[2] * 256.0) as u8,
                        (col[3] * 256.0) as u8,
                    );
                    ui.color_edit_button_srgba(&mut color);
                    *draw_mode = DrawMode::Outlined{fill_mode: FillMode::color(Color::rgba_u8(
                        color.r(),
                        color.g(),
                        color.b(),
                        color.a(),
                    )),
                    outline_mode: StrokeMode::new(Color::BLACK, 5.0)};
                },
            });
        }
        mouse.over_ui = egui_context.ctx_mut().wants_pointer_input();
    }
}

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

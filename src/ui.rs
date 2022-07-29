use crate::{MouseMovement, PrimitiveType, ShapeBase, Tool, ToolType};
use bevy::prelude::*;
use bevy_egui::egui::Color32;
use bevy_egui::{egui, EguiContext};
use bevy_mod_picking::Selection;
use bevy_prototype_lyon::draw::{DrawMode, FillMode, StrokeMode};

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
        if let ToolType::Primitive(mut sh) = current.tool {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut sh, PrimitiveType::Rectangle, "Rectangle");
                ui.selectable_value(&mut sh, PrimitiveType::Ellipse, "Ellipse");
            });
            prim_type = sh;
            current.tool = ToolType::Primitive(sh);
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
        for (_, mut sel, _) in query.iter_mut() {
            sel.set_selected(false);
        }
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
                            *draw_mode = DrawMode::Outlined {
                                fill_mode,
                                outline_mode: StrokeMode::new(Color::BLACK, 5.0),
                            };
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
                        let _ = ui.selectable_label(true, "Stroke");
                        if ui.selectable_label(false, "Outlined").clicked() {
                            *draw_mode = DrawMode::Outlined {
                                fill_mode: FillMode::color(Color::BLACK),
                                outline_mode: stroke_mode,
                            };
                            edited = true;
                            return;
                        }
                    });
                    if edited {
                        return;
                    }
                    let mut num = stroke_mode.options.line_width;
                    let col = stroke_mode.color.as_rgba_f32();
                    let mut color = Color32::from_rgba_premultiplied(
                        (col[0] * 256.0) as u8,
                        (col[1] * 256.0) as u8,
                        (col[2] * 256.0) as u8,
                        (col[3] * 256.0) as u8,
                    );
                    ui.add(egui::DragValue::new(&mut num));
                    ui.color_edit_button_srgba(&mut color);
                    *draw_mode = DrawMode::Stroke(StrokeMode::new(
                        Color::rgba_u8(color.r(), color.g(), color.b(), color.a()),
                        num,
                    ));
                }
                DrawMode::Outlined {
                    outline_mode,
                    fill_mode,
                } => {
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
                        let _ = ui.selectable_label(true, "Outlined");
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
                    let stroke_col = outline_mode.color.as_rgba_f32();
                    let mut color2 = Color32::from_rgba_premultiplied(
                        (stroke_col[0] * 256.0) as u8,
                        (stroke_col[1] * 256.0) as u8,
                        (stroke_col[2] * 256.0) as u8,
                        (stroke_col[3] * 256.0) as u8,
                    );
                    let mut num = outline_mode.options.line_width;
                    ui.color_edit_button_srgba(&mut color);
                    ui.add(egui::DragValue::new(&mut num));
                    ui.color_edit_button_srgba(&mut color2);
                    *draw_mode = DrawMode::Outlined {
                        fill_mode: FillMode::color(Color::rgba_u8(
                            color.r(),
                            color.g(),
                            color.b(),
                            color.a(),
                        )),
                        outline_mode: StrokeMode::new(
                            Color::rgba_u8(color2.r(), color2.g(), color2.b(), color2.a()),
                            num,
                        ),
                    };
                }
            });
        }
        mouse.over_ui = egui_context.ctx_mut().wants_pointer_input();
    }
}

use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use crate::{MouseMovement, PrimitiveType, Selected, ShapeBase, Tool, ToolType};

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(ui_example)
            .add_system(objects_list);
    }
}
fn ui_example(mut egui_context: ResMut<EguiContext>, mut current: ResMut<Tool>, mut mouse: ResMut<MouseMovement>) {
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
            _ => ()
        }
        ui.label("Choose drawing mode");
        ui.horizontal(|ui| {
            ui.selectable_value(&mut current.tool, ToolType::None, "None");
            ui.selectable_value(&mut current.tool, ToolType::Primitive(prim_type), "Primitive");
            ui.selectable_value(&mut current.tool, ToolType::CustomShape, "Custom Shape");
        });
        ui.end_row();
        ui.label("Choose shape color");
        ui.color_edit_button_srgba_premultiplied(&mut current.color);
        ui.end_row();
    });
    mouse.over_ui =  egui_context.ctx_mut().wants_pointer_input();

}

fn objects_list(mut commands: Commands, mut egui_context: ResMut<EguiContext>, query: Query<(&ShapeBase, Entity)>, query2: Query<Entity, With<Selected>>) {
    let mut ent = None;
    if let Ok(entity) = query2.get_single() {
        ent = Some(entity);
    }
    egui::Window::new("Objects").show(egui_context.ctx_mut(), |ui| {
       ui.vertical(|ui| {
           let mut ents = query.iter().collect::<Vec<(&ShapeBase, Entity)>>();
           ents.sort_by(|a, b| b.1.cmp(&a.1));
           ents.iter().for_each(|(_name, e) |{
               if ui.selectable_label(Some(*e) == ent, format!("{:?}", e)).clicked() {
                   for entity in query2.iter() {
                       commands.entity(entity).remove::<Selected>();
                   }
                   commands.entity(*e).insert(Selected);
           }});
       });
    });
    }

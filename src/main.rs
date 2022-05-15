use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_prototype_lyon::entity::ShapeBundle;

fn main() {
    let mut app = App::new();
    app
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_plugin(EguiPlugin)
        .add_startup_system(spawn_camera)
        .add_system(mouse_position)
        .add_system(spawn_rectangle)
        .add_system(mouse_motion)
        .add_system(camera_zoom)
        .add_system(ui_example)
        .init_resource::<MouseMovement>()
        .init_resource::<Tool>();
    #[cfg(target_arch = "wasm32")]
    {
        app.add_plugin(bevy_web_resizer::Plugin);
    }
    app.run();
}

#[derive(Default)]
struct MouseMovement {
    position: Vec2,
    normalized: Vec2,
}

#[derive(PartialEq)]
enum ToolType {
    None,
    Rectangle,
    Ellipse,
    Line,
}

#[derive(Default)]
struct Tool {
    tool: ToolType,
    color: [f32;4]
}

impl Default for ToolType {
    fn default() -> Self {
        Self::None
    }
}
#[derive(Component)]
struct Moving {
    origin: Vec2
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}
fn spawn_rectangle(
    mut commands: Commands,
    mut windows: ResMut<Windows>,
    tool: Res<Tool>,
    mouse_input: Res<Input<MouseButton>>,
    mut query: Query<(&mut DrawMode, Entity), With<Moving>>,
    mouse: Res<MouseMovement>,
) {
    let window = windows.get_primary_mut().unwrap();
    if mouse_input.just_pressed(MouseButton::Left) {
        if tool.tool != ToolType::None {
            commands
                .spawn_bundle(create_shape(&tool.tool, mouse.position))
                .insert(Moving {
                    origin: mouse.position,
                });

            //window.set_cursor_lock_mode(true);
        }


    }

    if mouse_input.just_released(MouseButton::Left) {
        if let Ok((mut rectangle_draw_mode, id)) = query.get_single_mut() {
            if let DrawMode::Fill(ref mut fill_mode) = *rectangle_draw_mode {
                fill_mode.color = Color::from(tool.color);
            }
            commands.entity(id).remove::<Moving>();
            window.set_cursor_lock_mode(false);
        }
    }
}

fn mouse_motion(mouse: Res<MouseMovement>, mut query: Query<(&mut Path, &Moving)>, tool: Res<Tool>) {
    if tool.tool != ToolType::None {
        if let Ok((mut path, moving)) = query.get_single_mut() {
            *path = edit_shape(&tool.tool, mouse.position, moving.origin);
        }
    }
}

fn mouse_position(
    windows: Res<Windows>,
    mut mouse: ResMut<MouseMovement>,
    camera: Query<(&Transform, &OrthographicProjection)>,
) {
    let window = windows.get_primary().unwrap();
    let window_size = Vec2::new(window.width(), window.height());
    let (pos, cam) = camera.get_single().unwrap();
    if let Some(cursor_position) = window.cursor_position() {
        let mouse_normalized_screen_pos = (cursor_position / window_size) * 2. - Vec2::ONE;
        mouse.position = pos.translation.truncate()
            + mouse_normalized_screen_pos * Vec2::new(cam.right, cam.top) * cam.scale;
        mouse.normalized = mouse_normalized_screen_pos;
    }
}

fn camera_zoom(
    mut whl: EventReader<MouseWheel>,
    mut mouse_movement: EventReader<MouseMotion>,
    mut cam: Query<(&mut Transform, &mut OrthographicProjection)>,
    mouse: Res<MouseMovement>,
    mouse_button: Res<Input<MouseButton>>,
) {
    let mut delta_zoom: f32 = whl.iter().map(|e| e.y).sum();
    #[cfg(target_arch = "wasm32")]
    {
        delta_zoom /= 100.0;
    }
    let mut delta_movement = Vec2::ZERO;
    for i in mouse_movement
        .iter()
        .map(|e| Vec2::new(e.delta.x, -e.delta.y))
    {
        delta_movement -= i;
    }
    if (delta_movement != Vec2::ZERO && mouse_button.pressed(MouseButton::Middle))
        || delta_zoom != 0.
    {
        let (mut pos, mut cam) = cam.single_mut();
        cam.scale -= 0.5 * delta_zoom * cam.scale;
        cam.scale = cam.scale.clamp(0.1, 1000.0);

        pos.translation = (mouse.position
            - mouse.normalized * Vec2::new(cam.right, cam.top) * cam.scale
            + delta_movement * cam.scale)
            .extend(pos.translation.z);
    }
}

fn ui_example(mut egui_context: ResMut<EguiContext>, mut current: ResMut<Tool>) {
    egui::Window::new("Hello").show(egui_context.ctx_mut(), |ui| {
        ui.label("Choose drawing mode");
        ui.horizontal(|ui| {
            ui.selectable_value(&mut current.tool, ToolType::None, "None");
            ui.selectable_value(&mut current.tool, ToolType::Rectangle, "Rectangle");
            ui.selectable_value(&mut current.tool, ToolType::Ellipse, "Ellipse");
            ui.selectable_value(&mut current.tool, ToolType::Line, "Line");
        });
        ui.end_row();
        ui.label("Choose shape color");
        ui.color_edit_button_rgba_premultiplied(&mut current.color);

    });
}

fn edit_shape(tool: &ToolType, mouse_position: Vec2, origin: Vec2) -> Path {
        match tool {
            &ToolType::None => {unreachable!()}
            &ToolType::Rectangle => {
                ShapePath::build_as(
                    &shapes::Rectangle {
                        extents: (mouse_position-origin)*Vec2::new(1.0, -1.0),
                        origin: RectangleOrigin::TopLeft,
                    })
            }
            &ToolType::Ellipse => {
                ShapePath::build_as(
                &shapes::Ellipse {
                    radii: (mouse_position-origin)/2.0,
                    center: (mouse_position-origin)/2.0
                })
            }
            &ToolType::Line => {
                ShapePath::build_as(
                &shapes::Line(Vec2::ZERO, mouse_position-origin)
                )
            }
        }
}

fn create_shape(tool: &ToolType, mouse_position: Vec2) -> ShapeBundle {
    let builder = GeometryBuilder::new();
        match tool {
            &ToolType::None => {unreachable!()}
            &ToolType::Rectangle => {
                builder.add(
                    &shapes::Rectangle {
                        extents: Vec2::ZERO,
                        origin: RectangleOrigin::Center,
                    }).build(DrawMode::Fill(FillMode::color(Color::WHITE)),
                             Transform::from_translation(mouse_position.extend(0.0)))
            }
            &ToolType::Ellipse => {
                builder.add(
                &shapes::Ellipse {
                    radii: Vec2::ZERO,
                    center: mouse_position
                }).build(DrawMode::Fill(FillMode::color(Color::WHITE)),
                         Transform::from_translation(mouse_position.extend(0.0)))
            }
            &ToolType::Line => {
                builder.add(
                &shapes::Line(mouse_position, mouse_position)
                ).build(DrawMode::Stroke(StrokeMode::new(Color::WHITE, 15.0)),
                        Transform::from_translation(mouse_position.extend(0.0)))
            }
        }

}
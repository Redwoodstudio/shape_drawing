use crate::tess::geom::euclid::Point2D;
use crate::tess::geom::Point;
use crate::tess::path::path::Builder;
use crate::ShapeSegment::{CubicBezier, Line, QuadraticBezier};
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::*;
use iyes_loopless::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_plugin(EguiPlugin)
        .add_startup_system(spawn_camera)
        .add_system(camera_zoom)
        .add_system(ui_example)
        .add_system(mouse_position)
        .add_system_set(
            ConditionSet::new()
                .run_if(should_handle_primitive)
                .with_system(primitive_handle_creation)
                .with_system(primitive_handle_update)
                .into(),
        )
        .insert_resource(ClearColor(Color::WHITE))
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
    Primitive,
    CustomShape,
}

struct Tool {
    tool: ToolType,
    color: [f32; 4],
}
enum PrimitiveType {
    Rectangle,
    Ellipse,
    RoundedRectangle,
}
impl Default for Tool {
    fn default() -> Self {
        Self {
            tool: ToolType::None,
            color: [0.0, 0.0, 0.0, 1.0],
        }
    }
}

impl Default for ToolType {
    fn default() -> Self {
        Self::None
    }
}
#[derive(Component)]
struct Moving;
#[derive(Component)]
struct PrimitiveShape {
    name: Option<String>,
    origin: Vec2,
}
fn spawn_camera(mut commands: Commands) {
    let shape = CustomShape {
        segments: vec![
            Line(Point::new(100.0, 0.0)),
            QuadraticBezier {
                ctrl: Point::new(0.0, 0.0),
                to: Point::new(0.0, 100.0),
            },
            Line(Point::new(200.0, 100.0)),
            Line(Point::new(200.0, 0.0)),
        ],
    };

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(GeometryBuilder::build_as(
        &shape,
        DrawMode::Fill(FillMode::color(Color::CRIMSON)),
        Transform::default(),
    ));
}
fn primitive_handle_creation(
    mut commands: Commands,
    tool: Res<Tool>,
    mouse_input: Res<Input<MouseButton>>,
    mut query: Query<(&Transform, Entity), With<Moving>>,
    mouse: Res<MouseMovement>,
) {
    if mouse_input.just_pressed(MouseButton::Left)  {
            let shape = GeometryBuilder::build_as(
                &shapes::Rectangle {
                    extents: Vec2::ZERO,
                    origin: RectangleOrigin::Center,
                },
                DrawMode::Fill(FillMode::color(Color::from(tool.color))),
                Transform::from_translation(mouse.position.extend(0.1)),
            );
            commands
                .spawn_bundle(shape)
                .insert(Moving)
                .insert(PrimitiveShape {
                    name: None,
                    origin: mouse.position,
                });
        }

    if mouse_input.just_released(MouseButton::Left) {
        if let Ok((transform, id)) = query.get_single_mut() {
            info!("End: {:?}", transform);
            commands.entity(id).remove::<Moving>();
        }
    }
}

fn primitive_handle_update(
    mouse: Res<MouseMovement>,
    mut query: Query<(&mut Path, &PrimitiveShape), With<Moving>>,
) {
    if let Ok((mut path, moving)) = query.get_single_mut() {
        *path = ShapePath::build_as(&shapes::Rectangle {
            extents: (mouse.position - moving.origin) * Vec2::new(1.0, -1.0),
            origin: RectangleOrigin::TopLeft,
        });
    };
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
            ui.selectable_value(&mut current.tool, ToolType::Primitive, "Primitive");
            ui.selectable_value(&mut current.tool, ToolType::CustomShape, "Custom Shape");
        });
        ui.end_row();
        ui.label("Choose shape color");
        ui.color_edit_button_rgba_premultiplied(&mut current.color);
        ui.end_row();
    });
}

/*fn edit_shape(tool: &ToolType, mouse_position: Vec2, origin: Vec2) -> Path {
    match tool {
        &ToolType::None => {
            unreachable!()
        }
        &ToolType::Rectangle => ShapePath::build_as(&shapes::Rectangle {
            extents: (mouse_position - origin) * Vec2::new(1.0, -1.0),
            origin: RectangleOrigin::TopLeft,
        }),
        &ToolType::Ellipse => ShapePath::build_as(&shapes::Ellipse {
            radii: (mouse_position - origin) / 2.0,
            center: (mouse_position - origin) / 2.0,
        }),
        &ToolType::CustomShape => ShapePath::build_as(&CustomShape {
            segments: vec![],
        }),
    }
}*/

/*fn create_shape(tool: &ToolType, mouse_position: Vec2) -> ShapeBundle {
    let builder = GeometryBuilder::new();
    match tool {
        &ToolType::None => {
            unreachable!()
        }
        &ToolType::Rectangle => builder
            .add(&shapes::Rectangle {
                extents: Vec2::ZERO,
                origin: RectangleOrigin::Center,
            })
            .build(
                DrawMode::Fill(FillMode::color(Color::WHITE)),
                Transform::from_translation(mouse_position.extend(0.0)),
            ),
        &ToolType::Ellipse => builder
            .add(&shapes::Ellipse {
                radii: Vec2::ZERO,
                center: mouse_position,
            })
            .build(
                DrawMode::Fill(FillMode::color(Color::WHITE)),
                Transform::from_translation(mouse_position.extend(0.0)),
            ),
        &ToolType::CustomShape => builder
            .add(&CustomShape {
                segments: vec![],
            }).build(DrawMode::Fill(FillMode::color(Color::WHITE)),
                     Transform::from_translation(mouse_position.extend(0.0))),
    }
}*/

fn should_handle_primitive(tool: Res<Tool>) -> bool {
    return tool.tool == ToolType::Primitive;
}
#[derive(Debug, Clone, PartialEq)]
struct CustomShape {
    pub segments: Vec<ShapeSegment>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
enum ShapeSegment {
    Line(Point<f32>),
    QuadraticBezier {
        ctrl: Point<f32>,
        to: Point<f32>,
    },
    CubicBezier {
        ctrl: Point<f32>,
        ctrl2: Point<f32>,
        to: Point<f32>,
    },
}

impl Geometry for CustomShape {
    fn add_geometry(&self, b: &mut Builder) {
        b.begin(Point2D::new(100.0, 0.0));
        for segment in self.segments.iter() {
            match *segment {
                Line(end) => b.line_to(end),
                QuadraticBezier { ctrl, to } => b.quadratic_bezier_to(ctrl, to),
                CubicBezier { ctrl, ctrl2, to } => b.cubic_bezier_to(ctrl, ctrl2, to),
            };
        }
        b.close();
    }
}

mod custom_shape;
mod helpers;
mod picking_helpers;
mod shape_transformation;
mod ui;

use crate::custom_shape::{custom_shape_handle_creation, custom_shape_handle_update, ShapeSegment};
use crate::picking_helpers::{spawn_highlight_rectangle, CustomPickingPlugins};
use crate::shape_transformation::ShapeTransformPlugin;
use crate::ui::UIPlugin;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_mod_picking::{DebugEventsPickingPlugin, PickableBundle, PickingCameraBundle};
use bevy_prototype_lyon::prelude::*;
use iyes_loopless::prelude::*;

fn main() {
    let mut app = App::new();
    app
        //.add_plugins_with(DefaultPlugins, |plugins| plugins.disable::<bevy::log::LogPlugin>())
        .add_plugins(DefaultPlugins)
        .add_plugins(CustomPickingPlugins)
        .add_plugin(ShapePlugin)
        .add_plugin(EguiPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(UIPlugin)
        .add_plugin(ShapeTransformPlugin)
        .add_plugin(DebugEventsPickingPlugin)
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_highlight_rectangle)
        //.add_system(select_event)
        .add_system(camera_zoom)
        .add_system(mouse_position)
        .add_system_set(
            ConditionSet::new()
                .run_if(should_handle_primitive)
                .with_system(primitive_handle_creation)
                .with_system(primitive_handle_update)
                .into(),
        )
        .add_system_set(
            ConditionSet::new()
                .run_if(should_handle_custom_shape)
                .with_system(custom_shape_handle_creation)
                .with_system(custom_shape_handle_update)
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
    //bevy_mod_debugdump::print_schedule(&mut app);
}

#[derive(Default)]
pub struct MouseMovement {
    position: Vec2,
    normalized: Vec2,
    over_ui: bool,
}

#[derive(PartialEq, Debug)]
enum ToolType {
    None,
    Primitive(PrimitiveType),
    CustomShape,
}

pub struct Tool {
    tool: ToolType,
    color: [u8; 4],
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum PrimitiveType {
    Rectangle,
    Ellipse,
    RoundedRectangle,
}
impl Default for Tool {
    fn default() -> Self {
        Self {
            tool: ToolType::None,
            color: [0, 0, 0, 255],
        }
    }
}

impl Default for ToolType {
    fn default() -> Self {
        Self::None
    }
}
#[derive(Component)]
pub struct Moving {
    origin: Vec2,
}
#[derive(Component)]
struct PrimitiveShape {
    shape: PrimitiveType,
}

#[derive(Component)]
pub struct ShapeBase {
    name: Option<String>,
    originx: Vec3,
}
fn spawn_camera(mut commands: Commands) {
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert_bundle(PickingCameraBundle::default());
}

fn primitive_handle_creation(
    mut commands: Commands,
    tool: Res<Tool>,
    mouse_input: Res<Input<MouseButton>>,
    mut query: Query<Entity, With<Moving>>,
    mouse: Res<MouseMovement>,
) {
    if mouse_input.just_pressed(MouseButton::Left) && !mouse.over_ui {
        let color = Color::rgba_u8(tool.color[0], tool.color[1], tool.color[2], tool.color[3]);
        let prim_type = match tool.tool {
            ToolType::Primitive(t) => t,
            _ => unreachable!(),
        };
        let shape = match prim_type {
            PrimitiveType::Rectangle => GeometryBuilder::build_as(
                &shapes::Rectangle {
                    extents: Vec2::ZERO,
                    origin: RectangleOrigin::TopLeft,
                },
                DrawMode::Fill(FillMode::color(color)),
                Transform::from_translation(mouse.position.extend(0.1)),
            ),
            PrimitiveType::Ellipse => GeometryBuilder::build_as(
                &shapes::Ellipse {
                    radii: Vec2::ZERO,
                    center: Vec2::ZERO,
                },
                DrawMode::Fill(FillMode::color(color)),
                Transform::from_translation(mouse.position.extend(0.1)),
            ),
            _ => unreachable!(),
        };

        commands
            .spawn_bundle(shape)
            .insert(Moving {
                origin: mouse.position,
            })
            .insert(ShapeBase {
                name: None,
                originx: Vec3::new(0.0, 0.0, 0.0),
            })
            .insert(PrimitiveShape { shape: prim_type });
    }

    if mouse_input.just_released(MouseButton::Left) {
        if let Ok(id) = query.get_single_mut() {
            commands.entity(id).remove::<Moving>();
            commands.entity(id).insert_bundle(PickableBundle::default());
        }
    }
}

fn primitive_handle_update(
    mouse: Res<MouseMovement>,
    mut query: Query<(&mut Path, &Moving, &PrimitiveShape)>,
) {
    if let Ok((mut path, moving, prim_type)) = query.get_single_mut() {
        *path = match prim_type.shape {
            PrimitiveType::Rectangle => ShapePath::build_as(&shapes::Rectangle {
                extents: (mouse.position - moving.origin) * Vec2::new(1.0, -1.0),
                origin: RectangleOrigin::TopLeft,
            }),
            PrimitiveType::Ellipse => ShapePath::build_as(&shapes::Ellipse {
                radii: (mouse.position - moving.origin) / 2.0,
                center: (mouse.position - moving.origin) / 2.0,
            }),
            _ => unreachable!(),
        };
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
    #[allow(unused_mut)]
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

fn should_handle_primitive(tool: Res<Tool>) -> bool {
    matches!(tool.tool, ToolType::Primitive(_))
}
fn should_handle_custom_shape(tool: Res<Tool>) -> bool {
    tool.tool == ToolType::CustomShape
}

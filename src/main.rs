mod ui;
mod picking_helpers;
use crate::tess::geom::euclid::Point2D;
use crate::tess::geom::Point;
use crate::tess::path::path::Builder;
use crate::ui::UIPlugin;
use crate::ShapeSegment::{CubicBezier, Line, QuadraticBezier};
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_mod_picking::{DefaultPickingPlugins, PickableBundle, PickingCameraBundle, PickingEvent, SelectionEvent};
use bevy_prototype_lyon::prelude::*;
use iyes_loopless::prelude::*;
use bevy::log::LogPlugin;
use crate::picking_helpers::CustomPickingPlugins;

fn main() {
    let mut app = App::new();
    app
        //.add_plugins_with(DefaultPlugins, |plugins| plugins.disable::<bevy::log::LogPlugin>())
        .add_plugins(DefaultPlugins)
        .add_plugins(CustomPickingPlugins)
        .add_plugin(ShapePlugin)
        .add_plugin(EguiPlugin)
        .add_plugin(UIPlugin)
        .add_startup_system(spawn_camera)
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
struct MouseMovement {
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

struct Tool {
    tool: ToolType,
    color: [u8; 4],
}

#[derive(PartialEq, Copy, Clone, Debug)]
enum PrimitiveType {
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
struct Moving {
    origin: Vec2,
}
#[derive(Component)]
struct PrimitiveShape {
    shape: PrimitiveType,
}

#[derive(Component)]
struct CustomShape {
    segments: Vec<ShapeSegment>,
}

#[derive(Component)]
struct ShapeBase {
    name: Option<String>,
    originx: Vec3,
}
fn spawn_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d()).insert_bundle(PickingCameraBundle::default());
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
                    origin: RectangleOrigin::Center,
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

fn custom_shape_handle_creation(
    mut commands: Commands,
    mouse_input: Res<Input<MouseButton>>,
    mouse: Res<MouseMovement>,
    tool: Res<Tool>,
    query: Query<&Moving>,
) {
    if mouse_input.just_released(MouseButton::Left) && !mouse.over_ui {
        if let Err(_) = query.get_single() {
            commands
                .spawn_bundle(GeometryBuilder::build_as(
                    &CustomShapeRaw {
                        segments: vec![],
                        closed: false,
                    },
                    DrawMode::Stroke(StrokeMode::color(Color::rgba_u8(tool.color[0], tool.color[1], tool.color[2], tool.color[3]))),
                    Transform::from_translation(mouse.position.extend(0.0)),
                ))
                .insert(CustomShape { segments: vec![Line(Point2D::zero())] })
                .insert(ShapeBase {
                    name: None,
                    originx: Vec3::ZERO,
                })
                .insert(Moving {
                    origin: mouse.position,
                });
        }
    }
}

fn custom_shape_handle_update(
    mut orig: Local<Vec2>,
    mut commands: Commands,
    mouse_input: Res<Input<MouseButton>>,
    mouse: Res<MouseMovement>,
    mut query: Query<(&mut Path, &mut DrawMode, &mut CustomShape, &Moving, Entity)>,
) {
        if let Ok((mut path, mut draw_mode, mut custom_shape, moving, entity)) =
            query.get_single_mut()
        {
            /*if custom_shape.segments.len() == 1 {
                if mouse_input.just_released(MouseButton::Left) && !mouse.over_ui {
                    custom_shape.segments.push(Line(point_from_positions(mouse.position, moving.origin)));
                }
                else {
                    let last = custom_shape.segments.len()-1;
                    custom_shape.segments[last] = Line(point_from_positions(mouse.position, moving.origin));
                    *path = ShapePath::build_as(&CustomShapeRaw {
                        segments: custom_shape.segments.clone(),
                        closed: false,
                    });
                }
                return;

            }*/
            if mouse_input.just_pressed(MouseButton::Left) && !mouse.over_ui{
                *orig = mouse.position;
            }

            if mouse_input.just_released(MouseButton::Left) && !mouse.over_ui {
            let mut closed = false;
            if orig.distance(moving.origin) <= 10.0 {
                closed = true;
                *draw_mode = DrawMode::Fill(FillMode::color(Color::CRIMSON));
                commands.entity(entity).remove::<Moving>().insert_bundle(PickableBundle::default());
            } else {
                custom_shape.segments.push(Line(point_from_positions(mouse.position, moving.origin)));
            }
            *path = ShapePath::build_as(&CustomShapeRaw {
                segments: custom_shape.segments.clone(),
                closed,
            });
        } else if !mouse_input.pressed(MouseButton::Left) {
                let last = custom_shape.segments.len()-1;
                custom_shape.segments[last] = Line(point_from_positions(mouse.position, moving.origin));
                *path = ShapePath::build_as(&CustomShapeRaw {
                    segments: custom_shape.segments.clone(),
                    closed: false,
                });
            } else {
                let last = custom_shape.segments.len()-1;
                custom_shape.segments[last] = QuadraticBezier { ctrl: rotate_around_pivot(mouse.position, moving.origin, *orig), to: point_from_positions(*orig, moving.origin) };
                *path = ShapePath::build_as(&CustomShapeRaw {
                    segments: custom_shape.segments.clone(),
                    closed: false,
                });
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

fn should_handle_primitive(tool: Res<Tool>) -> bool {
    return match tool.tool {
        ToolType::Primitive(_) => true,
        _ => false,
    };
}
fn should_handle_custom_shape(tool: Res<Tool>) -> bool {
    return tool.tool == ToolType::CustomShape;
}
#[derive(Debug, Clone, PartialEq)]
struct CustomShapeRaw {
    pub segments: Vec<ShapeSegment>,
    pub closed: bool,
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

impl Geometry for CustomShapeRaw {
    fn add_geometry(&self, b: &mut Builder) {
        b.begin(Point2D::new(0.0, 0.0));
        for segment in self.segments.iter() {
            match *segment {
                Line(end) => b.line_to(end),
                QuadraticBezier { ctrl, to } => b.quadratic_bezier_to(ctrl, to),
                CubicBezier { ctrl, ctrl2, to } => b.cubic_bezier_to(ctrl, ctrl2, to),
            };
        }
        b.end(self.closed);
    }
}

fn point_from_positions(mouse: Vec2, origin: Vec2) -> Point<f32> {
     Point2D::new(
        mouse.x - origin.x,
        mouse.y - origin.y,
    )
}

fn rotate_around_pivot(mouse: Vec2, origin: Vec2, pivot: Vec2) -> Point<f32> {
    Point2D::new(
        -mouse.x - origin.x + 2.0*pivot.x,
        -mouse.y - origin.y + 2.0*pivot.y,
    )
}
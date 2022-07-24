use crate::helpers::{point_from_positions, rotate_around_pivot};
use crate::ShapeSegment::*;
use crate::{MouseMovement, Moving, ShapeBase, Tool};
use bevy::prelude::*;
use bevy_mod_picking::PickableBundle;
use bevy_prototype_lyon::prelude::tess::math::Point;
use bevy_prototype_lyon::prelude::tess::path::path::Builder;
use bevy_prototype_lyon::prelude::{
    DrawMode, FillMode, Geometry, GeometryBuilder, Path, ShapePath, StrokeMode,
};

#[derive(Component)]
pub struct CustomShape {
    segments: Vec<ShapeSegment>,
}

pub fn custom_shape_handle_creation(
    mut commands: Commands,
    mouse_input: Res<Input<MouseButton>>,
    mouse: Res<MouseMovement>,
    tool: Res<Tool>,
    query: Query<&Moving>,
) {
    if mouse_input.just_released(MouseButton::Left) && !mouse.over_ui && query.get_single().is_err()
    {
        commands
            .spawn_bundle(GeometryBuilder::build_as(
                &CustomShapeRaw {
                    segments: vec![],
                    closed: false,
                },
                DrawMode::Stroke(StrokeMode::color(Color::rgba_u8(
                    tool.color[0],
                    tool.color[1],
                    tool.color[2],
                    tool.color[3],
                ))),
                Transform::from_translation(mouse.position.extend(0.0)),
            ))
            .insert(CustomShape {
                segments: vec![Line(Point::zero())],
            })
            .insert(ShapeBase {
                name: None,
                originx: Vec3::ZERO,
            })
            .insert(Moving {
                origin: mouse.position,
            });
    }
}

pub fn custom_shape_handle_update(
    mut orig: Local<Vec2>,
    mut commands: Commands,
    mouse_input: Res<Input<MouseButton>>,
    mouse: Res<MouseMovement>,
    mut query: Query<(&mut Path, &mut DrawMode, &mut CustomShape, &Moving, Entity)>,
    tool: Res<Tool>,
) {
    if let Ok((mut path, mut draw_mode, mut custom_shape, moving, entity)) = query.get_single_mut()
    {
        if mouse_input.just_pressed(MouseButton::Left) && !mouse.over_ui {
            *orig = mouse.position;
        }

        if mouse_input.just_released(MouseButton::Left) && !mouse.over_ui {
            let mut closed = false;
            if orig.distance(moving.origin) <= 10.0 {
                closed = true;
                *draw_mode = DrawMode::Fill(FillMode::color(Color::rgba_u8(
                    tool.color[0],
                    tool.color[1],
                    tool.color[2],
                    tool.color[3],
                )));
                commands
                    .entity(entity)
                    .remove::<Moving>()
                    .insert_bundle(PickableBundle::default());
            } else {
                custom_shape
                    .segments
                    .push(Line(point_from_positions(mouse.position, moving.origin)));
            }
            *path = ShapePath::build_as(&CustomShapeRaw {
                segments: custom_shape.segments.clone(),
                closed,
            });
        } else if !mouse_input.pressed(MouseButton::Left) {
            let last = custom_shape.segments.len() - 1;
            custom_shape.segments[last] = Line(point_from_positions(mouse.position, moving.origin));
            *path = ShapePath::build_as(&CustomShapeRaw {
                segments: custom_shape.segments.clone(),
                closed: false,
            });
        } else {
            let last = custom_shape.segments.len() - 1;
            custom_shape.segments[last] = QuadraticBezier {
                ctrl: rotate_around_pivot(mouse.position, moving.origin, *orig),
                to: point_from_positions(*orig, moving.origin),
            };
            *path = ShapePath::build_as(&CustomShapeRaw {
                segments: custom_shape.segments.clone(),
                closed: false,
            });
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomShapeRaw {
    pub segments: Vec<ShapeSegment>,
    pub closed: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ShapeSegment {
    Line(Point),
    QuadraticBezier {
        ctrl: Point,
        to: Point,
    },
    CubicBezier {
        ctrl: Point,
        ctrl2: Point,
        to: Point,
    },
}

impl Geometry for CustomShapeRaw {
    fn add_geometry(&self, b: &mut Builder) {
        b.begin(Point::new(0.0, 0.0));
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

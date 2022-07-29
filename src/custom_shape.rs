use crate::helpers::{point_from_positions, rotate_around_pivot};
use crate::tess::geom::euclid::Size2D;
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
    pub segments: Vec<ShapeSegment>,
    pub origin: Vec2,
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
                    origin: Vec2::ZERO,
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
                origin: Vec2::ZERO,
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
                origin: custom_shape.origin,
            });
        } else if !mouse_input.pressed(MouseButton::Left) {
            let last = custom_shape.segments.len() - 1;
            custom_shape.segments[last] = Line(point_from_positions(mouse.position, moving.origin));
            *path = ShapePath::build_as(&CustomShapeRaw {
                segments: custom_shape.segments.clone(),
                closed: false,
                origin: custom_shape.origin,
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
                origin: custom_shape.origin,
            });
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomShapeRaw {
    pub segments: Vec<ShapeSegment>,
    pub closed: bool,
    pub origin: Vec2,
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
        let v = Point::from((self.origin.x, self.origin.y));
        let o = Size2D::from((self.origin.x, self.origin.y));
        b.begin(v);
        for segment in self.segments.iter() {
            match *segment {
                Line(end) => b.line_to(end + o),
                QuadraticBezier { ctrl, to } => b.quadratic_bezier_to(ctrl + o, to + o),
                CubicBezier { ctrl, ctrl2, to } => b.cubic_bezier_to(ctrl + o, ctrl2 + o, to + o),
            };
        }
        b.end(self.closed);
    }
}

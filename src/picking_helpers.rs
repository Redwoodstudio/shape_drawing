use crate::MouseMovement;
use bevy::app::PluginGroupBuilder;
use bevy::ecs::schedule::ShouldRun;
use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;
use bevy::ui::FocusPolicy;
use bevy_mod_picking::{
    mesh_events_system, mesh_focus, mesh_selection, pause_for_picking_blockers, Hover, NoDeselect,
    PausedForBlockers, PickableBundle, PickableMesh, PickingEvent, PickingPlugin,
    PickingPluginsState, PickingSystem, Selection,
};
use bevy_prototype_lyon::draw::FillMode;
use bevy_prototype_lyon::path::ShapePath;
use bevy_prototype_lyon::prelude::{DrawMode, GeometryBuilder, Path, RectangleOrigin, StrokeMode};
use bevy_prototype_lyon::shapes;
use itertools::Itertools;

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

#[derive(Component)]
struct HighlightRect;

#[derive(Component)]
pub struct TransformScalePick {
    pub location: (f32, f32),
    pub entity: Option<Entity>,
    pub size: Vec2,
}

#[derive(Component)]
pub struct TransformRotationPick {
    pub location: (f32, f32),
    pub entity: Option<Entity>,
}
pub fn spawn_highlight_rectangle(mut commands: Commands) {
    let bundle = GeometryBuilder::build_as(
        &shapes::Rectangle {
            extents: Default::default(),
            origin: RectangleOrigin::Center,
        },
        DrawMode::Stroke(StrokeMode::color(Color::RED)),
        Default::default(),
    );
    commands
        .spawn_bundle(bundle)
        .insert(HighlightRect)
        .with_children(|parent| {
            for x in (-1..2)
                .chain(-1..2)
                .tuple_combinations::<(i32, i32)>()
                .unique()
                .filter_map(|p| {
                    if p != (0, 0) {
                        Some((p.0 as f32, p.1 as f32))
                    } else {
                        None
                    }
                })
            {
                let b = GeometryBuilder::build_as(
                    &shapes::Rectangle {
                        extents: Vec2::new(10.0, 10.0),
                        origin: RectangleOrigin::Center,
                    },
                    DrawMode::Outlined {
                        fill_mode: FillMode::color(Color::Rgba {
                            red: 0.0,
                            green: 0.0,
                            blue: 0.0,
                            alpha: 0.0,
                        }),
                        outline_mode: StrokeMode::color(Color::RED),
                    },
                    Transform::from_translation(Vec3::new(0.0, 0.0, 100.0)),
                );
                parent
                    .spawn_bundle(b)
                    .insert(TransformScalePick {
                        location: x,
                        entity: None,
                        size: Vec2::ZERO,
                    })
                    .insert(Visibility { is_visible: false })
                    .insert_bundle(PickableBundle::default())
                    .insert(NoDeselect)
                    .with_children(|p| {
                        let b = GeometryBuilder::build_as(
                            &shapes::Rectangle {
                                extents: Vec2::new(20.0, 20.0),
                                origin: RectangleOrigin::Center,
                            },
                            DrawMode::Outlined {
                                fill_mode: FillMode::color(Color::Rgba {
                                    red: 0.0,
                                    green: 0.0,
                                    blue: 0.0,
                                    alpha: 0.0,
                                }),
                                outline_mode: StrokeMode::color(Color::RED),
                            },
                            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                        );
                        p.spawn_bundle(b)
                            .insert(TransformRotationPick {
                                location: x,
                                entity: None,
                            })
                            .insert_bundle(PickableBundle::default())
                            .insert(NoDeselect);
                    });
            }
        });
}

fn highlight_selected(
    points: Res<Assets<Mesh>>,
    handles: Query<
        (&Selection, &Mesh2dHandle, &Transform, Entity),
        (
            Without<HighlightRect>,
            (Without<TransformScalePick>, Without<TransformRotationPick>),
        ),
    >,
    mut rect: Query<
        (&mut Transform, &mut Path, &DrawMode),
        (
            With<HighlightRect>,
            (Without<TransformScalePick>, Without<TransformRotationPick>),
        ),
    >,
    mut pickers: Query<
        (&mut Transform, &mut TransformScalePick, &mut Visibility),
        (Without<HighlightRect>, Without<TransformRotationPick>),
    >,
    mut rot_pickers: Query<
        (&mut Transform, &mut TransformRotationPick),
        (Without<HighlightRect>, Without<TransformScalePick>),
    >,
    cam: Query<&OrthographicProjection>,
) {
    if let Ok((mut rect_transform, mut path, draw_mode)) = rect.get_single_mut() {
        if let Some((handle, transform, entity)) = handles
            .iter()
            .filter_map(|(n, x, y, e)| {
                if n.selected() {
                    return Some((x, y, e));
                }
                None
            })
            .next()
        {
            if let Some(mesh) = points.get(&handle.0.clone()) {
                if let Some(aabb) = mesh.compute_aabb() {
                    *rect_transform = transform
                        .with_translation(
                            transform.translation
                                + Vec3::from(transform.rotation.mul_vec3a(aabb.center))
                                    * transform.scale
                                    * Vec3::new(1.0, 1.0, 0.0)
                                + Vec3::new(0.0, 0.0, 100.0),
                        )
                        .with_scale(Vec3::splat(1.0));

                    if let DrawMode::Stroke(stroke) = draw_mode {
                        *path = ShapePath::build_as(&shapes::Rectangle {
                            extents: aabb.half_extents.truncate()
                                * 2.0
                                * transform.scale.truncate()
                                + Vec2::splat(stroke.options.line_width),
                            origin: RectangleOrigin::Center,
                        });
                    }
                    for (mut p_transform, mut p, mut visibility) in pickers.iter_mut() {
                        *p_transform = Transform::from_translation(Vec3::new(
                            p.location.0 * aabb.half_extents.x * transform.scale.x,
                            p.location.1 * aabb.half_extents.y * transform.scale.y,
                            0.0,
                        ));
                        p.entity = Some(entity);
                        p.size = aabb.half_extents.truncate() * 2.0 * transform.scale.truncate();
                        visibility.is_visible = true;
                    }
                    if let Ok(cam) = cam.get_single() {
                        for (mut p_transform, mut p) in rot_pickers.iter_mut() {
                            *p_transform = Transform::from_translation(Vec3::new(
                                p.location.0 * 3.125 * cam.scale,
                                p.location.1 * 3.125 * cam.scale,
                                -1.0,
                            ));
                            p.entity = Some(entity);
                        }
                    }
                }
            }
        } else {
            *path = ShapePath::build_as(&shapes::Rectangle {
                extents: Vec2::ZERO,
                origin: RectangleOrigin::Center,
            });
            for (_, mut p, mut visibility) in pickers.iter_mut() {
                p.entity = None;
                visibility.is_visible = false;
            }
        }
    }
}

fn scale_pickers(
    mut rect: Query<
        &mut DrawMode,
        (
            With<HighlightRect>,
            Without<TransformScalePick>,
            Without<TransformRotationPick>,
        ),
    >,
    mut pickers: Query<
        (&mut Path, &mut DrawMode),
        (With<TransformScalePick>, Without<TransformRotationPick>),
    >,
    mut rot_pickers: Query<
        (&mut Path, &mut DrawMode),
        (Without<TransformScalePick>, With<TransformRotationPick>),
    >,
    cam: Query<&OrthographicProjection>,
) {
    if let Ok(cam_proj) = cam.get_single() {
        for (mut picker, mut draw_mode) in pickers.iter_mut() {
            *picker = ShapePath::build_as(&shapes::Rectangle {
                extents: Vec2::new(15.0, 15.0) * cam_proj.scale,
                origin: RectangleOrigin::Center,
            });
            *draw_mode = DrawMode::Outlined {
                fill_mode: FillMode::color(Color::Rgba {
                    red: 0.0,
                    green: 0.0,
                    blue: 0.0,
                    alpha: 0.0,
                }),
                outline_mode: StrokeMode::new(Color::RED, cam_proj.scale * 2.0),
            };
        }
        for (mut picker, mut draw_mode) in rot_pickers.iter_mut() {
            *picker = ShapePath::build_as(&shapes::Rectangle {
                extents: Vec2::new(25.0, 25.0) * cam_proj.scale,
                origin: RectangleOrigin::Center,
            });
            *draw_mode = DrawMode::Outlined {
                fill_mode: FillMode::color(Color::Rgba {
                    red: 0.0,
                    green: 0.0,
                    blue: 0.0,
                    alpha: 0.0,
                }),
                outline_mode: StrokeMode::new(Color::BLUE, cam_proj.scale * 2.0),
            };
        }
        if let Ok(mut rect_draw) = rect.get_single_mut() {
            *rect_draw = DrawMode::Stroke(StrokeMode::new(Color::RED, cam_proj.scale * 3.0));
        }
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
                    )
                    .with_system(highlight_selected)
                    .with_system(scale_pickers),
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

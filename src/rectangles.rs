use std::{cmp::max, fmt::Write};

use bevy::{ecs::event::Events, prelude::*, window::WindowResized};
use bevy_prototype_lyon::prelude::*;
use rand::{thread_rng, Rng};

pub struct RectanglesPlugin;

impl Plugin for RectanglesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Stats>();
        app.add_startup_system(setup);
        app.add_system(bounds_updater);
        app.add_system(movement);
        app.add_system(collision_detection);
        app.add_system(mouse_handler);
        app.add_system(stats_system);
    }
}

struct Stats {
    count: u32,
}

impl Default for Stats {
    fn default() -> Self {
        Stats { count: 250 }
    }
}

#[derive(Component)]
struct StatsText;

#[derive(Component)]
struct RectangleObject {
    velocity: f32,
    width: f32,
    teleport_target: f32,
}

fn setup(
    mut commands: Commands,
    windows: Res<Windows>,
    stats: Res<Stats>,
    asset_server: Res<AssetServer>,
) {
    spawn_rectangles(&mut commands, &windows, stats.count);

    commands
        .spawn_bundle(TextBundle {
            text: Text {
                sections: vec![
                    TextSection {
                        value: "Rectangle count: ".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 40.0,
                            color: Color::BLACK,
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 40.0,
                            color: Color::BLACK,
                        },
                    },
                ],
                ..default()
            },
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px(0.),
                    ..default()
                },
                ..default()
            },
            ..default()
        })
        .insert(StatsText);
}

fn mouse_handler(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    mut stats: ResMut<Stats>,
    rectangles: Query<Entity, With<RectangleObject>>,
) {
    let old = stats.count;
    if mouse_button_input.just_released(MouseButton::Left) {
        stats.count = max(1, stats.count * 2);
        spawn_rectangles(&mut commands, &windows, stats.count - old);
    }
    if mouse_button_input.just_released(MouseButton::Right) {
        stats.count /= 2;
        despawn_rectangles(&mut commands, rectangles, old - stats.count);
    }
}

fn spawn_rectangles(commands: &mut Commands, windows: &Windows, num: u32) {
    let mut rng = thread_rng();
    let window = windows.get_primary().unwrap();
    let (width, height) = (window.width(), window.height());
    let teleport_target = -(width / 2.);

    let default_shape = shapes::Rectangle {
        extents: Vec2::ZERO,
        origin: RectangleOrigin::BottomLeft,
    };
    let default_draw_mode = DrawMode::Outlined {
        fill_mode: FillMode {
            options: FillOptions::default().with_intersections(false),
            color: Color::WHITE,
        },
        outline_mode: StrokeMode::new(Color::BLACK, 1.5),
    };

    for _ in 0..num {
        let dimensions = Vec2::splat(10. + rng.gen::<f32>() * 40.);
        commands
            .spawn_bundle(GeometryBuilder::build_as(
                &shapes::Rectangle {
                    extents: dimensions,
                    ..default_shape
                },
                default_draw_mode,
                Transform::from_translation(Vec3::new(
                    (rng.gen::<f32>() - 0.5) * width,
                    (rng.gen::<f32>() - 0.5) * height,
                    0.,
                )),
            ))
            .insert(RectangleObject {
                velocity: rng.gen_range(60.0..120.0),
                width: dimensions.x,
                teleport_target: teleport_target - dimensions.x,
            });
    }
}

fn despawn_rectangles(
    commands: &mut Commands,
    rectangles: Query<Entity, With<RectangleObject>>,
    num: u32,
) {
    for r in rectangles.iter().take(num as usize) {
        commands.entity(r).despawn();
    }
}

fn bounds_updater(
    resize_event: Res<Events<WindowResized>>,
    mut rectangles_query: Query<&mut RectangleObject>,
) {
    let mut reader = resize_event.get_reader();
    let target_event = reader
        .iter(&resize_event)
        .filter(|e| e.id.is_primary())
        .last();

    if let Some(e) = target_event {
        let teleport_target = -(e.width / 2.);
        rectangles_query.for_each_mut(|mut r| {
            r.teleport_target = teleport_target - r.width;
        });
    }
}

fn movement(time: Res<Time>, mut rectangles_query: Query<(&RectangleObject, &mut Transform)>) {
    rectangles_query.for_each_mut(|(r, mut transform)| {
        transform.translation.x -= r.velocity * time.delta_seconds();
    });
}

fn collision_detection(mut rectangles_query: Query<(&RectangleObject, &mut Transform)>) {
    rectangles_query.for_each_mut(|(r, mut transform)| {
        if transform.translation.x < r.teleport_target {
            transform.translation.x = -transform.translation.x;
        }
    });
}

fn stats_system(stats: Res<Stats>, mut query: Query<&mut Text, With<StatsText>>) {
    if !stats.is_changed() {
        return;
    }

    let mut text = query.single_mut();
    text.sections[1].value.clear();
    write!(text.sections[1].value, "{}", stats.count).unwrap();
}

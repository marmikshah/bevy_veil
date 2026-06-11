//! Two stacked overlays — the bug `bevy_veil` exists to kill.
//!
//! The first (large) overlay opens a second, smaller one on top. In raw
//! `bevy_ui` the first overlay's button would still be clickable around the
//! edges of the second; here the scrim occludes it. Meanwhile the spinning
//! sprite (driven by raw input via `ui_not_capturing`) freezes whenever any
//! overlay is open — the UI→gameplay gate.
//!
//! Run: `cargo run --example stacked`

use bevy::prelude::*;
use bevy_veil::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, VeilPlugin))
        .add_systems(Startup, (setup, open_first))
        .add_systems(Update, spin_sprite.run_if(ui_not_capturing))
        .run();
}

#[derive(Component)]
struct Spinner;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Spinner,
        Sprite {
            color: Color::srgb(0.45, 0.70, 1.0),
            custom_size: Some(Vec2::new(80.0, 80.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 200.0, 0.0),
    ));
}

/// The base overlay. Its "Open dialog" button stacks a second overlay on top.
fn open_first(mut commands: Commands) {
    overlay(&mut commands, "first")
        .title("FIRST")
        .body("hold ESC-free; press the button")
        .button("Open dialog", |c| {
            overlay(c, "second")
                .title("SECOND")
                .body("scrim below blocks the first's button")
                .button("Close", |c| {
                    c.queue(|world: &mut World| {
                        if let Some(top) = world.resource::<OverlayStack>().top() {
                            world.entity_mut(top).despawn();
                        }
                    });
                })
                .dismissable(true)
                .push();
        })
        .push();
}

/// Raw-input-style gameplay system: spins only while no overlay captures input.
fn spin_sprite(time: Res<Time>, mut spinner: Query<&mut Transform, With<Spinner>>) {
    for mut t in spinner.iter_mut() {
        t.rotate_z(time.delta_secs());
    }
}

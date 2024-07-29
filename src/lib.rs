#![feature(adt_const_params)]

#[cfg(feature = "dev")]
mod dev_tools;
mod game;
mod screen;
mod ui;

use bevy::{
    asset::AssetMetaCheck,
    audio::{AudioPlugin, Volume},
    prelude::*,
};
use serde::{Deserialize, Serialize};
use std::ops::Deref;

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Order new `AppStep` variants by adding them here:
        app.configure_sets(
            Update,
            (AppSet::TickTimers, AppSet::RecordInput, AppSet::Update).chain(),
        );

        let settings = GameSettings {
            global_volume_level: VolumeSetting::from_divisor_added(2),
            soundtrack_volume_level_relative: VolumeSetting::from_divisor_removed(VolumeSetting::DIFF),
            sfx_volume_level_relative: VolumeSetting::from_divisor_removed(VolumeSetting::DIFF / 2),
        };

        // Spawn the main camera.
        app.add_systems(Startup, spawn_camera);

        // Add Bevy plugins.
        app.add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics on web build on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "Bevy Jam 5".to_string(),
                        canvas: Some("#bevy".to_string()),
                        fit_canvas_to_parent: true,
                        // don't let browser steal common inputs (does nothing on native)
                        prevent_default_event_handling: true,
                        ..default()
                    }
                    .into(),
                    ..default()
                })
                .set(AudioPlugin {
                    global_volume: GlobalVolume {
                        volume: (&settings.global_volume_level).into(),
                    },
                    ..default()
                }),
        );

        app.insert_resource(settings);

        // Add other plugins.
        app.add_plugins((game::plugin, screen::plugin, ui::plugin));

        // Enable dev tools for dev builds.
        #[cfg(feature = "dev")]
        app.add_plugins(dev_tools::plugin);
    }
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum AppSet {
    /// Tick timers.
    TickTimers,
    /// Record player input.
    RecordInput,
    /// Do everything else (consider splitting this into further variants).
    Update,
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Camera"),
        Camera2dBundle::default(),
        // Render all UI to this camera.
        // Not strictly necessary since we only use one camera,
        // but if we don't use this component, our UI will disappear as soon
        // as we add another camera. This includes indirect ways of adding cameras like using
        // [ui node outlines](https://bevyengine.org/news/bevy-0-14/#ui-node-outline-gizmos)
        // for debugging. So it's good to have this here for future-proofing.
        IsDefaultUiCamera,
    ));
}

trait Bounded<T>: Deref<Target = u8> {
    const MIN: T;
    const MAX: T;
}
#[derive(
    Reflect, Serialize, Deserialize, Debug, Deref, Copy, Clone, Eq, PartialEq, Ord, PartialOrd,
)]
struct BoundedU8<const MIN: u8 = 0, const MAX: u8 = 255>(u8);
impl<const MIN: u8, const MAX: u8> Bounded<u8> for BoundedU8<MIN, MAX> {
    const MIN: u8 = MIN;
    const MAX: u8 = MAX;
}

impl<const A: u8, const B: u8> std::ops::Add<u8> for BoundedU8<A, B> {
    type Output = Self;
    fn add(self, rhs: u8) -> Self::Output {
        Self(self.saturating_add(rhs).min(Self::MAX))
    }
}
impl<const A: u8, const B: u8> std::ops::Sub<u8> for BoundedU8<A, B> {
    type Output = Self;
    fn sub(self, rhs: u8) -> Self::Output {
        Self(self.saturating_sub(rhs).max(Self::MIN))
    }
}
impl<const A: u8, const B: u8> From<u8> for BoundedU8<A, B> {
    fn from(value: u8) -> Self {
        assert!((A..=B).contains(&value));
        Self(value)
    }
}

trait LevelSetting: Deref<Target: Bounded<u8>> + Sized {
    const MIN: u8 = Self::Target::MIN;
    const MAX: u8 = Self::Target::MAX;
    const DIFF: u8 = Self::MAX - Self::MIN;
    #[allow(unused)]
    fn fraction(&self) -> f32 {
        (*self.deref().deref() - Self::MIN) as f32 / Self::DIFF as f32
    }
    #[allow(unused)]
    fn from_fraction(frac: f32) -> Self {
        assert!((0f32..=1f32).contains(&frac));
        let diff_proportion = Self::DIFF as f32 * frac;
        Self::from_raw(diff_proportion as u8 + Self::MIN)
    }
    /// Divisor, adding to min
    #[allow(unused)]
    fn from_divisor_added(divisor: u8) -> Self {
        assert_ne!(divisor, 0);
        let diff_proportion = Self::DIFF / divisor;
        Self::from_raw(diff_proportion + Self::MIN)
    }
    /// Divisor, subtracting from max
    #[allow(unused)]
    fn from_divisor_removed(divisor: u8) -> Self {
        assert_ne!(divisor, 0);
        let diff_proportion = Self::DIFF / divisor;
        Self::from_raw(Self::MAX - diff_proportion)
    }
    #[allow(unused)]
    fn from_max() -> Self {
        Self::from_raw(Self::MAX)
    }
    /// Display level as percentage
    fn percent_display(&self) -> String {
        format!("{:.1}%", self.fraction() * 100f32)
    }
    fn from_raw(value: u8) -> Self;
}

#[derive(Serialize, Deserialize, Deref, Clone, Debug, Eq, PartialEq, Reflect)]
struct VolumeSetting(BoundedU8<0, 10>);
impl LevelSetting for VolumeSetting {
    fn from_raw(value: u8) -> Self {
        Self(value.into())
    }
}
impl From<&VolumeSetting> for Volume {
    fn from(value: &VolumeSetting) -> Self {
        const MAX_VOLUME: f32 = 0.35;
        // note: not sure if this is "different" between browser and desktop build
        Volume::new(value.fraction() * MAX_VOLUME)
    }
}

#[derive(Component, Debug, Clone, Copy, Eq, PartialEq, Reflect)]
#[reflect(Component)]
enum BinaryAdjustment {
    Up,
    Down,
}

#[derive(Component, Debug, Clone, Copy, Eq, PartialEq, Reflect)]
struct LevelSettingAction<S> {
    adjustment: BinaryAdjustment,
    scope: S,
}

#[derive(Serialize, Deserialize, Resource, Debug, Clone, Eq, PartialEq, Reflect)]
struct GameSettings {
    global_volume_level: VolumeSetting,
    soundtrack_volume_level_relative: VolumeSetting,
    sfx_volume_level_relative: VolumeSetting,
    // camera shake / vfx off?
}
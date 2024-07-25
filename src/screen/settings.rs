use crate::screen::Screen;
use crate::ui::prelude::*;
use crate::{GameSettings, MAX_VOLUME, VOLUME_INCREMENTS};
use bevy::audio::Volume;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Settings), enter_settings)
        .add_systems(
            Update,
            (handle_volume_action, handle_settings_action).run_if(in_state(Screen::Settings)),
        )
        .register_type::<VolumeAction>()
        .register_type::<SettingsAction>();
}

#[derive(Component, Debug, Clone, Copy, Eq, PartialEq, Reflect)]
#[reflect(Component)]
enum SettingsAction {
    Back,
}

#[derive(Component, Debug, Clone, Copy, Eq, PartialEq, Reflect)]
#[reflect(Component)]
enum VolumeAction {
    Up,
    Down,
}

fn enter_settings(mut commands: Commands, settings: Res<GameSettings>) {
    commands
        .ui_root()
        .insert(StateScoped(Screen::Settings))
        .with_children(|children| {
            children.header("Settings");

            children.label("Global Volume").with_children(|volume| {
                volume
                    .spawn(NodeBundle {
                        style: Style {
                            width: Val::Px(500.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|volume_text| {
                        volume_text.spawn((
                            TextBundle::from_section(
                                volume_level_to_display(settings.volume_level),
                                TextStyle {
                                    font_size: 16.0,
                                    color: Color::WHITE,
                                    ..default()
                                },
                            ),
                            VolumeDisplayMarker,
                        ));
                    });
                volume.button("Volume DOWN").insert(VolumeAction::Down);
                volume.button("Volume UP").insert(VolumeAction::Up);
            });

            children.button("Back").insert(SettingsAction::Back);
        });
}

#[derive(Component, Debug, Clone, Copy, Eq, PartialEq, Reflect)]
#[reflect(Component)]
struct VolumeDisplayMarker;

fn handle_volume_action(
    mut global_volume: ResMut<GlobalVolume>,
    mut settings: ResMut<GameSettings>,
    mut text_query: Query<&mut Text, With<VolumeDisplayMarker>>,
    mut button_query: InteractionQuery<&VolumeAction>,
) {
    for (interaction, action) in &mut button_query {
        if matches!(interaction, Interaction::Pressed) {
            // update actual setting
            settings.volume_level = match action {
                VolumeAction::Up => settings
                    .volume_level
                    .saturating_add(1)
                    .min(VOLUME_INCREMENTS),
                VolumeAction::Down => settings.volume_level.saturating_sub(1),
            };
            // show in ui
            text_query
                .get_single_mut() // assume only one, since we only have one marker
                .unwrap()
                .sections
                .first_mut() // only one section in text field
                .unwrap()
                .value = volume_level_to_display(settings.volume_level);
            // apply to audio
            global_volume.volume =
                Volume::new(MAX_VOLUME * level_to_fraction(settings.volume_level));
        }
    }
}

fn volume_level_to_display(level: u8) -> String {
    format!("{:.3}", level_to_fraction(level))
}

fn level_to_fraction(level: u8) -> f32 {
    level as f32 / VOLUME_INCREMENTS as f32
}

fn handle_settings_action(
    mut next_screen: ResMut<NextState<Screen>>,
    mut button_query: InteractionQuery<&SettingsAction>,
) {
    for (interaction, action) in &mut button_query {
        if matches!(interaction, Interaction::Pressed) {
            match action {
                SettingsAction::Back => next_screen.set(Screen::Title),
            }
        }
    }
}

use crate::screen::Screen;
use crate::ui::prelude::*;
use crate::{BinaryAdjustment, GameSettings, LevelSetting, LevelSettingAction};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Settings), enter_settings)
        .add_systems(
            Update,
            (handle_volume_action, handle_settings_action).run_if(in_state(Screen::Settings)),
        )
        .register_type::<LevelSettingAction<VolumeSettingScope>>()
        .register_type::<ScreenAction>();
}

#[derive(Component, Debug, Clone, Copy, Eq, PartialEq, Reflect)]
#[reflect(Component)]
enum ScreenAction {
    Back,
}

#[derive(Component, Debug, Clone, Copy, Eq, PartialEq, Reflect)]
enum VolumeSettingScope {
    Global,
    Soundtrack,
    Sfx,
}

fn enter_settings(mut commands: Commands, settings: Res<GameSettings>) {
    commands
        .ui_root()
        .insert(StateScoped(Screen::Settings))
        .with_children(|children| {
            children.header("Settings");

            children.settings_field(
                "Global audio volume",
                settings.global_volume_level.percent_display(),
                VolumeSettingScope::Global,
            );

            children.settings_field(
                "Music volume (relative)",
                settings.soundtrack_volume_level_relative.percent_display(),
                VolumeSettingScope::Soundtrack,
            );

            children.settings_field(
                "SFX volume (relative)",
                settings.sfx_volume_level_relative.percent_display(),
                VolumeSettingScope::Sfx,
            );

            children.button("Back").insert(ScreenAction::Back);
        });
}

fn handle_volume_action(
    mut global_volume: ResMut<GlobalVolume>,
    mut settings: ResMut<GameSettings>,
    mut text_query: Query<(&mut Text, &VolumeSettingScope)>,
    mut button_query: InteractionQuery<&LevelSettingAction<VolumeSettingScope>>,
) {
    for &LevelSettingAction { adjustment, scope } in button_query
        .iter_mut()
        .filter_map(|(i, b)| matches!(i, Interaction::Pressed).then_some(b))
    {
        // update record
        let (setting_level, update_global) = match scope {
            VolumeSettingScope::Global => (&mut settings.global_volume_level, true),
            VolumeSettingScope::Soundtrack => {
                (&mut settings.soundtrack_volume_level_relative, false)
            }
            VolumeSettingScope::Sfx => (&mut settings.sfx_volume_level_relative, false),
        };
        setting_level.0 = match adjustment {
            // type ensures bound
            BinaryAdjustment::Up => setting_level.0 + 1u8,
            BinaryAdjustment::Down => setting_level.0 - 1u8,
        };
        // update ui
        text_query
            .iter_mut()
            .find_map(|(text, &test)| (test == scope).then_some(text))
            .unwrap() // assume exactly one, since we (should) only have one marker
            .sections
            .first_mut() // only one section in text field
            .unwrap()
            .value = setting_level.percent_display();
        info!(
            "Updated setting of {:?} to level {:.}.",
            scope, setting_level.0 .0
        );
        // apply elsewhere?
        if update_global {
            global_volume.volume = (&settings.global_volume_level).into();
        }
    }
}

fn handle_settings_action(
    mut next_screen: ResMut<NextState<Screen>>,
    mut button_query: InteractionQuery<&ScreenAction>,
) {
    for (interaction, action) in &mut button_query {
        if matches!(interaction, Interaction::Pressed) {
            match action {
                ScreenAction::Back => next_screen.set(Screen::Title),
            }
        }
    }
}

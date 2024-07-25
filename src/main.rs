// Disable console on Windows for non-dev builds. ?: why is this here?
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

use bevy::prelude::*;
use bevy_jam_5::AppPlugin;

fn main() -> AppExit {
    App::new().add_plugins(AppPlugin).run()
}

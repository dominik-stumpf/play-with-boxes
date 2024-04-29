#![allow(clippy::type_complexity)]

// mod actions;
// mod audio;
mod game;
mod loading;
mod menu;
// mod player;

use std::time::Duration;

// use crate::actions::ActionsPlugin;
// use crate::audio::InternalAudioPlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;
// use crate::player::PlayerPlugin;
use crate::game::GamePlugin;

use bevy::app::App;
#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    /// During the loading State the LoadingPlugin will load our assets
    #[default]
    AssetLoading,
    /// During this State the actual game logic is executed
    Playing,
    /// Here the menu is drawn and waiting for player interaction
    Menu,
    /// Looking for a match after menu actions
    Matchmaking,
}

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum ClientTypeState {
    /// We have the client and the server running inside the same app.
    /// The server will also act as a client.
    #[cfg(not(target_family = "wasm"))]
    HostServer { client_id: Option<u64> },
    /// The program will act as a client
    Client { client_id: Option<u64> },
    #[default]
    /// Game is not running
    NotInGame,
}

pub struct GameSetupPlugin;

impl Plugin for GameSetupPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .init_state::<ClientTypeState>()
            .add_plugins((
                LoadingPlugin,
                MenuPlugin,
                GamePlugin,
                // InternalAudioPlugin,
                // ActionsPlugin,
                // PlayerPlugin,
            ));
        #[cfg(debug_assertions)]
        {
            app.add_plugins((
                FrameTimeDiagnosticsPlugin,
                LogDiagnosticsPlugin {
                    wait_duration: Duration::from_secs(5),
                    ..default()
                },
            ));
        }
    }
}

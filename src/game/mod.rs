#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

//! Run with
//! - `cargo run -- server`
//! - `cargo run -- client -c 1`
use std::net::SocketAddr;
use std::str::FromStr;

use bevy::asset::ron;
use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use bevy::DefaultPlugins;
// use bevy_inspector_egui::quick::{FilterQueryInspectorPlugin, WorldInspectorPlugin};
// use clap::{Parser, ValueEnum};
use lightyear::prelude::client::{
    InterpolationConfig, InterpolationDelay, NetConfig, PredictionConfig,
};
use lightyear::prelude::server::LeafwingInputPlugin;
// use serde::{Deserialize, Serialize};

use lightyear::prelude::{Mode, TransportConfig};
use lightyear::shared::log::add_log_layer;
use lightyear::transport::LOCAL_SOCKET;

use crate::ClientTypeState;

use self::client::ExampleClientPlugin;
use self::protocol::{protocol, MyProtocol, PlayerActions, PlayerId};
use self::server::ExampleServerPlugin;
use self::settings::*;
use self::shared::{shared_config, SharedPlugin};

mod client;
mod protocol;
mod server;
mod settings;
mod shared;

// #[derive(Parser, PartialEq, Debug)]
// enum Cli {
//     /// We have the client and the server running inside the same app.
//     /// The server will also act as a client.
//     #[cfg(not(target_family = "wasm"))]
//     HostServer {
//         #[arg(short, long, default_value = None)]
//         client_id: Option<u64>,
//     },
//     /// The program will act as a client
//     Client {
//         #[arg(short, long, default_value = None)]
//         client_id: Option<u64>,
//     },
// }

// fn main() {
//     // cfg_if::cfg_if! {
//     //     if #[cfg(target_family = "wasm")] {
//     //         let client_id = rand::random::<u64>();
//     //         let cli = Cli::Client {
//     //             client_id: Some(client_id)
//     //         };
//     //     } else {
//     //         let cli = Cli::parse();
//     //     }
//     // }
//     let settings_str = include_str!("../../assets/settings.ron");
//     let settings = ron::de::from_str::<Settings>(settings_str).unwrap();
//     run(settings, cli);
// }

fn run(settings: Settings, client_type_state: ClientTypeState) {
    match client_type_state {
        #[cfg(not(target_family = "wasm"))]
        ClientTypeState::HostServer { client_id } => {
            let client_net_config = NetConfig::Local {
                id: client_id.unwrap_or(settings.client.client_id),
            };
            let mut app = combined_app(settings, vec![], client_net_config);
            app.run();
        }
        ClientTypeState::Client { client_id } => {
            let server_addr = SocketAddr::new(
                settings.client.server_addr.into(),
                settings.client.server_port,
            );
            // use the cli-provided client id if it exists, otherwise use the settings client id
            let client_id = client_id.unwrap_or(settings.client.client_id);
            let net_config = get_client_net_config(&settings, client_id);
            let mut app = client_app(settings, net_config);
            app.run();
        }
        ClientTypeState::NotInGame => {
            panic!("Cannot be in game with unspecified client type state")
        }
    }
}

/// Build the client app
fn client_app(settings: Settings, net_config: client::NetConfig) -> App {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.build().set(LogPlugin {
        level: Level::INFO,
        filter: "wgpu=error,bevy_render=info,bevy_ecs=warn".to_string(),
        update_subscriber: Some(add_log_layer),
    }));
    // if settings.client.inspector {
    //     app.add_plugins(FilterQueryInspectorPlugin::<With<PlayerId>>::default());
    // }
    let client_config = client::ClientConfig {
        shared: shared_config(Mode::Separate),
        net: net_config,
        prediction: PredictionConfig {
            input_delay_ticks: settings.client.input_delay_ticks,
            correction_ticks_factor: settings.client.correction_ticks_factor,
            ..default()
        },
        interpolation: InterpolationConfig {
            delay: InterpolationDelay::default().with_send_interval_ratio(2.0),
            ..default()
        },
        replication: client::ReplicationConfig {
            // enable send because we pre-spawn entities on the client
            enable_send: true,
            enable_receive: true,
        },
        ..default()
    };
    let plugin_config = client::PluginConfig::new(client_config, protocol());
    app.add_plugins((
        client::ClientPlugin::new(plugin_config),
        ExampleClientPlugin,
        SharedPlugin,
    ));
    app
}

/// An app that contains both the client and server plugins
#[cfg(not(target_family = "wasm"))]
fn combined_app(
    settings: Settings,
    extra_transport_configs: Vec<TransportConfig>,
    client_net_config: client::NetConfig,
) -> App {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.build().set(LogPlugin {
        level: Level::INFO,
        filter: "wgpu=error,bevy_render=info,bevy_ecs=warn".to_string(),
        update_subscriber: Some(add_log_layer),
    }));
    // if settings.client.inspector {
    //     app.add_plugins(FilterQueryInspectorPlugin::<With<PlayerId>>::default());
    // }

    // server plugin
    let mut net_configs = get_server_net_configs(&settings);
    let extra_net_configs = extra_transport_configs.into_iter().map(|c| {
        build_server_netcode_config(settings.server.conditioner.as_ref(), &settings.shared, c)
    });
    net_configs.extend(extra_net_configs);
    let server_config = server::ServerConfig {
        shared: shared_config(Mode::HostServer),
        net: net_configs,
        replication: lightyear::server::replication::ReplicationConfig {
            enable_send: true,
            enable_receive: true,
        },
        ..default()
    };
    app.add_plugins((
        server::ServerPlugin::new(server::PluginConfig::new(server_config, protocol())),
        ExampleServerPlugin {
            predict_all: settings.server.predict_all,
        },
    ));

    // client plugin
    let client_config = client::ClientConfig {
        shared: shared_config(Mode::HostServer),
        net: client_net_config,
        prediction: PredictionConfig {
            input_delay_ticks: settings.client.input_delay_ticks,
            correction_ticks_factor: settings.client.correction_ticks_factor,
            ..default()
        },
        interpolation: InterpolationConfig {
            delay: InterpolationDelay::default().with_send_interval_ratio(2.0),
            ..default()
        },
        replication: client::ReplicationConfig {
            // enable send because we pre-spawn entities on the client
            enable_send: true,
            enable_receive: true,
        },
        ..default()
    };
    let plugin_config = client::PluginConfig::new(client_config, protocol());
    app.add_plugins((
        client::ClientPlugin::new(plugin_config),
        ExampleClientPlugin,
    ));
    // shared plugin
    app.add_plugins(SharedPlugin);
    app
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        // app.init_state::<GameState>()
        //     .init_state::<ClientTypeState>()
        //     .add_plugins((
        //         LoadingPlugin,
        //         MenuPlugin,
        //         InternalAudioPlugin,
        //         // ActionsPlugin,
        //         // PlayerPlugin,
        //     ));
        // #[cfg(debug_assertions)]
        // {
        //     app.add_plugins((
        //         FrameTimeDiagnosticsPlugin,
        //         LogDiagnosticsPlugin {
        //             wait_duration: Duration::from_secs(5),
        //             ..default()
        //         },
        //     ));
        // }
    }
}

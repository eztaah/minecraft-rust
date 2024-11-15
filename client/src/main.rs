#![allow(clippy::type_complexity)]

mod camera;
mod constants;
mod game;
mod input;
mod lighting;
mod menu;
mod network;
mod player;
mod splash_screen;
mod ui;
mod world;

use bevy::{
    prelude::*,
    render::{
        render_resource::WgpuFeatures,
        settings::{RenderCreation, WgpuSettings},
        RenderPlugin,
    },
};
use input::{data::GameAction, keyboard::get_bindings};
use menu::settings::{DisplayQuality, Volume};
use serde::{Deserialize, Serialize};
use shared::world::{load_blocks_items, BlockData, ItemData, Registry};
use std::collections::BTreeMap;

#[derive(Component)]
pub struct MenuCamera;

pub const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

// Enum that will be used as a global state for the game
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum GameState {
    Splash,
    #[default]
    Menu,
    PreGameLoading,
    Game,
}

#[derive(Event)]
pub struct LoadWorldEvent {
    pub world_name: String,
}

#[derive(Resource, Serialize, Deserialize)]
pub struct KeyMap {
    pub map: BTreeMap<GameAction, Vec<KeyCode>>,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            // Ensures that pixel-art textures will remain pixelated, and not become a blurry mess
            .set(ImagePlugin::default_nearest())
            .set(RenderPlugin {
                render_creation: RenderCreation::Automatic(WgpuSettings {
                    // WARNING: This is a native-only feature. It will not work with WebGL or WebGPU
                    features: WgpuFeatures::POLYGON_MODE_LINE,
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                file_path: "../data".to_string(),
                ..Default::default()
            }),
    );
    app.add_event::<LoadWorldEvent>();
    network::add_base_netcode(&mut app);
    app.insert_resource(DisplayQuality::Medium)
        .insert_resource(Volume(7))
        .insert_resource(get_bindings())
        .insert_resource(Registry::<BlockData>::new())
        .insert_resource(Registry::<ItemData>::new())
        // Declare the game state, whose starting value is determined by the `Default` trait
        .insert_resource(world::WorldMap { ..default() })
        .init_state::<GameState>()
        .enable_state_scoped_entities::<GameState>()
        // Adds the plugins for each state
        .add_plugins((
            splash_screen::splash_plugin,
            menu::menu_plugin,
            game::game_plugin,
        ))
        .add_systems(PreStartup, load_blocks_items)
        .run();
}

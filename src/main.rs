// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod window;
mod piece;

use std::collections::VecDeque;

use bevy::prelude::*;
use bevy::DefaultPlugins;
use bevy_ascii_terminal::prelude::*;
use bevy_kira_audio::{Audio, AudioControl, AudioPlugin, AudioSource};
use rand::rngs::ThreadRng;
use rand::Rng;
use window::WindowPlugin;

fn main() {
    App::new()
        .add_plugin(WindowPlugin)
        .add_plugins(DefaultPlugins)
        .add_plugin(TerminalPlugin)
        .add_plugin(AudioPlugin)
        .run();
}

#[derive(Debug, StageLabel, Clone, Eq, PartialEq, Hash)]
enum GameState {
    Begin,
    Playing,
}

pub const BOARD_SIZE: UVec2 = UVec2::from_array([10,24]);


#[derive(Component)]
struct Board {

}

fn setup(
    mut commands: Commands
) {
    let mut term = Terminal::with_size(BOARD_SIZE + 2);
    term.draw_border(BorderGlyphs::single_line());

    for point in &piece::I.points {

    }
}

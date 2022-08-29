// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod window;
mod piece;
mod board;

use bevy::prelude::*;
use bevy::DefaultPlugins;
use bevy_ascii_terminal::prelude::*;
use piece::*;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use window::WindowPlugin;

fn main() {
    App::new()
        .add_plugin(WindowPlugin)
        .add_plugins(DefaultPlugins)
        .add_plugin(TerminalPlugin)
        .init_resource::<ShuffleBag>()
        .init_resource::<Board>()
        .add_startup_system(setup)
        .add_system(try_spin)
        .add_system(render)
        .run();
}

#[derive(Debug, StageLabel, Clone, Eq, PartialEq, Hash)]
enum GameState {
    Begin,
    Playing,
}

pub const BOARD_SIZE: UVec2 = UVec2::from_array([10,24]);



#[derive(Component)]
struct Active; 

#[derive(Component)]
struct Position(Vec2);

#[derive(Default, Clone)]
pub struct ShuffleBag {
    pieces: Vec<Piece>,
}

impl ShuffleBag {
    pub fn get_piece(&mut self, rng: &mut ThreadRng) -> Piece {
        if self.pieces.is_empty() {
            self.pieces.extend(PIECES);
            self.pieces.shuffle(rng);
        }

        self.pieces.remove(self.pieces.len() - 1)
    } 
}

fn setup(
    mut commands: Commands
) {
    let term = Terminal::with_size(BOARD_SIZE + 2);

    commands.spawn_bundle(TerminalBundle::from(term))
    .insert(AutoCamera);

    for i in 0..PIECES.len() {
        let x = -18 + i as i32 * 6;
        let xy = IVec2::new(x, 0).as_vec2();
        commands.spawn().insert(PIECES[i].clone()).insert(Position(xy));
    }
}

fn render(
    mut q_term: Query<&mut Terminal>,
    q_pieces: Query<(&Piece, &Position)>,
) {
    let mut term = q_term.single_mut();
    term.clear();

    for (piece, pos) in &q_pieces {
        let pos = term.from_world(pos.0.as_ivec2());
        for tile in piece.grid_points() {
            let pos = pos + tile;
            if term.is_in_bounds(pos) {
                term.put_char(pos + tile, '*');
            }
        }
    }
}

fn try_spin(
    input: Res<Input<KeyCode>>,
    mut q_piece: Query<&mut Piece>,
) {
    if input.just_pressed(KeyCode::Space) {
        for mut piece in &mut q_piece {
            piece.rotate(Rotation::Clockwise);
        }
    }

    if input.just_pressed(KeyCode::LControl) {
        for mut piece in &mut q_piece {
            piece.rotate(Rotation::Counterclockwise);
        }
    }
}

fn gravity(
    mut q_piece: Query<(&Piece, &mut Position), With<Active>>,
) {
    for mut piece in &mut q_piece {

    }
}

fn generate_next_piece(
    mut bag: ResMut<ShuffleBag>,
) {
    
}


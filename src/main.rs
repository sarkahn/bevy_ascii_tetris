// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod window;
mod piece;
mod board;

use bevy::prelude::*;
use bevy::DefaultPlugins;
use bevy_ascii_terminal::prelude::*;
//use board::Board;
use piece::*;
use rand::Rng;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use window::WindowPlugin;

fn main() {
    App::new()
        .add_plugin(WindowPlugin)
        .add_plugins(DefaultPlugins)
        .add_plugin(TerminalPlugin)
        .init_resource::<ShuffleBag>()
        .insert_resource(Board {
            state: vec![0; BOARD_WIDTH * BOARD_HEIGHT]
        })
        .insert_resource(FallSpeed(FALL_SPEED_START))
        .add_startup_system(setup)
        .add_system(get_next)
        .add_system(movement.after(get_next))
        .add_system(render.after(movement))
        .run();
}

#[derive(Debug, StageLabel, Clone, Eq, PartialEq, Hash)]
enum GameState {
    Begin,
    Playing,
}

pub const BOARD_WIDTH: usize = 10;
pub const BOARD_HEIGHT: usize = 20;
pub const BOARD_SIZE: UVec2 = 
    UVec2::from_array([BOARD_WIDTH as u32, BOARD_HEIGHT as u32]);
/// Blocks per second
pub const FALL_SPEED_START: f32 = 1.5;
pub const FALL_SPEED_ACCEL: f32 = 0.15;
pub const FALL_SPEED_MAX: f32 = 12.5;
pub const DROP_SPEED: f32 = 10.0;


#[derive(Component)]
struct Active;

#[derive(Component)]
struct NextPiece;

#[derive(Default, Clone)]
pub struct ShuffleBag {
    pieces: Vec<Piece>,
}

#[derive(Default, Clone)]
pub struct Board {
    state: Vec<usize>,
}

#[derive(Default, Clone)]
pub struct FallSpeed(f32);

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
    let term = Terminal::with_size(BOARD_SIZE);

    commands.spawn_bundle(TerminalBundle::from(term))
    .insert(AutoCamera);

}

fn render(
    mut q_term: Query<&mut Terminal>,
    q_pieces: Query<&Piece>,
) {
    let mut term = q_term.single_mut();
    term.clear();

    for piece in &q_pieces {
        for pos in board_points(piece) {
            //println!("POS {}", pos);
            if term.is_in_bounds(pos) {
                term.put_char(pos, '*');
            }
        }
    }
}

fn movement(
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut q_piece: Query<(Entity, &mut Piece), With<Active>>,
    fall_speed: Res<FallSpeed>,
    mut board: ResMut<Board>, 
    mut commands: Commands,
) {
    let dt = time.delta_seconds();
    for (entity, mut piece) in &mut q_piece {
        let hor = input.just_pressed(KeyCode::D) as i32 - input.just_pressed(KeyCode::A) as i32;
        if hor != 0 && can_move(&board, &piece, IVec2::new(hor, 0)) {
            piece.pos.x += hor as f32;
        }

        let rot = input.just_pressed(KeyCode::E) as i32 - input.just_pressed(KeyCode::Q) as i32;
        let rot = match rot {
            1 => Some(Rotation::Clockwise),
            -1 => Some(Rotation::Counterclockwise),
            _ => None,
        };
        if let Some(rot) = rot {
            piece.rotate(rot);
            if !can_move(&board, &piece, [0,0]) {
                piece.rotate(rot.opposite());
            }
        }

        let fall = fall_speed.0 
            + DROP_SPEED * input.pressed(KeyCode::S) as i32 as f32;

        let curr = piece.pos;
        let curr_grid = curr.floor().as_ivec2();
        let next = piece.pos - Vec2::new(0., fall * dt);
        let next_grid = next.floor().as_ivec2();
        let diff = curr_grid.y - next_grid.y;

        let mut blocked = false;
        for y in 1..=diff {
            let movement = IVec2::new(0, -y);
            if !can_move(&board, &piece, movement) {
                blocked = true;
            }
        }

        if !blocked {
            piece.pos = next;
        } else {
            // Place piece on board
        }
    }
}

fn place(
    mut board: &mut Board,
    piece: &Piece
) {
    for p in board_points(&piece) {
        board.state[to_index(p)] = piece.tile_index;
    }
}

fn get_next(
    q_piece: Query<&Piece, With<Active>>,
    mut bag: ResMut<ShuffleBag>,
    mut commands: Commands,
) {
    if !q_piece.is_empty() {
        return;
    }

    let mut rng = ThreadRng::default();
    let mut piece = bag.get_piece(&mut rng);
    piece.pos.y = BOARD_HEIGHT as f32 + 2.0;
    //let piece = piece::I.clone();

    commands.spawn().insert(piece).insert(Active);
}

fn to_index(xy: impl GridPoint) -> usize {
    xy.as_index(BOARD_WIDTH)
}

fn line_is_filled(board: &Board, line: usize) -> bool {
    let i = line * BOARD_WIDTH;
    board.state[i..i+BOARD_WIDTH].iter().all(|v|*v != 0)
}

fn get_tile(board: &Board, pos: IVec2) -> Option<usize> {
    let w = BOARD_WIDTH as i32;
    let h = BOARD_HEIGHT as i32;
    if pos.x < 0 || pos.x >= w || pos.y < 0 || pos.y >= h {
        return None;
    }
    Some(board.state[to_index(pos)])
}

fn board_points(piece: &Piece) -> impl Iterator<Item=IVec2> + '_ {
    let x = BOARD_SIZE.x as i32 / 2;
    let offset = IVec2::new(x, 0);
    piece.grid_points().map(move |p| p + offset)
}

fn in_bounds(pos: IVec2) -> bool {
    let [x,y] = pos.to_array();
    x >= 0 && x < BOARD_WIDTH as i32 && y >= 0
}

fn can_move(board: &Board, piece: &Piece, movement: impl GridPoint) -> bool {
    let movement = movement.as_ivec2();

    board_points(piece).map(|p| 
        p + movement
    ).all(|p| {
        get_tile(board, p).map_or(in_bounds(p), |tile| tile == 0) 
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn move_test() {
        let board = Board {
            state: vec![0;BOARD_WIDTH * BOARD_HEIGHT],
        };
        let mut piece = piece::I;
        //piece.rotate(Rotation::Clockwise);

        board_points(&piece).for_each(|p| println!("{}", p));

        //println!("{}" , can_move(&board, &piece, [0,0]));
    }
}
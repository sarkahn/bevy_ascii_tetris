// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod window;
mod piece;
mod board;
mod shuffle_bag;
mod score;

use bevy::prelude::*;
use bevy::DefaultPlugins;
use bevy::utils::HashSet;
use bevy_ascii_terminal::prelude::*;
use board::Board;
use piece::*;
use score::Scoring;
use shuffle_bag::ShuffleBag;
use window::WindowPlugin;

fn main() {
    App::new()
        .add_plugin(WindowPlugin)
        .add_plugins(DefaultPlugins)
        .add_plugin(TerminalPlugin)
        .init_resource::<ShuffleBag>()
        .init_resource::<Scoring>()
        .insert_resource(Board {
            state: vec![0; BOARD_WIDTH * BOARD_HEIGHT]
        })
        .insert_resource(FallSpeed(FALL_SPEED_START))
        .add_startup_system(setup)
        .add_system(get_next)
        .add_system(movement.after(get_next))
        .add_system_to_stage(CoreStage::PostUpdate, place)
        .add_system_to_stage(CoreStage::Last, render)
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
pub const SOFT_DROP_SPEED: f32 = 10.0;
pub const PIECE_GLYPH: char = '█';
pub const BOARD_GLYPH: char = '█';
pub const DROP_GHOST_GLYPH: char = '■';
pub const DROP_GHOST_ALPHA: f32 = 0.09;


#[derive(Component)]
struct Active;

#[derive(Component)]
struct PlacePiece;


#[derive(Default, Clone)]
pub struct FallSpeed(f32);

pub enum DropType {
    Normal,
    Soft,
    Hard,
}

fn setup(
    mut commands: Commands
) {
    let term = Terminal::with_size(BOARD_SIZE);

    commands.spawn_bundle(TerminalBundle::from(term))
    .insert(AutoCamera);

}

fn get_next(
    q_piece: Query<&Piece, With<Active>>,
    mut bag: ResMut<ShuffleBag>,
    mut commands: Commands,
) {
    if !q_piece.is_empty() {
        return;
    }
    let mut piece = bag.get_piece();
    piece.pos.x = BOARD_WIDTH as f32 / 2.0;
    piece.pos.y = BOARD_HEIGHT as f32 + 2.0;


    commands.spawn().insert(piece).insert(Active);
}

fn movement(
    mut commands: Commands,
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    fall_speed: Res<FallSpeed>,
    board: ResMut<Board>, 
    mut q_piece: Query<(Entity, &mut Piece), With<Active>>,
    mut score: ResMut<Scoring>,
) {
    let dt = time.delta_seconds();
    for (entity, mut piece) in &mut q_piece {
        let hor = input.just_pressed(KeyCode::D) as i32 - input.just_pressed(KeyCode::A) as i32;
        if hor != 0 && can_move(&board, piece.pos, grid_points(&piece.points), IVec2::new(hor, 0)) {
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
            if !can_move(&board, piece.pos, grid_points(&piece.points), [0,0]) {
                piece.rotate(rot.opposite());
            }
        }

        let (mut fall, drop_type) = if input.pressed(KeyCode::S) {
            (fall_speed.0 + SOFT_DROP_SPEED * dt, DropType::Soft)
        } else if input.just_pressed(KeyCode::Space) {
            (30.0, DropType::Hard)
        } else {
            (fall_speed.0, DropType::Normal)
        };

        let (pos, hit, lines_moved) = try_drop(piece.pos, &piece.points, &board, fall);

        if lines_moved != 0 {
            match drop_type {
                DropType::Normal => (),
                DropType::Soft => score.soft_drop(lines_moved),
                DropType::Hard => score.hard_drop(lines_moved),
            };
        }
     
        if hit {
            commands.entity(entity).insert(PlacePiece);
        }

        piece.pos = pos;
    }
}

fn place(
    mut board: ResMut<Board>,
    q_piece: Query<(Entity, &Piece), With<PlacePiece>>,
    mut commands: Commands,
    mut score: ResMut<Scoring>,
    mut lines: Local<HashSet<usize>>,
) {
    for (entity, piece) in &q_piece {
        for p in piece.grid_points() {
            lines.insert(p.y as usize);
            let i = to_index(p);
            board.state[i] = piece.piece_id;
        }
        commands.entity(entity).despawn();
    }

    let mut count = 0;
    for line in lines.iter() {
        if board.is_line_filled(*line) {
            board.clear_line(*line);
            count += 1;
        }
    }
    lines.clear();

    if count != 0 {
        score.line_clears(count);
    }
}

fn render(
    mut q_term: Query<&mut Terminal>,
    q_pieces: Query<&Piece, With<Active>>,
    board: Res<Board>,
) {
    let mut term = q_term.single_mut();
    term.clear();

    for piece in &q_pieces {
        // Draw drop ghost
        let (drop_point,_,_) = try_drop(piece.pos, &piece.points, &board, 30.);
        for pos in grid_points(&piece.points) {
            let pos = drop_point.floor().as_ivec2() + pos;
            if term.is_in_bounds(pos) {
                let mut col = piece.color;
                col.set_a(DROP_GHOST_ALPHA);
                term.put_char(pos, DROP_GHOST_GLYPH.fg(col));
            }
        }

        for pos in piece.grid_points() {
            if term.is_in_bounds(pos) {
                term.put_char(pos, PIECE_GLYPH.fg(piece.color));
            }
        }

    }

    for (i,tile) in board.state.iter().enumerate().filter(|(_,p)|**p != 0) {
        let xy = to_xy(i);
        let color = PIECES[*tile - 1].color;
        term.put_char(xy, BOARD_GLYPH.fg(color));
    }
}

fn to_index(xy: impl GridPoint) -> usize {
    xy.as_index(BOARD_WIDTH)
}

fn to_xy(i: usize) -> IVec2 {
    let index = i as i32;
    let w = BOARD_WIDTH as i32;
    let x = index % w;
    let y = index / w;
    IVec2::new(x, y)
}

fn get_tile(board: &Board, xy: IVec2) -> Option<usize> {
    if in_bounds(xy) {
        Some(board.state[to_index(xy)])
    } else {
        None
    }
}

/// Pieces spawn above the board so the points above the board are valid
fn in_stage(xy: IVec2) -> bool {
    let [x,y] = xy.to_array();
    x >= 0 && x < BOARD_WIDTH as i32 && y >= 0
}

fn in_bounds(xy: IVec2) -> bool {
    let [x,y] = xy.to_array();
    x >= 0 && x < BOARD_WIDTH as i32 && y >= 0 && y < BOARD_HEIGHT as i32
}

fn try_drop(
    pos: Vec2,
    points: &[Vec2],
    board: &Board,
    dist: f32
) -> (Vec2, bool, usize) {
    let curr_grid = pos.floor().as_ivec2();
    let mut next = pos - Vec2::new(0., dist);
    let next_grid = next.floor().as_ivec2();
    let diff = curr_grid.y - next_grid.y;
    
    let mut hit = false;
    for y in 1..=diff {
        let movement = IVec2::new(0, -y);
        if !can_move(&board, pos, grid_points(points), movement) {
            next.y = pos.y - (y as f32 - 1.0);
            hit = true;
            break;
        }
    }
    (next, hit, diff as usize)
}

fn can_move(board: &Board,
    pos: Vec2,
    points: impl Iterator<Item=IVec2>, 
    movement: impl GridPoint
) -> bool {
    let movement = movement.as_ivec2();
    let pos = pos.floor().as_ivec2();

    points.map(|p| pos + p + movement).all(|p| {
        get_tile(board, p).map_or(in_stage(p), |tile| tile == 0) 
    }) 
}

fn grid_points(points: &[Vec2]) -> impl Iterator<Item=IVec2> + '_ {
    points.iter().map(|p|p.floor().as_ivec2())
}
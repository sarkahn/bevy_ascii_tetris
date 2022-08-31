// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod board;
mod piece;
mod score;
mod shuffle_bag;
mod window;

use std::collections::BTreeSet;

use bevy::audio::AudioSink;
use bevy::prelude::*;
use bevy::DefaultPlugins;
use bevy_ascii_terminal::prelude::*;
use bevy_ascii_terminal::TiledCameraBundle;
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
        .add_state(GameState::Begin)
        .init_resource::<ShuffleBag>()
        .init_resource::<Scoring>()
        .init_resource::<Sounds>()
        .insert_resource(FallSpeed(FALL_SPEED_START))
        .insert_resource(Board {
            state: vec![0; BOARD_WIDTH * BOARD_HEIGHT],
        })
        .add_startup_system(startup)
        .add_system_set(SystemSet::on_enter(GameState::Begin).with_system(begin_setup))
        .add_system_set(SystemSet::on_update(GameState::Begin).with_system(begin_input))
        .add_system_set(SystemSet::on_enter(GameState::Playing).with_system(play_setup))
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(get_next)
                .with_system(movement.after(get_next))
                .with_system(place.after(movement)),
        )
        .add_system_to_stage(CoreStage::Last, render)
        .add_system_to_stage(CoreStage::Last, ui_score.after(render))
        .add_system_to_stage(CoreStage::Last, ui_next.after(render))
        .add_system_set(SystemSet::on_enter(GameState::GameOver).with_system(game_over_setup))
        .add_system_set(SystemSet::on_update(GameState::GameOver).with_system(game_over_input))
        .run();
}

#[derive(Debug, StageLabel, Clone, Eq, PartialEq, Hash)]
enum GameState {
    Begin,
    Playing,
    GameOver,
}

pub const BOARD_WIDTH: usize = 10;
pub const BOARD_HEIGHT: usize = 20;
pub const BOARD_SIZE: UVec2 = UVec2::from_array([BOARD_WIDTH as u32, BOARD_HEIGHT as u32]);
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

#[derive(Component)]
struct BoardTerminal;

#[derive(Component)]
struct ScoreTerminal;

#[derive(Component)]
struct NextPieceTerminal;

#[derive(Default, Clone)]
pub struct FallSpeed(f32);

#[derive(Default, Clone)]
pub struct Sounds {
    line_1: Handle<AudioSource>,
    line_2_3: Handle<AudioSource>,
    dead: Handle<AudioSource>,
    place: Handle<AudioSource>,
    start: Handle<AudioSource>,
    tetris: Handle<AudioSource>,
    music: Handle<AudioSource>,
    music_sink: Handle<AudioSink>,
}

pub enum DropType {
    Normal,
    Soft,
    Hard,
}
fn startup(mut sfx: ResMut<Sounds>, server: Res<AssetServer>, mut commands: Commands) {
    let cam_size = [BOARD_WIDTH + 20, BOARD_HEIGHT + 2];
    commands.spawn_bundle(
        TiledCameraBundle::new()
            .with_tile_count(cam_size)
            .with_clear_color(Color::BLACK),
    );
    sfx.line_1 = server.load("1line.wav");
    sfx.line_2_3 = server.load("2_3_lines.wav");
    sfx.dead = server.load("dead.wav");
    sfx.place = server.load("place.wav");
    sfx.start = server.load("start.wav");
    sfx.tetris = server.load("tetris.wav");
    sfx.music = server.load("theme.ogg");
}

fn begin_setup(q_term: Query<Entity, With<Terminal>>, mut commands: Commands) {
    q_term.for_each(|e| commands.entity(e).despawn());

    let mut term = Terminal::with_size([30, BOARD_HEIGHT]);

    term.draw_border(BorderGlyphs::double_line());
    term.draw_box(
        [0, 5].pivot(Pivot::Center),
        [15, 3],
        UiBox::double_line().color_fill(Color::GRAY, Color::BLACK),
    );
    term.put_string([-5, 5].pivot(Pivot::Center), "ASCII TETRIS".fg(Color::RED));
    term.put_string([-5, 2].pivot(Pivot::Center), "Controls:");
    term.put_string([2, 9], "Side Movement: A/D/←/→");
    term.put_string([-8, -1].pivot(Pivot::Center), "Soft Drop: S/↓");
    term.put_string([-8, -2].pivot(Pivot::Center), "Hard Drop: Space");

    term.put_string([-9, -4].pivot(Pivot::Center), "Press Space to Begin");

    commands.spawn_bundle(TerminalBundle::from(term));
}

fn begin_input(
    input: Res<Input<KeyCode>>,
    mut state: ResMut<State<GameState>>,
    q_term: Query<Entity, With<Terminal>>,
    audio: Res<Audio>,
    mut sfx: ResMut<Sounds>,
    mut commands: Commands,
    sinks: Res<Assets<AudioSink>>,
) {
    if input.just_pressed(KeyCode::Space) {
        state.set(GameState::Playing).unwrap();
        for entity in &q_term {
            commands.entity(entity).despawn();
        }

        let sink =
            audio.play_with_settings(sfx.music.clone(), PlaybackSettings::LOOP.with_volume(0.65));
        sfx.music_sink = sinks.get_handle(sink);
    }
}

fn play_setup(
    q_term: Query<Entity, With<Terminal>>,
    mut board: ResMut<Board>,
    mut bag: ResMut<ShuffleBag>,
    mut score: ResMut<Scoring>,
    mut fall_speed: ResMut<FallSpeed>,
    mut commands: Commands,
) {
    q_term.for_each(|e| commands.entity(e).despawn());

    let term = Terminal::with_size(BOARD_SIZE + 2);

    commands
        .spawn_bundle(TerminalBundle::from(term))
        .insert(BoardTerminal);

    commands
        .spawn_bundle(
            TerminalBundle::new()
                .with_size([10, 8])
                .with_position([10, -7])
                .with_depth(2),
        )
        .insert(ScoreTerminal);

    commands
        .spawn_bundle(
            TerminalBundle::new()
                .with_size([10, 8])
                .with_position([10, 7])
                .with_depth(1),
        )
        .insert(NextPieceTerminal);

    board.reset();
    *bag = ShuffleBag::default();
    *fall_speed = FallSpeed(FALL_SPEED_START);
    *score = Scoring::default();
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
        let hor = input.just_pressed(KeyCode::D) as i32 - input.just_pressed(KeyCode::A) as i32
            + input.just_pressed(KeyCode::Right) as i32
            - input.just_pressed(KeyCode::Left) as i32;
        if hor != 0
            && can_move(
                &board,
                piece.pos,
                grid_points(&piece.points),
                IVec2::new(hor, 0),
            )
        {
            piece.pos.x += hor as f32;
        }

        let rot = input.just_pressed(KeyCode::E) as i32 - input.just_pressed(KeyCode::Q) as i32
            + input.just_pressed(KeyCode::X) as i32
            - input.just_pressed(KeyCode::Z) as i32;
        let rot = match rot {
            1 => Some(Rotation::Clockwise),
            -1 => Some(Rotation::Counterclockwise),
            _ => None,
        };
        if let Some(rot) = rot {
            piece.rotate(rot);
            if !can_move(&board, piece.pos, grid_points(&piece.points), [0, 0]) {
                piece.rotate(rot.opposite());
            }
        }

        let mut fall = fall_speed.0 + FALL_SPEED_ACCEL * score.level() as f32;
        let drop_type = if input.pressed(KeyCode::S) || input.pressed(KeyCode::Down) {
            fall = (fall + SOFT_DROP_SPEED) * dt;
            DropType::Soft
        } else if input.just_pressed(KeyCode::Space) {
            fall = 30.0;
            DropType::Hard
        } else {
            fall *= dt;
            DropType::Normal
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

#[allow(clippy::too_many_arguments)]
fn place(
    mut board: ResMut<Board>,
    q_piece: Query<(Entity, &Piece), With<PlacePiece>>,
    mut commands: Commands,
    mut score: ResMut<Scoring>,
    mut lines: Local<BTreeSet<usize>>,
    mut state: ResMut<State<GameState>>,
    audio: Res<Audio>,
    sfx: Res<Sounds>,
) {
    for (entity, piece) in &q_piece {
        for p in piece.grid_points() {
            if p.y >= BOARD_HEIGHT as i32 {
                state.set(GameState::GameOver).unwrap();
                audio.play(sfx.dead.clone());
                return;
            }
            lines.insert(p.y as usize);
            let i = to_index(p);
            board.state[i] = piece.piece_id;
        }
        // audio.play(sfx.place.clone());
        commands.entity(entity).despawn();
    }

    let mut count = 0;
    // Lines must be cleared in reverse order
    for line in lines.iter().rev() {
        if board.is_line_filled(*line) {
            board.clear_line(*line);
            count += 1;
        }
    }
    lines.clear();

    if count != 0 {
        score.line_clears(count);
        let sound = match count {
            1 => &sfx.line_1,
            4 => &sfx.tetris,
            _ => &sfx.line_2_3,
        };
        audio.play(sound.clone());
    }
}

fn render(
    mut q_term: Query<&mut Terminal, With<BoardTerminal>>,
    q_pieces: Query<&Piece, With<Active>>,
    board: Res<Board>,
) {
    if q_term.is_empty() {
        return;
    }

    let mut term = q_term.single_mut();
    term.clear();

    for piece in &q_pieces {
        // Draw drop ghost
        let (drop_point, _, _) = try_drop(piece.pos, &piece.points, &board, 30.);
        for pos in grid_points(&piece.points) {
            // Add one to all positions to account for terminal border
            let pos = drop_point.floor().as_ivec2() + pos + 1;
            if term.is_in_bounds(pos) {
                let mut col = piece.color;
                col.set_a(DROP_GHOST_ALPHA);
                term.put_char(pos, DROP_GHOST_GLYPH.fg(col));
            }
        }

        for pos in piece.grid_points() {
            let pos = pos + 1;
            if term.is_in_bounds(pos) {
                term.put_char(pos, PIECE_GLYPH.fg(piece.color));
            }
        }
    }

    for (i, tile) in board.state.iter().enumerate().filter(|(_, p)| **p != 0) {
        let color = PIECES[*tile - 1].color;
        let xy = to_xy(i) + 1;
        term.put_char(xy, BOARD_GLYPH.fg(color));
    }
    term.draw_border(BorderGlyphs::double_line().fg(Color::WHITE));
}

fn ui_score(mut q_term: Query<&mut Terminal, With<ScoreTerminal>>, score: Res<Scoring>) {
    if q_term.is_empty() {
        return;
    }

    if score.is_changed() {
        let mut term = q_term.single_mut();

        term.clear();
        let glyphs = BorderGlyphs::from_string(
            "╠═╗
          ║ ║
          ╩═╝",
        );
        term.draw_border(glyphs);
        term.put_string([1, 6], "Score:");
        term.put_string([2, 5], score.score().to_string());
        term.put_string([1, 4], "Level:");
        term.put_string([2, 3], score.level().to_string());
        term.put_string([1, 2], "Lines:");
        term.put_string([2, 1], score.lines().to_string());
    }
}

fn ui_next(bag: Res<ShuffleBag>, mut q_term: Query<&mut Terminal, With<NextPieceTerminal>>) {
    if q_term.is_empty() {
        return;
    }

    if bag.is_changed() {
        let mut term = q_term.single_mut();
        term.clear();
        let glyphs = BorderGlyphs::from_string(
            "╦═╗
          ║ ║
          ╠═╝",
        );
        term.draw_border(glyphs);
        term.put_string([2, 1].pivot(Pivot::TopLeft), "Next:");
        let piece = bag.peek();
        for p in piece.grid_points() {
            term.put_char(p.pivot(Pivot::Center), PIECE_GLYPH.fg(piece.color));
        }
    }
}

fn game_over_setup(
    q_term: Query<Entity, With<Terminal>>,
    q_pieces: Query<Entity, With<Piece>>,
    score: Res<Scoring>,
    mut commands: Commands,
    sfx: Res<Sounds>,
    sinks: Res<Assets<AudioSink>>,
) {
    for entity in &q_term {
        commands.entity(entity).despawn();
    }

    for entity in &q_pieces {
        commands.entity(entity).despawn();
    }

    let final_score = score.score().to_string();

    let mut term = Terminal::with_size([30, 8]);
    term.draw_border(BorderGlyphs::double_line());

    term.put_string([-5, 3].pivot(Pivot::Center), "Game Over!");
    term.put_string([-6, 1].pivot(Pivot::Center), "Final Score:");
    let x = final_score.chars().count() as i32 / 2;
    term.put_string([-x, 0].pivot(Pivot::Center), final_score.fg(Color::YELLOW));

    term.put_string([-10, -2].pivot(Pivot::Center), "Press Space to restart");

    commands.spawn_bundle(TerminalBundle::from(term));

    if let Some(music) = sinks.get(&sfx.music_sink) {
        music.stop();
    }
    //audio.play_with_settings(sfx.music.clone(), PlaybackSettings::ONCE.with_volume(0.0));
}

fn game_over_input(input: Res<Input<KeyCode>>, mut state: ResMut<State<GameState>>) {
    if input.just_pressed(KeyCode::Space) {
        state.set(GameState::Begin).unwrap();
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

/// Pieces spawn above the board so the points above the board are valid for
/// movement, but not rendering
fn in_stage(xy: IVec2) -> bool {
    let [x, y] = xy.to_array();
    x >= 0 && x < BOARD_WIDTH as i32 && y >= 0
}

fn in_bounds(xy: IVec2) -> bool {
    let [x, y] = xy.to_array();
    x >= 0 && x < BOARD_WIDTH as i32 && y >= 0 && y < BOARD_HEIGHT as i32
}

/// Try to move a block down by the given amount.
///
/// Returns (position after move, hit, and number of lines moved)
fn try_drop(pos: Vec2, points: &[Vec2], board: &Board, dist: f32) -> (Vec2, bool, usize) {
    let curr_grid = pos.floor().as_ivec2();
    let mut next = pos - Vec2::new(0., dist);
    let next_grid = next.floor().as_ivec2();
    let diff = curr_grid.y - next_grid.y;

    let mut hit = false;
    for y in 1..=diff {
        let movement = IVec2::new(0, -y);
        if !can_move(board, pos, grid_points(points), movement) {
            next.y = pos.y - (y as f32 - 1.0);
            hit = true;
            break;
        }
    }
    (next, hit, diff as usize)
}

fn can_move(
    board: &Board,
    pos: Vec2,
    points: impl Iterator<Item = IVec2>,
    movement: impl GridPoint,
) -> bool {
    let movement = movement.as_ivec2();
    let pos = pos.floor().as_ivec2();

    points
        .map(|p| pos + p + movement)
        .all(|p| get_tile(board, p).map_or(in_stage(p), |tile| tile == 0))
}

fn grid_points(points: &[Vec2]) -> impl Iterator<Item = IVec2> + '_ {
    points.iter().map(|p| p.floor().as_ivec2())
}

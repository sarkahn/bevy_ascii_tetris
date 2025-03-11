mod board;
mod piece;
mod score;
mod shuffle_bag;
use std::collections::BTreeSet;

use bevy::audio::AudioSink;
use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy::{DefaultPlugins, audio::Volume};
use bevy_ascii_terminal::*;
use board::{Board, EMPTY_SQUARE};
use piece::*;
use score::Scoring;
use shuffle_bag::ShuffleBag;

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
pub const DROP_GHOST_GLYPH: char = '□';
pub const DROP_GHOST_ALPHA: f32 = 0.09;
pub const MUSIC_VOLUME: f32 = 0.2;
pub const SOUND_VOLUME: f32 = 0.5;

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

#[derive(Event)]
struct NextPiece;

#[derive(Default, Clone, Resource, Deref, DerefMut)]
pub struct FallSpeed(f32);

#[derive(Resource, Deref, DerefMut)]
pub struct RepeatTimer(Timer);

pub enum DropType {
    Normal,
    Soft,
    Hard,
}

#[derive(Debug, States, PartialEq, Eq, Hash, Clone)]
enum GameState {
    Setup,
    Title,
    Playing,
    GameOver,
}

#[derive(Component)]
struct Music;

#[derive(Resource)]
pub struct Settings {
    music_volume: f32,
    sound_volume: f32,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    prevent_default_event_handling: false,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            TerminalPlugins,
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .init_resource::<ShuffleBag>()
        .init_resource::<Scoring>()
        .insert_resource(FallSpeed(FALL_SPEED_START))
        .insert_resource(Board {
            state: vec![0; BOARD_WIDTH * BOARD_HEIGHT],
        })
        .insert_resource(Settings {
            music_volume: 0.0,
            sound_volume: 0.0,
        })
        .add_event::<NextPiece>()
        .add_systems(Startup, setup)
        .add_systems(OnEnter(GameState::Title), restart_to_title.after(setup))
        .add_systems(Update, title_input.run_if(in_state(GameState::Title)))
        .add_systems(OnEnter(GameState::GameOver), game_over)
        .add_systems(OnEnter(GameState::Playing), next_piece)
        .add_systems(
            Update,
            (
                options_input,
                game_over_input.run_if(in_state(GameState::GameOver)),
            ),
        )
        .add_systems(
            PreUpdate,
            next_piece
                .run_if(on_event::<NextPiece>)
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (movement, place, draw_board, draw_score, draw_next)
                .chain()
                .run_if(in_state(GameState::Playing)),
        )
        .insert_state(GameState::Setup)
        .run();
}

#[rustfmt::skip]
fn setup(mut commands: Commands) {
    commands.spawn((
        Terminal::new(BOARD_SIZE),
        BoardTerminal,
        TerminalMeshPivot::BottomLeft,
        SetTerminalLayerPosition(1),
        TerminalBorder::single_line(),
    ));

    commands.spawn((
        Terminal::new([7, 6]),
        ScoreTerminal,
        TerminalMeshPivot::BottomLeft,
        TerminalBorder::single_line(),
        SetTerminalGridPosition(IVec2::new(BOARD_WIDTH as i32 + 2, 0)),
    ));

    commands.spawn((
        Terminal::new([7, 6]),
        SetTerminalGridPosition(IVec2::new(BOARD_WIDTH as i32 + 2, BOARD_HEIGHT as i32 + 2)),
        NextPieceTerminal,
        TerminalMeshPivot::TopLeft,
        TerminalBorder::single_line(),
    ));

    commands.spawn(TerminalCamera::new());

    commands.set_state(GameState::Title);
}

// on event: Restart
#[allow(clippy::too_many_arguments)]
fn restart_to_title(
    q_pieces: Query<Entity, With<Piece>>,
    mut board: ResMut<Board>,
    mut bag: ResMut<ShuffleBag>,
    mut score: ResMut<Scoring>,
    mut fall_speed: ResMut<FallSpeed>,
    mut commands: Commands,
    mut q_board_term: Query<&mut Terminal, With<BoardTerminal>>,
) {
    for entity in &q_pieces {
        commands.entity(entity).despawn();
    }

    board.reset();
    *bag = ShuffleBag::default();
    *fall_speed = FallSpeed(FALL_SPEED_START);
    *score = Scoring::default();

    let mut term = q_board_term.single_mut();
    term.clear();
    term.resize([BOARD_WIDTH + 20, BOARD_HEIGHT]);
    term.put_string([0, 6].pivot(Pivot::Center), "ASCII TETRIS".fg(color::RED));
    term.put_string([0, 4].pivot(Pivot::Center), "Controls:");
    term.put_string(
        [0, 0].pivot(Pivot::Center),
        "Movement: A/D/←/→
Soft Drop: S/↓
Hard Drop: Space

Toggle Music: M
Toggle Sound: N

Press Space to Begin",
    );
}

fn title_input(
    mut q_board_term: Query<&mut Terminal, With<BoardTerminal>>,
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    server: Res<AssetServer>,
    settings: Res<Settings>,
) {
    if input.just_pressed(KeyCode::Space) {
        commands.set_state(GameState::Playing);
        let mut term = q_board_term.single_mut();
        term.clear();
        term.resize([BOARD_WIDTH, BOARD_HEIGHT]);
        commands.spawn((
            AudioPlayer::new(server.load("start.wav")),
            PlaybackSettings::ONCE.with_volume(Volume::new(settings.sound_volume)),
        ));
        commands.spawn((
            AudioPlayer::new(server.load("theme.ogg")),
            PlaybackSettings::LOOP.with_volume(Volume::new(settings.music_volume)),
            Music,
        ));
    }
}

fn options_input(
    input: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<Settings>,
    q_music: Query<&AudioSink, With<Music>>,
) {
    if input.just_pressed(KeyCode::KeyM) {
        settings.music_volume = MUSIC_VOLUME - settings.music_volume;
        q_music.iter().for_each(|player| {
            player.set_volume(settings.music_volume);
        });
    }

    if input.just_pressed(KeyCode::KeyN) {
        settings.sound_volume = SOUND_VOLUME - settings.sound_volume;
    }
}

fn game_over_input(input: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if input.just_pressed(KeyCode::Space) {
        commands.set_state(GameState::Title);
    }
}

fn next_piece(
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

    commands.spawn((piece, Active));
}

#[allow(clippy::too_many_arguments)]
fn movement(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    fall_speed: Res<FallSpeed>,
    board: ResMut<Board>,
    mut q_piece: Query<(Entity, &mut Piece), With<Active>>,
    mut score: ResMut<Scoring>,
    time: Res<Time>,
    mut key_events: EventReader<KeyboardInput>,
) {
    let dt = time.delta_secs();
    for (entity, mut piece) in &mut q_piece {
        // Manual input polling handles key repeat automatically, feels much more
        // responsive for movement
        for evt in key_events.read() {
            if evt.state == ButtonState::Pressed {
                let right =
                    (evt.key_code == KeyCode::KeyD || evt.key_code == KeyCode::ArrowRight) as i32;
                let left =
                    (evt.key_code == KeyCode::KeyA || evt.key_code == KeyCode::ArrowLeft) as i32;
                let hor = right - left;
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
            }
        }

        let rot = input.any_just_pressed([KeyCode::KeyE, KeyCode::KeyX]) as i32
            - input.any_just_pressed([KeyCode::KeyQ, KeyCode::KeyZ]) as i32;
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
        let drop_type =
            if input.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]) && piece_is_visible(&piece) {
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
    mut lines_to_clear: Local<BTreeSet<usize>>,
    server: Res<AssetServer>,
    settings: Res<Settings>,
) {
    for (entity, piece) in &q_piece {
        for p in piece.grid_points() {
            if p.y >= BOARD_HEIGHT as i32 {
                commands.set_state(GameState::GameOver);
                return;
            }
            lines_to_clear.insert(p.y as usize);
            let i = p.as_index(BOARD_SIZE);
            board.state[i] = piece.piece_id;
        }
        // audio.play(sfx.place.clone());
        commands.entity(entity).despawn();
        commands.send_event(NextPiece);
        commands.spawn((
            AudioPlayer::new(server.load("place.wav")),
            PlaybackSettings::ONCE.with_volume(Volume::new(settings.sound_volume)),
        ));
    }

    let mut count = 0;
    // Lines must be cleared in reverse order
    for line in lines_to_clear.iter().rev() {
        if board.is_line_filled(*line) {
            board.clear_line(*line);
            count += 1;
        }
    }
    lines_to_clear.clear();

    if count != 0 {
        score.line_clears(count);
        let sound: Handle<AudioSource> = match count {
            1 => server.load("1line.wav"),
            4 => server.load("tetris.wav"),
            _ => server.load("2_3_lines.wav"),
        };
        commands.spawn((
            AudioPlayer::new(sound),
            PlaybackSettings::ONCE.with_volume(Volume::new(settings.sound_volume)),
        ));
    }
}

fn draw_board(
    mut q_term: Query<&mut Terminal, With<BoardTerminal>>,
    q_pieces: Query<&Piece, With<Active>>,
    board: Res<Board>,
) {
    if q_term.is_empty() || q_pieces.is_empty() {
        return;
    }

    let mut term = q_term.single_mut();
    term.clear();

    for piece in &q_pieces {
        // Draw drop ghost
        let (drop_point, _, _) = try_drop(piece.pos, &piece.points, &board, 30.);
        for pos in grid_points(&piece.points) {
            let pos = drop_point.floor().as_ivec2() + pos;
            if term.bounds().contains_point(pos) {
                let mut col = piece.color;
                col.set_alpha(DROP_GHOST_ALPHA);
                term.put_char(pos, DROP_GHOST_GLYPH).fg(col);
            }
        }

        // Draw actual piece
        for pos in piece.grid_points() {
            if term.bounds().contains_point(pos) {
                term.put_char(pos, PIECE_GLYPH).fg(piece.color);
            }
        }
    }

    for (i, tile_index) in board
        .state
        .iter()
        .enumerate()
        .filter(|(_, p)| **p < EMPTY_SQUARE)
    {
        let color = PIECES[*tile_index].color;
        let xy = term.index_to_tile(i);
        term.put_char(xy, BOARD_GLYPH).fg(color);
    }
}

fn draw_score(mut q_term: Query<&mut Terminal, With<ScoreTerminal>>, score: Res<Scoring>) {
    if q_term.is_empty() {
        return;
    }

    if score.is_changed() {
        let mut term = q_term.single_mut();

        term.clear();
        term.put_string([1, 0], "Score:");
        term.put_string([2, 1], score.score().to_string());
        term.put_string([1, 2], "Level:");
        term.put_string([2, 3], score.level().to_string());
        term.put_string([1, 4], "Lines:");
        term.put_string([2, 5], score.lines().to_string());
    }
}

fn draw_next(bag: Res<ShuffleBag>, mut q_term: Query<&mut Terminal, With<NextPieceTerminal>>) {
    if q_term.is_empty() {
        return;
    }

    if bag.is_changed() {
        let mut term = q_term.single_mut();
        term.clear();

        term.put_string([1, 0].pivot(Pivot::TopLeft), "Next:");
        let piece = bag.peek();
        for p in piece.grid_points() {
            let p = IVec2::new(3, 2) + p;
            term.put_char(p, PIECE_GLYPH).fg(piece.color);
        }
    }
}

fn game_over(
    mut q_board_term: Query<&mut Terminal, With<BoardTerminal>>,
    q_pieces: Query<Entity, With<Piece>>,
    score: Res<Scoring>,
    mut commands: Commands,
    server: Res<AssetServer>,
    q_music: Query<Entity, With<Music>>,
    settings: Res<Settings>,
) {
    for entity in &q_pieces {
        commands.entity(entity).despawn();
    }
    let mut term = q_board_term.single_mut();

    term.clear();
    term.resize([BOARD_WIDTH + 20, BOARD_HEIGHT]);

    term.put_string([0, 3].pivot(Pivot::Center), "Game Over!".fg(color::RED));
    term.put_string([0, 2].pivot(Pivot::Center), "Final Score: ");
    term.put_string(
        [0, 0].pivot(Pivot::Center),
        score.score().to_string().fg(color::YELLOW),
    );
    term.put_string([0, -2].pivot(Pivot::Center), "Press Space to restart");

    for entity in &q_music {
        commands.entity(entity).despawn();
    }
    commands.spawn((
        AudioPlayer::new(server.load("dead.wav")),
        PlaybackSettings::ONCE.with_volume(Volume::new(settings.sound_volume)),
    ));
}

fn get_tile(board: &Board, xy: IVec2) -> Option<usize> {
    if in_bounds(xy) {
        Some(board.state[xy.as_index(BOARD_SIZE)])
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

fn piece_is_visible(piece: &Piece) -> bool {
    piece
        .points
        .iter()
        .any(|p| in_stage(piece.pos.as_ivec2() + p.as_ivec2()))
}

/// Try to move a block down by the given amount.
///
/// Returns (position after move, whether or not we hit something, and number of lines moved)
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
    let movement = movement.to_ivec2();
    let pos = pos.floor().as_ivec2();

    points
        .map(|p| pos + p + movement)
        .all(|p| get_tile(board, p).map_or(in_stage(p), |tile| tile == EMPTY_SQUARE))
}

fn grid_points(points: &[Vec2]) -> impl Iterator<Item = IVec2> + '_ {
    points.iter().map(|p| p.floor().as_ivec2())
}

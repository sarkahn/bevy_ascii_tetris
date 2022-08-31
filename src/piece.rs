use bevy::prelude::{Component, Vec2, IVec2, Mat2, Color, default};
use bevy_ascii_terminal::GridPoint;

const ROT_CLOCKWISE: Mat2 = Mat2::from_cols_array(&[0.,-1.,1.,0.]);

#[derive(Debug, Component, Clone, Default)]
pub struct Piece {
    pub points: [Vec2;4],
    pub color: Color,
    pub piece_id: usize,
    pub pos: Vec2,
}

#[derive(Debug, Clone, Copy)]
pub enum Rotation {
    Clockwise,
    Counterclockwise,
}

impl Rotation {
    pub fn opposite(&self) -> Rotation {
        match self {
            Rotation::Clockwise => Rotation::Counterclockwise,
            Rotation::Counterclockwise => Rotation::Clockwise,
        }
    }
}

impl Piece {
    pub fn grid_points(&self) -> impl Iterator<Item=IVec2> + '_ {
        let pos = self.pos.floor().as_ivec2();
        self.points.iter().map(move |p| pos + p.floor().as_ivec2())
    }

    pub fn rotate(&mut self, direction: Rotation) {
        let rot = match direction {
            Rotation::Clockwise => ROT_CLOCKWISE,
            Rotation::Counterclockwise => -ROT_CLOCKWISE,
        };
        for p in self.points.iter_mut() {
            *p = rot.mul_vec2(*p);
        }
    }
}

pub const I: Piece = Piece {
    points: [
        Vec2::from_array([-1.5, 0.5]),
        Vec2::from_array([-0.5, 0.5]),
        Vec2::from_array([0.5, 0.5]),
        Vec2::from_array([1.5, 0.5]),
    ],
    color: Color::rgb(0.,1.,1.),
    piece_id: 1,
    pos: Vec2::ZERO,
};

pub const J: Piece = Piece {
    points: [
        Vec2::from_array([-1., 1.]),
        Vec2::from_array([-1., 0.]),
        Vec2::from_array([0., 0.]),
        Vec2::from_array([1., 0.]),
    ],
    color: Color::rgb(0.,0.,1.),
    piece_id: 2,
    pos: Vec2::ZERO,
};

pub const L: Piece = Piece {
    points: [
        Vec2::from_array([-1., 0.]),
        Vec2::from_array([0., 0.]),
        Vec2::from_array([1., 0.]),
        Vec2::from_array([1., 1.]),
    ],
    color: Color::rgb(1., 0.66, 0.),
    piece_id: 3,
    pos: Vec2::ZERO,
};

pub const O: Piece = Piece {
    points: [
        Vec2::from_array([-0.5, 0.5]),
        Vec2::from_array([-0.5, -0.5]),
        Vec2::from_array([0.5, -0.5]),
        Vec2::from_array([0.5, 0.5]),
    ],
    color: Color::rgb(1.,1.,0.),
    piece_id: 4,
    pos: Vec2::ZERO,
};

pub const S: Piece = Piece {
    points: [
        Vec2::from_array([-1., 0.]),
        Vec2::from_array([0., -0.]),
        Vec2::from_array([0., 1.]),
        Vec2::from_array([1., 1.]),
    ],
    color: Color::rgb(0.,1.,0.),
    piece_id: 5,
    pos: Vec2::ZERO,
};

pub const T: Piece = Piece {
    points: [
        Vec2::from_array([-1., 0.]),
        Vec2::from_array([0., 0.]),
        Vec2::from_array([0., 1.]),
        Vec2::from_array([1., 0.]),
    ],
    color: Color::rgb(0.6,0.,1.),
    piece_id: 6,
    pos: Vec2::ZERO,
};

pub const Z: Piece = Piece {
    points: [
        Vec2::from_array([-1., 1.]),
        Vec2::from_array([0., 1.]),
        Vec2::from_array([0., 0.]),
        Vec2::from_array([1., 0.]),
    ],
    color: Color::rgb(1.,0.,0.),
    piece_id: 7,
    pos: Vec2::ZERO,
};

pub const PIECES: [Piece;7] = [
    I, J, L, O, S, T, Z
];
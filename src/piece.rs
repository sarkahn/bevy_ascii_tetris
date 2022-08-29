use bevy::prelude::{Component, Vec2};


#[derive(Component)]
pub struct Piece {
    pub points: [Vec2;4],
}

pub const I: Piece = Piece {
    points: [
        Vec2::from_array([-1.5, 0.5]),
        Vec2::from_array([-1.5, 0.5]),
        Vec2::from_array([-1.5, 0.5]),
        Vec2::from_array([-1.5, 0.5]),
    ],
};

pub const J: Piece = Piece {
    points: [
        Vec2::from_array([-1., 1.]),
        Vec2::from_array([-1., 0.]),
        Vec2::from_array([0., 0.]),
        Vec2::from_array([1., 0.]),
    ],
};

pub const L: Piece = Piece {
    points: [
        Vec2::from_array([-1., 0.]),
        Vec2::from_array([0., 0.]),
        Vec2::from_array([1., 0.]),
        Vec2::from_array([1., 1.]),
    ],
};

pub const O: Piece = Piece {
    points: [
        Vec2::from_array([-0.5, 0.5]),
        Vec2::from_array([-0.5, -0.5]),
        Vec2::from_array([0.5, -0.5]),
        Vec2::from_array([0.5, 0.5]),
    ],
};

pub const S: Piece = Piece {
    points: [
        Vec2::from_array([-1., 0.]),
        Vec2::from_array([0., -0.]),
        Vec2::from_array([0., 1.]),
        Vec2::from_array([1., 1.]),
    ],
};

pub const T: Piece = Piece {
    points: [
        Vec2::from_array([-1., 0.]),
        Vec2::from_array([0., 0.]),
        Vec2::from_array([0., 1.]),
        Vec2::from_array([1., 0.]),
    ],
};

pub const Z: Piece = Piece {
    points: [
        Vec2::from_array([-1., 1.]),
        Vec2::from_array([0., 1.]),
        Vec2::from_array([0., 0.]),
        Vec2::from_array([1., 0.]),
    ],
};

pub const PIECES: [Piece;7] = [
    I, J, L, O, S, T, Z
];
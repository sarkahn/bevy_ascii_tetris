use bevy::prelude::{Component, Vec2, IVec2, Mat2};

const ROT_CLOCKWISE: Mat2 = Mat2::from_cols_array(&[0.,-1.,1.,0.]);

#[derive(Component, Clone)]
pub struct Piece {
    pub points: [Vec2;4],
}

#[derive(Debug, Clone, Copy)]
pub enum Rotation {
    Clockwise,
    Counterclockwise,
}

impl Piece {
    pub fn grid_points(&self) -> impl Iterator<Item=IVec2> + '_ {
        self.points.iter().map(move |p|p.floor().as_ivec2())
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
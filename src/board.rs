use bevy::prelude::{UVec2, IVec2};
use bevy_ascii_terminal::GridPoint;


#[derive(Clone, Default)]
pub struct Board {
    pub state: Vec<usize>,
    size: UVec2,
}

impl Board {
    pub fn width(&self) -> usize {
        self.size.x as usize
    }

    pub fn height(&self) -> usize {
        self.size.y as usize
    }

    pub fn check_line_clear(&self, line: usize) -> bool {
        self.state[line * self.width()..self.width()].iter().all(|v| *v != 0)
    }

    pub fn clear_line(&mut self, line: usize) {
        let w = self.width();
        let i = line * w;
        self.state.drain(i..i + w).count();
        for _ in 0..w {
            self.state.push(0);
        }
    }

    pub fn to_index(&self, xy: impl GridPoint) -> usize {
        xy.as_index(self.width())
    }

    pub fn in_bounds(&self, xy: impl GridPoint) -> bool {
        let xy = xy.as_ivec2();
        !(xy.cmplt(IVec2::ZERO).any() || xy.cmpge(self.size.as_ivec2()).any())
    }

    pub fn has_tile(&self, xy: impl GridPoint) -> bool {
        if !self.in_bounds(xy) {
            return false;
        }
        let i = xy.as_index(self.width());
        self.state[i] >= 0
    }

    pub fn tile_value(&self, xy: impl GridPoint) -> usize {
        let i = self.to_index(xy);
        self.state[i]
    }

    pub fn set_tile(&mut self, xy: impl GridPoint, value: usize) {
        let i = self.to_index(xy);
        self.state[i] = value;
    }
} 
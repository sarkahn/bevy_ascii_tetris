use bevy::prelude::{UVec2, IVec2};
use bevy_ascii_terminal::GridPoint;

use crate::BOARD_WIDTH;

#[derive(Default, Clone)]
pub struct Board {
    pub state: Vec<usize>,
}

impl Board {
    pub fn is_line_filled(&self, line: usize) -> bool {
        let i = line * BOARD_WIDTH;
        self.state[i..i + BOARD_WIDTH].iter().all(|v| *v != 0)
    }

    pub fn clear_line(&mut self, line: usize) {
        let i = line * BOARD_WIDTH;
        self.state.drain(i..i + BOARD_WIDTH).count();
        self.state.extend([0;BOARD_WIDTH].iter());
    }
}
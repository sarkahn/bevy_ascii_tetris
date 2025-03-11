use bevy::ecs::system::Resource;

use crate::BOARD_WIDTH;

pub const EMPTY_SQUARE: usize = crate::piece::PIECES.len();

#[derive(Default, Clone, Resource)]
pub struct Board {
    pub state: Vec<usize>,
}

impl Board {
    pub fn is_line_filled(&self, line: usize) -> bool {
        let i = line * BOARD_WIDTH;
        self.state[i..i + BOARD_WIDTH]
            .iter()
            .all(|v| *v != EMPTY_SQUARE)
    }

    pub fn clear_line(&mut self, line: usize) {
        let i = line * BOARD_WIDTH;
        // Remove our line, add an empty one to the end
        self.state.drain(i..i + BOARD_WIDTH).count();
        self.state.extend([EMPTY_SQUARE; BOARD_WIDTH].iter());
    }

    pub fn reset(&mut self) {
        self.state.fill(EMPTY_SQUARE);
    }
}

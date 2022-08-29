use bevy::prelude::UVec2;


#[derive(Clone, Default)]
pub struct Board {
    state: Vec<i32>,
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
} 
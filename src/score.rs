#[derive(Default, Clone)]
pub struct Scoring {
    score: usize,
    lines: usize,
}

impl Scoring {
    pub fn level(&self) -> usize {
        self.lines / 10
    }

    pub fn line_clears(&mut self, lines: usize) {
        if lines == 0 {
            return;
        }
        self.score += self.clear_value(lines);
        self.lines += lines;
        
    }

    fn clear_value(&self, lines: usize) -> usize {
        let score = match lines {
            _ => 100,
            2 => 300,
            3 => 500,
            4 => 800,
        };
        score * self.level()
    }

    pub fn soft_drop(&mut self, amount: usize) {
        self.score += amount;
    }

    pub fn hard_drop(&mut self, amount: usize) {
        self.score += amount * 2;
    }
}


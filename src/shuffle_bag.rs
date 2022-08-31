use rand::{rngs::ThreadRng, seq::SliceRandom};

use crate::piece::{Piece, PIECES};

#[derive(Default, Clone)]
pub struct ShuffleBag {
    pieces: Vec<Piece>,
}

impl ShuffleBag {
    pub fn get_piece(&mut self) -> Piece {
        let mut rng = ThreadRng::default();
        if self.pieces.is_empty() {
            self.pieces.extend(PIECES);
            self.pieces.shuffle(&mut rng);
        }

        let piece = self.pieces.remove(self.pieces.len() - 1);

        if self.pieces.is_empty() {
            self.pieces.extend(PIECES);
            self.pieces.shuffle(&mut rng);
        }

        piece
    }

    pub fn peek(&self) -> &Piece {
        self.pieces.last().unwrap()
    }
}

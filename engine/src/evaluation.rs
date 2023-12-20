use strum::IntoEnumIterator;

use crate::position::{Piece,Position, Side};


fn piece_value(piece: Piece) -> f64 {
    // Piece values from AlphaZero
    match piece {
        Piece::Pawn => 1.0,
        Piece::Knight => 3.05,
        Piece::Bishop => 3.33,
        Piece::Rook => 5.63,
        Piece::Queen => 9.5,
        Piece::King => 100.0,
    }
}

pub fn evaluate(position: &Position) -> f64 {
    let mut eval = 0.0;

    for piece in Piece::iter() {
        let num_white_pieces = f64::from(position.pieces.get(piece).get(Side::White).num_squares_set());
        let num_black_pieces = f64::from(position.pieces.get(piece).get(Side::Black).num_squares_set());
        eval += num_white_pieces * piece_value(piece) - num_black_pieces * piece_value(piece);
    }

    eval
}

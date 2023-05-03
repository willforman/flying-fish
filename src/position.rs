use std::fmt;
use std::ops::Index;

use strum::IntoEnumIterator;

use crate::bitboard::{BitBoard,Square};
use crate::bitboard::Square::*;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Side {
    White,
    Black
}

pub(crate) enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King
}

pub const PIECE_CHARS: [char; 6] = [
    'p', 'n', 'b', 'r', 'q', 'k'
];

pub(crate) struct Sides {
    white: BitBoard,
    black: BitBoard
}

impl Sides {
    fn start() -> Self {
        Self {
            white: BitBoard::from_squares(&[
                A1, B1, C1, D1, E1, F1, G1, H1,
                A2, B2, C2, D2, E2, F2, G2, H2,
            ]),
            black: BitBoard::from_squares(&[
                A7, B7, C7, D7, E7, F7, G7, H7,
                A8, B8, C8, D8, E8, F8, G8, H8,
            ])
        }
    }
}

pub(crate) struct Pieces {
    pawns: Sides,
    knights: Sides,
    bishops: Sides,
    rooks: Sides,
    queens: Sides,
    kings: Sides,
}

impl Pieces {
    fn start() -> Self {
        Self {
            pawns: Sides {
                white: { BitBoard::from_squares(&[A2, B2, C2, D2, E2, F2, G2, H2]) },
                black: { BitBoard::from_squares(&[A7, B7, C7, D7, E7, F7, G7, H7]) }
            },
            knights: Sides {
                white: { BitBoard::from_squares(&[B1, G1]) },
                black: { BitBoard::from_squares(&[B8, G8]) }
            },
            bishops: Sides {
                white: { BitBoard::from_squares(&[C1, F1]) },
                black: { BitBoard::from_squares(&[C8, F8]) }
            },
            rooks: Sides {
                white: { BitBoard::from_squares(&[A1, H1]) },
                black: { BitBoard::from_squares(&[A8, H8]) }
            },
            queens: Sides {
                white: { BitBoard::from_squares(&[D1]) },
                black: { BitBoard::from_squares(&[D8]) }
            },
            kings: Sides {
                white: { BitBoard::from_squares(&[E1]) },
                black: { BitBoard::from_squares(&[E8]) }
            },
        }
    }
}

impl Index<&Piece> for Pieces {
    type Output = Sides;

    fn index(&self, index: &Piece) -> &Self::Output {
        match index {
            Piece::Pawn => &self.pawns,
            Piece::Knight => &self.knights,
            Piece::Bishop => &self.bishops,
            Piece::Rook => &self.rooks,
            Piece::Queen => &self.queens,
            Piece::King => &self.kings,
        }
    }
}

struct CastlingRights {
    white_king_side: bool,
    white_queen_side: bool,
    black_king_side: bool,
    black_queen_side: bool,
}

impl CastlingRights {
    fn start() -> Self {
        Self {
            white_king_side: true,
            white_queen_side: true,
            black_king_side: true,
            black_queen_side: true,
        }
    }
}

pub(crate) struct State {
    to_move: Side,
    half_move_counter: u8,
    en_passant_target: Option<Square>,
    castling_rights: CastlingRights,
}

impl State {
    fn start() -> Self {
        Self {
            to_move: Side::White,
            half_move_counter: 0,
            en_passant_target: None,
            castling_rights: CastlingRights::start()
        }
    }
}

pub(crate) struct Position {
    state: State,
    sides: Sides,
    pieces: Pieces,
}

impl Position {
    fn start() -> Self {
        Self {
            state: State::start(),
            sides: Sides::start(),
            pieces: Pieces::start(),
        }
    }

    fn piece_type_at(&self, sq: &Square) -> Option<(Piece, Side)> {
        if self.pieces.pawns.white.is_piece_at(sq) {
            Some((Piece::Pawn, Side::White))
        } else if self.pieces.knights.white.is_piece_at(sq) {
            Some((Piece::Knight, Side::White))
        } else if self.pieces.bishops.white.is_piece_at(sq) {
            Some((Piece::Bishop, Side::White))
        } else if self.pieces.rooks.white.is_piece_at(sq) {
            Some((Piece::Rook, Side::White))
        } else if self.pieces.queens.white.is_piece_at(sq) {
            Some((Piece::Queen, Side::White))
        } else if self.pieces.kings.white.is_piece_at(sq) {
            Some((Piece::King, Side::White))
        } else if self.pieces.pawns.black.is_piece_at(sq) {
            Some((Piece::Pawn, Side::Black))
        } else if self.pieces.knights.black.is_piece_at(sq) {
            Some((Piece::Knight, Side::Black))
        } else if self.pieces.bishops.black.is_piece_at(sq) {
            Some((Piece::Bishop, Side::Black))
        } else if self.pieces.rooks.black.is_piece_at(sq) {
            Some((Piece::Rook, Side::Black))
        } else if self.pieces.queens.black.is_piece_at(sq) {
            Some((Piece::Queen, Side::Black))
        } else if self.pieces.kings.black.is_piece_at(sq) {
            Some((Piece::King, Side::Black))
        } else {
            None
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut board_str = String::with_capacity(64 + 7);
        for (idx, sq) in Square::iter().enumerate().rev() {
            let piece_side = self.piece_type_at(&sq);
            let ch = match piece_side {
                Some((Piece::Pawn, ..)) => 'p',
                Some((Piece::Knight, ..)) => 'n',
                Some((Piece::Bishop, ..)) => 'b',
                Some((Piece::Rook, ..)) => 'r',
                Some((Piece::Queen, ..)) => 'q',
                Some((Piece::King, ..)) => 'k',
                None => '.'
            };
            board_str.push(match piece_side {
                Some((_, Side::White)) => ch.to_ascii_uppercase(),
                _ => ch,
            });

            if idx % 8 == 0 && idx != 0 {
                board_str.push('\n');
            }
        }
        write!(f, "{}", board_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        let got = Position::start();
        let want = "RNBQKBNR\nPPPPPPPP\n........\n........\n........\n........\npppppppp\nrnbqkbnr";

        assert_eq!(format!("{}", got), want);
    }

    #[test]
    fn test_state_start() {
        let got = Position::start();

        assert!(got.state.castling_rights.white_king_side);
        assert!(got.state.castling_rights.white_queen_side);
        assert!(got.state.castling_rights.black_king_side);
        assert!(got.state.castling_rights.black_queen_side);

        assert_eq!(got.state.half_move_counter, 0);
        assert_eq!(got.state.en_passant_target, None);
        assert_eq!(got.state.to_move, Side::White);
    }
}

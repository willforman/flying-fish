use std::fmt;
use std::ops::{Index, IndexMut};

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::bitboard::{BitBoard,Square};
use crate::bitboard::Square::*;

mod fen;

#[derive(thiserror::Error, Debug)]
pub enum PositionError {
    #[error("char -> piece: got {0}")]
    FromCharPiece(char),
}

#[derive(Debug, PartialEq, Eq, EnumIter)]
pub(crate) enum Side {
    White,
    Black
}

#[derive(Debug, PartialEq, Eq, EnumIter)]
pub(crate) enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King
}

impl Into<char> for Piece {
    fn into(self) -> char {
        match self {
            Piece::Pawn => 'p',
            Piece::Knight => 'n',
            Piece::Bishop => 'b',
            Piece::Rook => 'r',
            Piece::Queen => 'q',
            Piece::King => 'k'
        }
    }
}

impl TryFrom<char> for Piece {
    type Error = PositionError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'p' => Ok(Piece::Pawn),
            'n' => Ok(Piece::Knight),
            'b' => Ok(Piece::Bishop),
            'r' => Ok(Piece::Rook),
            'q' => Ok(Piece::Queen),
            'k' => Ok(Piece::King),
            _ => Err(PositionError::FromCharPiece(value))
        }
    }
}

pub(crate) struct Sides {
    white: BitBoard,
    black: BitBoard
}

impl Sides {
    fn new() -> Self {
        Self {
            white: BitBoard::empty(),
            black: BitBoard::empty(),
        }
    }
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

impl Index<&Side> for Sides {
    type Output = BitBoard;

    fn index(&self, index: &Side) -> &Self::Output {
        match index {
            Side::White => &self.white,
            Side::Black => &self.black,
        }
    }
}

impl IndexMut<&Side> for Sides {
    fn index_mut(&mut self, index: &Side) -> &mut Self::Output {
        match index {
            Side::White => &mut self.white,
            Side::Black => &mut self.black,
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
    fn new() -> Self {
        Self {
            pawns: Sides::new(),
            knights: Sides::new(),
            bishops: Sides::new(),
            rooks: Sides::new(),
            queens: Sides::new(),
            kings: Sides::new(),
        }
    }
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

impl IndexMut<&Piece> for Pieces {
    fn index_mut(&mut self, index: &Piece) -> &mut Self::Output {
        match index {
            Piece::Pawn => &mut self.pawns,
            Piece::Knight => &mut self.knights,
            Piece::Bishop => &mut self.bishops,
            Piece::Rook => &mut self.rooks,
            Piece::Queen => &mut self.queens,
            Piece::King => &mut self.kings,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct CastlingRights {
    pub(crate) white_king_side: bool,
    pub(crate) white_queen_side: bool,
    pub(crate) black_king_side: bool,
    pub(crate) black_queen_side: bool,
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

    pub(crate) fn new(white_king_side: bool, white_queen_side: bool, black_king_side: bool, black_queen_side: bool) -> Self {
        Self {
            white_king_side,
            white_queen_side,
            black_king_side,
            black_queen_side,
        }
    }
}

pub(crate) struct State {
    to_move: Side,
    half_move_clock: u8,
    en_passant_target: Option<Square>,
    castling_rights: CastlingRights,
}

impl State {
    fn start() -> Self {
        Self {
            to_move: Side::White,
            half_move_clock: 0,
            en_passant_target: None,
            castling_rights: CastlingRights::start()
        }
    }
}

pub struct Position {
    state: State,
    sides: Sides,
    pieces: Pieces,
}

impl Position {
    pub(crate) fn start() -> Self {
        Self {
            state: State::start(),
            sides: Sides::start(),
            pieces: Pieces::start(),
        }
    }

    fn is_piece_at(&self, square: &Square) -> Option<(Piece, Side)> {
        for piece in Piece::iter() {
            let sides = &self.pieces[&piece];
            if sides.white.is_piece_at(square) {
                return Some((piece, Side::White));
            }
            else if sides.black.is_piece_at(square) {
                return Some((piece, Side::Black));
            }
        }

        None
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut board_str = String::with_capacity(64 + 7);
        for (idx, sq) in Square::iter().enumerate() {
            let ch = match self.is_piece_at(&sq) {
                Some((p, Side::White)) => <Piece as Into<char>>::into(p).to_ascii_uppercase(),
                Some((p, Side::Black)) => <Piece as Into<char>>::into(p),
                None => '.',
            };

            board_str.push(ch);

            if (idx + 1) % 8 == 0 && (idx + 1) != 64 {
                board_str.push('\n');
            }
        }
        write!(f, "{}", board_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use testresult::TestResult;

    #[test]
    fn test_display() {
        let got = Position::start();
        let want = "RNBQKBNR\nPPPPPPPP\n........\n........\n........\n........\npppppppp\nrnbqkbnr";

        assert_eq!(format!("{}", got), want);
    }

    #[test]
    fn test_state_start() {
        let pos = Position::start();

        assert!(pos.state.castling_rights.white_king_side);
        assert!(pos.state.castling_rights.white_queen_side);
        assert!(pos.state.castling_rights.black_king_side);
        assert!(pos.state.castling_rights.black_queen_side);

        assert_eq!(pos.state.half_move_clock, 0);
        assert_eq!(pos.state.en_passant_target, None);
        assert_eq!(pos.state.to_move, Side::White);
    }

}

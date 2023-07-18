use std::fmt;
use std::ops::{Index, IndexMut};

use strum::IntoEnumIterator;
use strum_macros::{Display,EnumIter};

use crate::bitboard::{BitBoard,Square,Move};
use crate::bitboard::Square::*;

mod fen;

#[derive(thiserror::Error, Debug)]
pub enum PositionError {
    #[error("char -> piece: got {0}")]
    FromCharPiece(char),

    #[error("no piece at {0}")]
    MoveNoPiece(String),

    #[error("to_move is the other side, for move: {0} {1} -> {2}")]
    MoveNotToMove(String, String, String),
}

#[derive(Debug, PartialEq, Eq, EnumIter, Clone, Copy, Display)]
pub enum Side {
    White,
    Black
}

impl Side {
    pub(crate) fn opposite_side(self) -> Side {
        if self == Side::White { Side::Black } else { Side::White }
    }
}

#[derive(Debug, PartialEq, Eq, EnumIter, Clone, Copy, Display)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King
}

pub(crate) const SLIDING_PIECES: [Piece; 3] = [Piece::Bishop, Piece::Rook, Piece::Queen];

impl Piece {
    pub(crate) fn is_slider(&self) -> bool {
        match self {
            Piece::Pawn | Piece::Knight | Piece::King => false,
            Piece::Bishop | Piece::Rook | Piece::Queen => true,
        }
    }
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

#[derive(Clone, Debug, PartialEq, Eq)]
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

    pub(crate) fn get(&self, side: Side) -> BitBoard {
        match side {
            Side::White => self.white,
            Side::Black => self.black,
        }
    }

    fn get_mut(&mut self, side: Side) -> &mut BitBoard {
        match side {
            Side::White => &mut self.white,
            Side::Black => &mut self.black,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
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

    pub(crate) fn get(&self, piece: Piece) -> &Sides {
        match piece {
            Piece::Pawn => &self.pawns,
            Piece::Knight => &self.knights,
            Piece::Bishop => &self.bishops,
            Piece::Rook => &self.rooks,
            Piece::Queen => &self.queens,
            Piece::King => &self.kings,
        }
    }

    fn get_mut(&mut self, piece: Piece) -> &mut Sides {
        match piece {
            Piece::Pawn => &mut self.pawns,
            Piece::Knight => &mut self.knights,
            Piece::Bishop => &mut self.bishops,
            Piece::Rook => &mut self.rooks,
            Piece::Queen => &mut self.queens,
            Piece::King => &mut self.kings,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct State {
    pub(crate) to_move: Side,
    pub(crate) half_move_clock: u8,
    pub(crate) en_passant_target: Option<Square>,
    pub(crate) castling_rights: CastlingRights,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Position {
    pub(crate) state: State,
    pub(crate) sides: Sides,
    pub(crate) pieces: Pieces,
}

impl Position {
    pub fn start() -> Self {
        Self {
            state: State::start(),
            sides: Sides::start(),
            pieces: Pieces::start(),
        }
    }

    pub(crate) fn is_piece_at(&self, square: Square) -> Option<(Piece, Side)> {
        for piece in Piece::iter() {
            let sides = &self.pieces.get(piece);
            if sides.white.is_piece_at(square) {
                return Some((piece, Side::White));
            }
            else if sides.black.is_piece_at(square) {
                return Some((piece, Side::Black));
            }
        }

        None
    }

    pub fn make_move(&mut self, mve: Move) -> Result<(), PositionError> {
        if let Some((piece, side)) = self.is_piece_at(mve.src) {
            if side != self.state.to_move {
                Err(PositionError::MoveNotToMove(side.to_string(), mve.src.to_string(), mve.dest.to_string()))
            } else {
                self.state.to_move = side.opposite_side();

                if piece == Piece::Pawn || self.is_piece_at(mve.dest).is_some() {
                    self.state.half_move_clock = 0;
                } else {
                    self.state.half_move_clock += 1;
                }

                if piece == Piece::Pawn && mve.src.abs_diff(mve.dest) == 16 {
                    self.state.en_passant_target = Some(mve.dest);
                } else {
                    self.state.en_passant_target = None;
                }

                if piece == Piece::King {
                    if side == Side::White {
                        self.state.castling_rights.white_king_side = false;
                        self.state.castling_rights.white_queen_side = false;
                    } else {
                        self.state.castling_rights.black_king_side = false;
                        self.state.castling_rights.black_queen_side = false;
                    }
                }

                if piece == Piece::Rook {
                    if mve.src == A1 {
                        self.state.castling_rights.white_queen_side = false;
                    } else if mve.src == H1 {
                        self.state.castling_rights.white_king_side = false;
                    }
                    if mve.src == A8 {
                        self.state.castling_rights.black_queen_side = false;
                    }
                    if mve.src == H8 {
                        self.state.castling_rights.black_king_side = false;
                    }
                }

                if let Some((opp_piece, opp_side)) = self.is_piece_at(mve.dest) {
                    self.sides.get_mut(opp_side).clear_square(mve.dest);
                    self.pieces.get_mut(opp_piece).get_mut(opp_side).clear_square(mve.dest);
                }

                self.sides.get_mut(side).move_piece(mve);
                self.pieces.get_mut(piece).get_mut(side).move_piece(mve);

                Ok(())
            }
        } else {
            Err(PositionError::MoveNoPiece(mve.src.to_string()))
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut board_str = String::with_capacity(64 + 7);
        for (idx, sq) in Square::iter().enumerate() {
            let ch = match self.is_piece_at(sq) {
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
    use crate::bitboard::Square::*;
    use testresult::TestResult;
    use test_case::test_case;

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

    #[test_case(Position::start(), Move { src: D2, dest: D4 })]
    fn test_make_move(mut position: Position, mve: Move) {
        assert!(position.is_piece_at(mve.src).is_some());
        assert!(position.is_piece_at(mve.dest).is_none());

        let res = position.make_move(mve);

        assert!(res.is_ok());

        assert!(position.is_piece_at(mve.src).is_none());
        assert!(position.is_piece_at(mve.dest).is_some());
    }

    #[test_case(Position::start(), Move { src: D7, dest: D5 })]
    fn test_make_move_err(mut position: Position, mve: Move) {
        let res = position.make_move(mve);
        assert!(res.is_err());
    }
}

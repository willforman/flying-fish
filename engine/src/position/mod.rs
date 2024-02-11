use std::fmt;

use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

use crate::bitboard::Square::*;
use crate::bitboard::{BitBoard, Direction, Square};

mod fen;

#[derive(thiserror::Error, Debug)]
pub enum PositionError {
    #[error("char -> piece: got {0}")]
    FromCharPiece(char),

    #[error("no piece at {0}")]
    MoveNoPiece(String),

    #[error("no piece at {0}")]
    RemoveNoPiece(String),

    #[error("to_move is the other side, for move: {0} {1} -> {2}")]
    MoveNotToMove(String, String, String),
}

#[derive(Debug, PartialEq, Eq, EnumIter, Clone, Copy, Display, Deserialize, Serialize)]
pub enum Side {
    White,
    Black,
}

impl Side {
    pub(crate) fn opposite_side(self) -> Side {
        if self == Side::White {
            Side::Black
        } else {
            Side::White
        }
    }
}

#[derive(Debug, PartialEq, Eq, EnumIter, Clone, Copy, Display, Hash, Deserialize, Serialize)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

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
            Piece::King => 'k',
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
            _ => Err(PositionError::FromCharPiece(value)),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Move {
    pub src: Square,
    pub dest: Square,
    pub promotion: Option<Piece>,
}

impl Move {
    pub fn new(src: Square, dest: Square) -> Move {
        Self {
            src,
            dest,
            promotion: None,
        }
    }

    pub fn with_promotion(src: Square, dest: Square, promotion: Piece) -> Self {
        Self {
            src,
            dest,
            promotion: Some(promotion),
        }
    }
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {}", self.src, self.dest)?;
        if self.promotion.is_some() {
            write!(f, " ({})", self.promotion.unwrap())?;
        }
        Ok(())
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct Sides {
    white: BitBoard,
    black: BitBoard,
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
                A1, B1, C1, D1, E1, F1, G1, H1, A2, B2, C2, D2, E2, F2, G2, H2,
            ]),
            black: BitBoard::from_squares(&[
                A7, B7, C7, D7, E7, F7, G7, H7, A8, B8, C8, D8, E8, F8, G8, H8,
            ]),
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

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
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
                black: { BitBoard::from_squares(&[A7, B7, C7, D7, E7, F7, G7, H7]) },
            },
            knights: Sides {
                white: { BitBoard::from_squares(&[B1, G1]) },
                black: { BitBoard::from_squares(&[B8, G8]) },
            },
            bishops: Sides {
                white: { BitBoard::from_squares(&[C1, F1]) },
                black: { BitBoard::from_squares(&[C8, F8]) },
            },
            rooks: Sides {
                white: { BitBoard::from_squares(&[A1, H1]) },
                black: { BitBoard::from_squares(&[A8, H8]) },
            },
            queens: Sides {
                white: { BitBoard::from_squares(&[D1]) },
                black: { BitBoard::from_squares(&[D8]) },
            },
            kings: Sides {
                white: { BitBoard::from_squares(&[E1]) },
                black: { BitBoard::from_squares(&[E8]) },
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

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct CastlingRights {
    pub white_king_side: bool,
    pub white_queen_side: bool,
    pub black_king_side: bool,
    pub black_queen_side: bool,
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

    pub(crate) fn new(
        white_king_side: bool,
        white_queen_side: bool,
        black_king_side: bool,
        black_queen_side: bool,
    ) -> Self {
        Self {
            white_king_side,
            white_queen_side,
            black_king_side,
            black_queen_side,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct State {
    pub to_move: Side,
    pub half_move_clock: u8,
    pub en_passant_target: Option<Square>,
    pub castling_rights: CastlingRights,
    pub full_move_counter: u16,
}

impl State {
    fn start() -> Self {
        Self {
            to_move: Side::White,
            half_move_clock: 0,
            en_passant_target: None,
            castling_rights: CastlingRights::start(),
            full_move_counter: 1,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Position {
    pub state: State,
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

    pub fn is_piece_at(&self, square: Square) -> Option<(Piece, Side)> {
        for piece in Piece::iter() {
            let sides = &self.pieces.get(piece);
            if sides.white.is_square_set(square) {
                return Some((piece, Side::White));
            } else if sides.black.is_square_set(square) {
                return Some((piece, Side::Black));
            }
        }

        None
    }

    pub fn make_move(&mut self, mve: &Move) -> Result<(), PositionError> {
        if self.state.to_move == Side::Black {
            self.state.full_move_counter += 1;
        }

        if let Some((piece, side)) = self.is_piece_at(mve.src) {
            if side != self.state.to_move {
                Err(PositionError::MoveNotToMove(
                    side.to_string(),
                    mve.src.to_string(),
                    mve.dest.to_string(),
                ))
            } else {
                self.state.to_move = side.opposite_side();

                if piece == Piece::Pawn || self.is_piece_at(mve.dest).is_some() {
                    self.state.half_move_clock = 0;
                } else {
                    self.state.half_move_clock += 1;
                }

                if piece == Piece::Pawn && mve.src.abs_diff(mve.dest) == 16 {
                    let ep_dir = if side == Side::White {
                        Direction::IncRank
                    } else {
                        Direction::DecRank
                    };

                    let mut ep_target_bb = BitBoard::from_square(mve.src);
                    ep_target_bb.shift(ep_dir);
                    let ep_target = ep_target_bb.to_squares()[0];

                    self.state.en_passant_target = Some(ep_target);
                } else {
                    self.state.en_passant_target = None;
                }

                if let Some((opp_piece, opp_side)) = self.is_piece_at(mve.dest) {
                    self.sides.get_mut(opp_side).clear_square(mve.dest);
                    self.pieces
                        .get_mut(opp_piece)
                        .get_mut(opp_side)
                        .clear_square(mve.dest);
                }

                if piece == Piece::Pawn && (mve.dest >= A8 || mve.dest <= H1) {
                    // Promotion
                    self.sides.get_mut(side).move_piece(mve.src, mve.dest);

                    self.pieces
                        .get_mut(Piece::Pawn)
                        .get_mut(side)
                        .clear_square(mve.src);
                    self.pieces
                        .get_mut(mve.promotion.unwrap())
                        .get_mut(side)
                        .set_square(mve.dest);

                    return Ok(());
                }

                if piece == Piece::King {
                    if side == Side::White {
                        self.state.castling_rights.white_king_side = false;
                        self.state.castling_rights.white_queen_side = false;
                    } else {
                        self.state.castling_rights.black_king_side = false;
                        self.state.castling_rights.black_queen_side = false;
                    }

                    if mve.src.abs_diff(mve.dest) == 2 {
                        // Castled
                        let rook_move = match mve.dest {
                            C1 => Move::new(A1, D1),
                            G1 => Move::new(H1, F1),
                            C8 => Move::new(A8, D8),
                            G8 => Move::new(H8, F8),
                            _ => panic!("want: [C1, G1, C8, G8], got: {}", mve.dest),
                        };

                        self.sides
                            .get_mut(side)
                            .move_piece(rook_move.src, rook_move.dest);
                        self.pieces
                            .get_mut(Piece::Rook)
                            .get_mut(side)
                            .move_piece(rook_move.src, rook_move.dest);
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

                self.sides.get_mut(side).move_piece(mve.src, mve.dest);
                self.pieces
                    .get_mut(piece)
                    .get_mut(side)
                    .move_piece(mve.src, mve.dest);

                debug_assert!(
                    !self.pieces.kings.white.is_empty(),
                    "position somehow lost white king\n{:?}",
                    self
                );
                debug_assert!(
                    !self.pieces.kings.white.is_empty(),
                    "position somehow lost black king\n{:?}",
                    self
                );

                Ok(())
            }
        } else {
            Err(PositionError::MoveNoPiece(mve.src.to_string()))
        }
    }

    pub fn remove_piece(&mut self, square: Square) -> Result<(), PositionError> {
        if let Some((piece, side)) = self.is_piece_at(square) {
            self.sides.get_mut(side).clear_square(square);
            self.pieces
                .get_mut(piece)
                .get_mut(side)
                .clear_square(square);
            Ok(())
        } else {
            Err(PositionError::RemoveNoPiece(square.to_string()))
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut board_str = String::with_capacity(64 + 7);
        Square::list_white_perspective()
            .into_iter()
            .enumerate()
            .for_each(|(idx, square)| {
                let ch = match self.is_piece_at(square) {
                    Some((p, Side::White)) => <Piece as Into<char>>::into(p).to_ascii_uppercase(),
                    Some((p, Side::Black)) => <Piece as Into<char>>::into(p),
                    None => '.',
                };

                board_str.push(ch);
                if (idx + 1) % 8 == 0 && idx != 63 {
                    board_str.push('\n');
                }
            });
        write!(f, "{}", board_str)
    }
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;
    use testresult::TestResult;

    #[test]
    fn test_display() {
        let got = Position::start();
        let want = "rnbqkbnr\npppppppp\n........\n........\n........\n........\nPPPPPPPP\nRNBQKBNR";

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

    #[test_case(Position::start(), Move::new(D2, D4))]
    fn test_make_move(mut position: Position, mve: Move) {
        assert!(position.is_piece_at(mve.src).is_some());
        assert!(position.is_piece_at(mve.dest).is_none());

        let res = position.make_move(&mve);

        assert!(res.is_ok());

        assert!(position.is_piece_at(mve.src).is_none());
        assert!(position.is_piece_at(mve.dest).is_some());
    }

    #[test_case(Position::start(), Move::new(D7, D5))]
    fn test_make_move_err(mut position: Position, mve: Move) {
        let res = position.make_move(&mve);
        assert!(res.is_err());
    }

    #[test_case(Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap(), 
        Move::new(A2, A4),A3 ; "kiwipete")]
    fn test_make_move_ep_target(
        mut position: Position,
        mve: Move,
        want_en_passant_target: Square,
    ) -> TestResult {
        position.make_move(&mve)?;
        assert!(position.state.en_passant_target.is_some());
        assert_eq!(
            position.state.en_passant_target.unwrap(),
            want_en_passant_target
        );
        Ok(())
    }

    #[test_case(Move::new(A1, G7), "A1 -> G7" ; "no promotion")]
    #[test_case(Move::with_promotion(F7, B6, Piece::Queen), "F7 -> B6 (Queen)" ; "with promotion")]
    fn test_move_debug(mve: Move, want: &str) {
        let got = format!("{:?}", mve);
        assert_eq!(got, want);
    }
}

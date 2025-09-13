use std::fmt;

use arrayvec::ArrayVec;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

use crate::bitboard::Square::*;
use crate::bitboard::{BitBoard, Direction, Square};
use crate::position;
use crate::position::zobrist_hash::ZobristHash;

mod fen;
mod zobrist_hash;

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

    #[error("game is over, half move clock is at 50: move {0}")]
    GameOverHalfMoveClock(String),

    #[error("cannot undo move, previous move isn't stored")]
    NoMoveToUndo,

    #[error("cannot move pawn to last row without promotion: move {0}")]
    PawnMoveMissingPromotion(Move),
}

#[derive(Debug, PartialEq, Eq, EnumIter, Clone, Copy, Display)]
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

#[derive(Debug, PartialEq, Eq, EnumIter, Clone, Copy, Display, Hash, PartialOrd, Ord)]
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

    #[inline(always)]
    pub(crate) fn index(self) -> usize {
        self as usize
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

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
        if let Some(promotion) = self.promotion {
            let promotion_ch: char = promotion.into();
            write!(f, "{}{}{}", self.src, self.dest, promotion_ch)
        } else {
            write!(f, "{}{}", self.src, self.dest)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq, Clone)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub to_move: Side,
    pub half_move_clock: u8,
    pub en_passant_target: Option<Square>,
    pub castling_rights: CastlingRights,
    pub full_move_counter: u8,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnmakeMoveState {
    mve: Move,
    piece_moved: Piece,
    captured_piece: Option<Piece>,
    castling_rights: CastlingRights,
    en_passant_target: Option<Square>,
    half_move_clock: u8,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Position {
    pub state: State,
    pub(crate) sides: Sides,
    pub(crate) pieces: Pieces,
    pub(crate) zobrist_hash: ZobristHash,
}

impl Position {
    pub fn start() -> Self {
        Self {
            state: State::start(),
            sides: Sides::start(),
            pieces: Pieces::start(),
            zobrist_hash: ZobristHash::calculate(&Pieces::start(), &State::start()),
        }
    }

    pub fn is_piece_at(&self, square: Square, side: Side) -> Option<Piece> {
        for piece in Piece::iter() {
            if self.pieces.get(piece).get(side).is_square_set(square) {
                return Some(piece);
            }
        }

        None
    }

    pub fn is_piece_at_no_side(&self, square: Square) -> Option<(Piece, Side)> {
        if let Some(piece) = self.is_piece_at(square, Side::White) {
            return Some((piece, Side::White));
        }
        if let Some(piece) = self.is_piece_at(square, Side::Black) {
            return Some((piece, Side::Black));
        }
        None
    }

    pub fn is_capture(&self, mve: &Move) -> bool {
        let opp_pieces = &self.sides.get(self.state.to_move.opposite_side());
        opp_pieces.is_square_set(mve.dest)
    }

    pub fn make_move(&mut self, mve: Move) -> UnmakeMoveState {
        debug_assert!(
            self.state.half_move_clock < 50,
            "Game is over the half move clock"
        );

        let side = self.state.to_move;
        let opp_side = side.opposite_side();

        let piece = self
            .is_piece_at(mve.src, side)
            .expect("No piece to move found");

        if self.state.to_move == Side::Black {
            self.state.full_move_counter += 1;
        }
        self.zobrist_hash.flip_side_to_move();

        if let Some(en_passant_target) = self.state.en_passant_target {
            // Clear previous en passant target.
            self.zobrist_hash.flip_en_passant_file(en_passant_target);
            if mve.dest == en_passant_target && piece == Piece::Pawn {
                let ep_capture_dir = if side == Side::White {
                    Direction::DecRank
                } else {
                    Direction::IncRank
                };

                let mut ep_capture_bb = BitBoard::from_square(en_passant_target);
                ep_capture_bb.shift(ep_capture_dir);
                let ep_capture_sq = ep_capture_bb.to_squares()[0];

                self.move_piece(mve.src, en_passant_target, Piece::Pawn, side);

                self.remove_piece(ep_capture_sq, Piece::Pawn, opp_side);
                self.state.en_passant_target = None;
                self.state.to_move = self.state.to_move.opposite_side();

                return UnmakeMoveState {
                    mve,
                    piece_moved: Piece::Pawn,
                    captured_piece: Some(Piece::Pawn),
                    en_passant_target: Some(mve.dest),
                    half_move_clock: 0,
                    castling_rights: self.state.castling_rights.clone(),
                };
            }
        }

        let captured_piece = self.is_piece_at(mve.dest, opp_side);

        let unmake_move_state = UnmakeMoveState {
            mve,
            piece_moved: piece,
            captured_piece,
            en_passant_target: self.state.en_passant_target,
            half_move_clock: self.state.half_move_clock,
            castling_rights: self.state.castling_rights.clone(),
        };

        self.state.to_move = opp_side;

        if piece == Piece::Pawn || captured_piece.is_some() {
            self.state.half_move_clock = 0;
        } else {
            debug_assert!(
                self.state.half_move_clock != 255,
                "half move clock handling is incorrect, would have overflowed"
            );
            self.state.half_move_clock += 1;
        }

        if let Some(opp_piece) = captured_piece {
            self.remove_piece(mve.dest, opp_piece, opp_side);

            if opp_piece == Piece::Rook {
                if self.state.castling_rights.white_king_side && mve.dest == H1 {
                    self.state.castling_rights.white_king_side = false;
                    self.zobrist_hash.flip_castling_rights_white_kingside();
                } else if self.state.castling_rights.white_queen_side && mve.dest == A1 {
                    self.state.castling_rights.white_queen_side = false;
                    self.zobrist_hash.flip_castling_rights_white_queenside();
                } else if self.state.castling_rights.black_king_side && mve.dest == H8 {
                    self.state.castling_rights.black_king_side = false;
                    self.zobrist_hash.flip_castling_rights_black_kingside();
                } else if self.state.castling_rights.black_queen_side && mve.dest == A8 {
                    self.state.castling_rights.black_queen_side = false;
                    self.zobrist_hash.flip_castling_rights_black_queenside();
                }
            }
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
            self.zobrist_hash.flip_en_passant_file(ep_target);
        } else {
            self.state.en_passant_target = None;
        }

        // Promotion
        if piece == Piece::Pawn && (mve.dest >= A8 || mve.dest <= H1) {
            let promotion = mve
                .promotion
                .expect("Pawn moved to end of board, expected promotion");

            self.remove_piece(mve.src, Piece::Pawn, side);
            self.add_piece(mve.dest, promotion, side);

            return unmake_move_state;
        }

        if piece == Piece::King {
            if side == Side::White {
                if self.state.castling_rights.white_queen_side {
                    self.state.castling_rights.white_queen_side = false;
                    self.zobrist_hash.flip_castling_rights_white_queenside();
                }
                if self.state.castling_rights.white_king_side {
                    self.state.castling_rights.white_king_side = false;
                    self.zobrist_hash.flip_castling_rights_white_kingside();
                }
            } else {
                if self.state.castling_rights.black_queen_side {
                    self.state.castling_rights.black_queen_side = false;
                    self.zobrist_hash.flip_castling_rights_black_queenside();
                }
                if self.state.castling_rights.black_king_side {
                    self.state.castling_rights.black_king_side = false;
                    self.zobrist_hash.flip_castling_rights_black_kingside();
                }
            }

            if mve.src.abs_diff(mve.dest) == 2 {
                // Castled
                let (rook_src, rook_dest) = match mve.dest {
                    C1 => (A1, D1),
                    G1 => (H1, F1),
                    C8 => (A8, D8),
                    G8 => (H8, F8),
                    _ => unreachable!("want: [C1, G1, C8, G8], got: {}", mve.dest),
                };

                self.move_piece(rook_src, rook_dest, Piece::Rook, side);
            }
        }

        if piece == Piece::Rook {
            if self.state.castling_rights.white_queen_side && mve.src == A1 {
                self.state.castling_rights.white_queen_side = false;
                self.zobrist_hash.flip_castling_rights_white_queenside();
            } else if self.state.castling_rights.white_king_side && mve.src == H1 {
                self.state.castling_rights.white_king_side = false;
                self.zobrist_hash.flip_castling_rights_white_kingside();
            } else if self.state.castling_rights.black_queen_side && mve.src == A8 {
                self.state.castling_rights.black_queen_side = false;
                self.zobrist_hash.flip_castling_rights_black_queenside();
            } else if self.state.castling_rights.black_king_side && mve.src == H8 {
                self.state.castling_rights.black_king_side = false;
                self.zobrist_hash.flip_castling_rights_black_kingside();
            }
        }

        self.move_piece(mve.src, mve.dest, piece, side);

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

        unmake_move_state
    }

    pub fn unmake_move(&mut self, undo_move_state: UnmakeMoveState) -> Result<(), PositionError> {
        let mve = undo_move_state.mve;
        let opp_side = self.state.to_move;
        let moved_side = opp_side.opposite_side();
        let mut piece_moved = undo_move_state.piece_moved;

        if self.state.castling_rights.white_queen_side
            != undo_move_state.castling_rights.white_queen_side
        {
            self.zobrist_hash.flip_castling_rights_white_queenside();
        }
        if self.state.castling_rights.white_queen_side
            != undo_move_state.castling_rights.white_king_side
        {
            self.zobrist_hash.flip_castling_rights_white_kingside();
        }
        if self.state.castling_rights.black_queen_side
            != undo_move_state.castling_rights.black_queen_side
        {
            self.zobrist_hash.flip_castling_rights_black_queenside();
        }
        if self.state.castling_rights.black_queen_side
            != undo_move_state.castling_rights.black_king_side
        {
            self.zobrist_hash.flip_castling_rights_black_kingside();
        }
        self.state.castling_rights = undo_move_state.castling_rights;

        if let Some(prev_en_passant_target) = self.state.en_passant_target {
            self.zobrist_hash
                .flip_en_passant_file(prev_en_passant_target);
        }
        self.state.en_passant_target = undo_move_state.en_passant_target;

        self.state.half_move_clock = undo_move_state.half_move_clock;
        if moved_side == Side::Black {
            self.state.full_move_counter -= 1;
        }
        self.state.to_move = moved_side;
        self.zobrist_hash.flip_side_to_move();

        self.move_piece(mve.dest, mve.src, piece_moved, moved_side);

        // If the move was a promotion, we need to make sure to put the pawn back and
        // clear the piece that was promoted.
        if let Some(promotion_piece) = mve.promotion {
            piece_moved = Piece::Pawn;
            self.remove_piece(mve.dest, promotion_piece, moved_side);
        }

        // Handle undoing castling
        if piece_moved == Piece::King && mve.src.abs_diff(mve.dest) == 2 {
            let (rook_src, rook_dest) = match mve.dest {
                C1 => (A1, D1),
                G1 => (H1, F1),
                C8 => (A8, D8),
                G8 => (H8, F8),
                _ => unreachable!("want: [C1, G1, C8, G8], got: {}", mve.dest),
            };
            self.move_piece(rook_dest, rook_src, Piece::Rook, moved_side);
        }

        if let Some(en_passant_target) = undo_move_state.en_passant_target {
            self.zobrist_hash.flip_en_passant_file(en_passant_target);
            if mve.dest == en_passant_target && undo_move_state.piece_moved == Piece::Pawn {
                let ep_capture_dir = if moved_side == Side::White {
                    Direction::DecRank
                } else {
                    Direction::IncRank
                };

                let mut ep_capture_bb = BitBoard::from_square(en_passant_target);
                ep_capture_bb.shift(ep_capture_dir);
                let ep_capture_sq = ep_capture_bb.to_squares()[0];

                self.add_piece(ep_capture_sq, Piece::Pawn, opp_side);

                return Ok(());
            }
        }

        if let Some(captured_piece) = undo_move_state.captured_piece {
            self.add_piece(mve.dest, captured_piece, opp_side);
        }

        Ok(())
    }

    fn add_piece(&mut self, square: Square, piece: Piece, side: Side) {
        debug_assert!(!self.is_piece_at(square, side).is_some());

        self.sides.get_mut(side).set_square(square);
        self.pieces.get_mut(piece).get_mut(side).set_square(square);

        self.zobrist_hash.add_piece(square, piece, side);
    }

    pub fn remove_piece(&mut self, square: Square, piece: Piece, side: Side) {
        debug_assert!(self.is_piece_at(square, side).is_some());

        self.sides.get_mut(side).clear_square(square);
        self.pieces
            .get_mut(piece)
            .get_mut(side)
            .clear_square(square);

        self.zobrist_hash.remove_piece(square, piece, side);
    }

    fn move_piece(&mut self, src_square: Square, dest_square: Square, piece: Piece, side: Side) {
        debug_assert!(self.is_piece_at(src_square, side).is_some());
        debug_assert!(!self.is_piece_at(dest_square, side).is_some());

        self.sides.get_mut(side).move_piece(src_square, dest_square);
        self.pieces
            .get_mut(piece)
            .get_mut(side)
            .move_piece(src_square, dest_square);

        self.zobrist_hash
            .move_piece(src_square, dest_square, piece, side);
    }

    pub fn piece_locs(&self) -> impl Iterator<Item = (Piece, Side, Square)> + '_ {
        Side::iter().flat_map(move |side| {
            Piece::iter().flat_map(move |piece| {
                let board_for_piece_side = self.pieces.get(piece).get(side);
                board_for_piece_side
                    .squares()
                    .map(move |sq| (piece, side, sq))
            })
        })
    }

    pub(crate) fn validate_position(&self, mve: Move) -> Result<(), String> {
        if self.pieces.get(Piece::King).get(Side::White).is_empty() {
            return Err("White king missing".to_string());
        }
        if self.pieces.get(Piece::King).get(Side::Black).is_empty() {
            return Err("Black king missing".to_string());
        }

        let pieces_vec: Vec<_> = Piece::iter().collect();
        // Check that no pieces have the same square set.
        for (idx, piece_outer) in pieces_vec.iter().enumerate() {
            for piece_inner in &pieces_vec[idx + 1..] {
                let bb_outer = self.pieces.get(*piece_outer).get(Side::White);
                let bb_inner = self.pieces.get(*piece_inner).get(Side::White);

                let intersection = bb_outer & bb_inner;
                if !intersection.is_empty() {
                    return Err(format!(
                        "Invalid state: move={:?} caused {:?} {:?} and {:?} to set the same squares: {:?}\n{:?}",
                        mve,
                        Side::White,
                        piece_outer,
                        piece_inner,
                        intersection.to_squares(),
                        intersection
                    ));
                }
            }
        }

        for (idx, piece_outer) in pieces_vec.iter().enumerate() {
            for piece_inner in &pieces_vec[idx + 1..] {
                let bb_outer = self.pieces.get(*piece_outer).get(Side::Black);
                let bb_inner = self.pieces.get(*piece_inner).get(Side::Black);

                let intersection = bb_outer & bb_inner;
                if !intersection.is_empty() {
                    return Err(format!(
                        "Invalid state: move={:?} caused {:?} {:?} and {:?} to set the same squares: {:?}\n{:?}",
                        mve,
                        Side::Black,
                        piece_outer,
                        piece_inner,
                        intersection.to_squares(),
                        intersection
                    ));
                }
            }
        }
        Ok(())
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_fen())
    }
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut board_str = String::with_capacity(64 + 7);
        Square::list_white_perspective()
            .into_iter()
            .enumerate()
            .for_each(|(idx, square)| {
                let ch = match (
                    self.is_piece_at(square, Side::White),
                    self.is_piece_at(square, Side::Black),
                ) {
                    (Some(p), None) => <Piece as Into<char>>::into(p).to_ascii_uppercase(),
                    (None, Some(p)) => <Piece as Into<char>>::into(p),
                    _ => '.',
                };

                board_str.push(ch);
                if (idx + 1) % 8 == 0 && idx != 63 {
                    board_str.push('\n');
                }
            });
        writeln!(f, "{}", board_str)?;
        writeln!(
            f,
            "half move={}, full move={}, en_passant={:?}, castling_rights={:?} zobrist={}",
            self.state.half_move_clock,
            self.state.full_move_counter,
            self.state.en_passant_target,
            self.state.castling_rights,
            self.zobrist_hash,
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use test_case::test_case;
    use testresult::TestResult;

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

    #[test_case(Position::start(), Move::new(D2, D4), Side::White)]
    fn test_make_move(mut position: Position, mve: Move, side: Side) {
        assert!(position.is_piece_at(mve.src, side).is_some());
        assert!(position.is_piece_at(mve.dest, side).is_none());

        let _ = position.make_move(mve);

        assert!(position.is_piece_at(mve.src, side).is_none());
        assert!(
            position
                .is_piece_at(mve.src, side.opposite_side())
                .is_none()
        );
        assert!(position.is_piece_at(mve.dest, side).is_some());
    }

    // #[test_case(Position::start(), Move::new(D7, D5))]
    // fn test_make_move_err(mut position: Position, mve: Move) {
    //     let res = position.make_move(mve);
    //     assert!(res.is_err());
    // }

    #[test_case(Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap(), 
        Move::new(A2, A4),A3 ; "kiwipete")]
    fn test_make_move_ep_target(
        mut position: Position,
        mve: Move,
        want_en_passant_target: Square,
    ) -> TestResult {
        position.make_move(mve);
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

    #[test_case(Position::from_fen("7k/8/4q3/8/8/4R3/5P2/K7 b - - 0 1").unwrap(), Move::new(E6, E3), true)]
    #[test_case(Position::from_fen("7k/8/4q3/8/8/4R3/5P2/K7 b - - 0 1").unwrap(), Move::new(E6, E4), false)]
    fn test_is_capture(position: Position, mve: Move, is_capture_want: bool) {
        let is_capture_got = position.is_capture(&mve);

        assert_eq!(is_capture_got, is_capture_want);
    }

    #[test_case(Position::start(), Move::new(D2, D4))]
    #[test_case(Position::from_fen("k7/8/8/8/8/8/8/4K2R w K - 0 1").unwrap(), Move::new(E1, G1) ; "castling kingside")]
    #[test_case(Position::from_fen("k7/8/8/8/8/8/8/R3K3 w K - 0 1").unwrap(), Move::new(E1, C1) ; "castling queenside")]
    #[test_case(Position::from_fen("k7/8/8/5Pp1/8/8/8/7K w - g6 0 1").unwrap(), Move::new(F5, G6) ; "en passant white")]
    fn test_unmake_move(position: Position, mve: Move) -> TestResult {
        let mut move_position = position.clone();
        let undo_move_state = move_position.make_move(mve);
        println!("{:?}", undo_move_state);
        move_position.unmake_move(undo_move_state)?;

        assert_eq!(move_position, position);
        Ok(())
    }

    #[test_case(Position::from_fen("r3k2r/p1pPqPb1/Bn2PnP1/3PN3/1p2P3/2N4P/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap(), &[Move::new(E1, D1)] ; "kiwipete to move flipped")]
    fn test_validate_state_after_moves(mut position: Position, moves: &[Move]) -> TestResult {
        for &mve in moves {
            position.make_move(mve);
            position.validate_position(mve)?;
        }
        Ok(())
    }

    #[test_case(Position::start(), vec![Move::new(E2, E4), Move::new(E7, E5)] ; "basic")]
    #[test_case(Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 0").unwrap(), vec![Move::new(F3, H3), Move::new(C7, C5), Move::new(D5, C6), Move::new(B6, C4), Move::new(E1, D1)] ; "en passant and castling")]
    fn test_zobrist_hash_updating(mut position: Position, moves: Vec<Move>) -> TestResult {
        let mut unmake_move_state_stack = vec![];
        let mut hash_stack = vec![];
        for mve in moves.iter() {
            assert!(!hash_stack.contains(&position.zobrist_hash));
            hash_stack.push(position.zobrist_hash);

            let unmake_move_state = position.make_move(*mve);
            unmake_move_state_stack.push(unmake_move_state);

            // Ensure incremental hash is the same as hash generated from scratch.
            let full_gen_hash = ZobristHash::calculate(&position.pieces, &position.state);
            assert_eq!(
                full_gen_hash, position.zobrist_hash,
                "Incremental hash not the same for: {:?}",
                mve
            );
        }
        println!("{:?}", hash_stack);

        for (unmake_move_state, hash) in unmake_move_state_stack
            .into_iter()
            .rev()
            .zip(hash_stack.into_iter().rev())
        {
            position.unmake_move(unmake_move_state.clone())?;
            assert_eq!(
                position.zobrist_hash, hash,
                "Hash not the same after unmake move for {:?}",
                unmake_move_state.mve
            );
        }

        Ok(())
    }
}

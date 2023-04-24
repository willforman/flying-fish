use crate::bitboard::{BitBoard,Square};

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

pub(crate) struct Sides {
    white: BitBoard,
    black: BitBoard
}

// impl Sides {
//     fn start() -> Self {
//         Self {
//             BitBoard(0b)
//         }
//     }
// }

pub(crate) struct Pieces {
    pawn: Sides,
    knight: Sides,
    bishop: Sides,
    rook: Sides,
    queen: Sides,
    king: Sides,
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

// impl Position {
//     fn start() -> Self {
//         Self {
//             state: State::start(),
//             sides: 
//         }
//     }
// }

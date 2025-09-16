use std::fmt::{Display, write};

use strum::IntoEnumIterator;

use crate::{Piece, Side, Square, bitboard::BitBoard, position::State};

const RNG_SEED: u64 = 123456789;

struct RandomU64Generator {
    curr: u64,
}

impl RandomU64Generator {
    const fn new(seed: u64) -> Self {
        Self { curr: seed }
    }

    /// Standard Xorshift
    const fn generate(&mut self) -> u64 {
        let mut x = self.curr;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 5;
        self.curr = x;
        return x;
    }
}

struct ZobristRandomHashes {
    pieces: [u64; 64 * 12],
    black_to_move: u64,
    castling_rights: [u64; 4],
    en_passant_file: [u64; 8],
}

impl ZobristRandomHashes {
    const fn init() -> Self {
        let mut rng = RandomU64Generator::new(RNG_SEED);

        let mut pieces = [0; 64 * 12];
        let mut i = 0;
        while i < (64 * 12) {
            pieces[i] = rng.generate();
            i += 1;
        }

        Self {
            pieces,
            black_to_move: rng.generate(),
            castling_rights: [
                rng.generate(),
                rng.generate(),
                rng.generate(),
                rng.generate(),
            ],
            en_passant_file: [
                rng.generate(),
                rng.generate(),
                rng.generate(),
                rng.generate(),
                rng.generate(),
                rng.generate(),
                rng.generate(),
                rng.generate(),
            ],
        }
    }
}

const ZOBRIST_RANDOM_HASHES: ZobristRandomHashes = ZobristRandomHashes::init();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ZobristHash(u64);

impl Display for ZobristHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ZobristHash {
    pub(crate) fn calculate(pieces: &[BitBoard; 12], state: &State) -> ZobristHash {
        let mut hash = 0;
        for side in Side::iter() {
            for square in Square::iter() {
                for piece in Piece::iter() {
                    let bb_idx = piece as usize + (side as usize * 6);
                    if pieces[bb_idx].is_square_set(square) {
                        hash ^= ZOBRIST_RANDOM_HASHES.pieces[bb_idx * (square as usize)];
                    }
                }
            }
        }

        if state.to_move == Side::Black {
            hash ^= ZOBRIST_RANDOM_HASHES.black_to_move;
        }

        if state.castling_rights.white_king_side {
            hash ^= ZOBRIST_RANDOM_HASHES.castling_rights[0];
        }
        if state.castling_rights.white_queen_side {
            hash ^= ZOBRIST_RANDOM_HASHES.castling_rights[1];
        }
        if state.castling_rights.black_king_side {
            hash ^= ZOBRIST_RANDOM_HASHES.castling_rights[2];
        }
        if state.castling_rights.black_queen_side {
            hash ^= ZOBRIST_RANDOM_HASHES.castling_rights[3];
        }

        if let Some(en_passant_target) = state.en_passant_target {
            let en_passant_file = en_passant_target as usize / 8;
            hash ^= ZOBRIST_RANDOM_HASHES.en_passant_file[en_passant_file];
        }

        Self(hash)
    }

    pub(crate) fn add_piece(&mut self, square: Square, piece: Piece, side: Side) {
        let bb_idx = piece as usize + (side as usize * 6);

        self.0 ^= ZOBRIST_RANDOM_HASHES.pieces[bb_idx * (square as usize)];
    }

    pub(crate) fn remove_piece(&mut self, square: Square, piece: Piece, side: Side) {
        let bb_idx = piece as usize + (side as usize * 6);

        self.0 ^= ZOBRIST_RANDOM_HASHES.pieces[bb_idx * (square as usize)];
    }

    pub(crate) fn move_piece(
        &mut self,
        src_square: Square,
        dest_square: Square,
        piece: Piece,
        side: Side,
    ) {
        let bb_idx = piece as usize + (side as usize * 6);

        self.0 ^= ZOBRIST_RANDOM_HASHES.pieces[bb_idx * (src_square as usize)];
        self.0 ^= ZOBRIST_RANDOM_HASHES.pieces[bb_idx * (dest_square as usize)];
    }

    pub(crate) fn flip_side_to_move(&mut self) {
        self.0 ^= ZOBRIST_RANDOM_HASHES.black_to_move;
    }

    pub(crate) fn flip_castling_rights_white_kingside(&mut self) {
        self.0 ^= ZOBRIST_RANDOM_HASHES.castling_rights[0];
    }

    pub(crate) fn flip_castling_rights_white_queenside(&mut self) {
        self.0 ^= ZOBRIST_RANDOM_HASHES.castling_rights[1];
    }

    pub(crate) fn flip_castling_rights_black_kingside(&mut self) {
        self.0 ^= ZOBRIST_RANDOM_HASHES.castling_rights[2];
    }

    pub(crate) fn flip_castling_rights_black_queenside(&mut self) {
        self.0 ^= ZOBRIST_RANDOM_HASHES.castling_rights[3];
    }

    pub(crate) fn flip_en_passant_file(&mut self, en_passant_square: Square) {
        self.0 ^= ZOBRIST_RANDOM_HASHES.en_passant_file[en_passant_square as usize / 8];
    }
}

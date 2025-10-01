use std::fmt::Display;

use crate::GenerateMoves;
use crate::bitboard::Square;
use crate::position::{Piece, Position, Side};

/// An evaluation of a position. Is always from the side to move's perspective.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Eval(i32);

impl Display for Eval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(mate) = self.is_mate() {
            let sign = if mate % 2 == 0 { '+' } else { '-' };
            write!(f, "{}m{}", sign, mate)
        } else {
            write!(f, "cp {}", self.0)
        }
    }
}

impl Eval {
    pub const DRAW: Eval = Eval(0);
    pub const MAX: Eval = Eval::mate_in(1);
    pub const MIN: Eval = Eval::mate_in(0);

    const MATE_BASE: i32 = 30_000;

    pub const fn mate_in(ply: u8) -> Self {
        if ply % 2 == 0 {
            Self(-Self::MATE_BASE + (ply as i32 / 2))
        } else {
            Self(Self::MATE_BASE - (ply as i32 / 2))
        }
    }

    pub fn is_mate(&self) -> Option<u8> {
        let from_mate = if self.0 >= 0 {
            (Self::MATE_BASE - self.0) * 2 + 1
        } else {
            (Self::MATE_BASE + self.0) * 2
        };
        if from_mate <= 250 {
            return Some(
                from_mate
                    .try_into()
                    .expect("Bug with Eval mate calculation"),
            );
        }
        None
    }

    pub fn flip(&self) -> Eval {
        if let Some(mate) = self.is_mate() {
            Eval::mate_in(mate + 1)
        } else {
            Eval(-self.0)
        }
    }
}

pub trait EvaluatePosition {
    fn evaluate(&self, position: &Position, move_gen: impl GenerateMoves) -> Eval;
}

#[derive(Clone, Copy)]
pub struct PositionEvaluator;

// Source: https://www.chessprogramming.org/Simplified_Evaluation_Function
#[inline(always)]
fn get_piece_square_bonus(piece: Piece, square: Square, is_early_or_mid_game: bool) -> i32 {
    #[rustfmt::skip]
    let table = match (piece, is_early_or_mid_game) {
        (Piece::Pawn, ..) => [
            0,  0,  0,  0,  0,  0,  0,  0,
             5, 10, 10,-20,-20, 10, 10,  5,
             5, -5,-10,  0,  0,-10, -5,  5,
             0,  0,  0, 20, 20,  0,  0,  0,
             5,  5, 10, 25, 25, 10,  5,  5,
            10, 10, 20, 30, 30, 20, 10, 10,
            50, 50, 50, 50, 50, 50, 50, 50,
             0,  0,  0,  0,  0,  0,  0,  0
        ],
        (Piece::King, true) => [
            -30,-40,-40,-50,-50,-40,-40,-30,
            -30,-40,-40,-50,-50,-40,-40,-30,
            -30,-40,-40,-50,-50,-40,-40,-30,
            -30,-40,-40,-50,-50,-40,-40,-30,
            -20,-30,-30,-40,-40,-30,-30,-20,
            -10,-20,-20,-20,-20,-20,-20,-10,
             20, 20,  0,  0,  0,  0, 20, 20,
             20, 30, 10,  0,  0, 10, 30, 20
        ],
        (Piece::King, false) => [
            -50,-40,-30,-20,-20,-30,-40,-50,
            -30,-20,-10,  0,  0,-10,-20,-30,
            -30,-10, 20, 30, 30, 20,-10,-30,
            -30,-10, 30, 40, 40, 30,-10,-30,
            -30,-10, 30, 40, 40, 30,-10,-30,
            -30,-10, 20, 30, 30, 20,-10,-30,
            -30,-30,  0,  0,  0,  0,-30,-30,
            -50,-30,-30,-30,-30,-30,-30,-50
        ],
        (Piece::Knight, ..) => [
            -50,-40,-30,-30,-30,-30,-40,-50,
            -40,-20,  0,  0,  0,  0,-20,-40,
            -30,  0, 10, 15, 15, 10,  0,-30,
            -30,  5, 15, 20, 20, 15,  5,-30,
            -30,  0, 15, 20, 20, 15,  0,-30,
            -30,  5, 10, 15, 15, 10,  5,-30,
            -40,-20,  0,  5,  5,  0,-20,-40,
            -50,-40,-30,-30,-30,-30,-40,-50
        ],
        (Piece::Bishop, ..) => [
            -20,-10,-10,-10,-10,-10,-10,-20,
            -10,  0,  0,  0,  0,  0,  0,-10,
            -10,  0,  5, 10, 10,  5,  0,-10,
            -10,  5,  5, 10, 10,  5,  5,-10,
            -10,  0, 10, 10, 10, 10,  0,-10,
            -10, 10, 10, 10, 10, 10, 10,-10,
            -10,  5,  0,  0,  0,  0,  5,-10,
            -20,-10,-10,-10,-10,-10,-10,-20
        ],
        (Piece::Rook, ..) => [
              0,  0,  0,  0,  0,  0,  0,  0,
              5, 10, 10, 10, 10, 10, 10,  5,
             -5,  0,  0,  0,  0,  0,  0, -5,
             -5,  0,  0,  0,  0,  0,  0, -5,
             -5,  0,  0,  0,  0,  0,  0, -5,
             -5,  0,  0,  0,  0,  0,  0, -5,
             -5,  0,  0,  0,  0,  0,  0, -5,
              0,  0,  0,  5,  5,  0,  0,  0
        ],
        (Piece::Queen, ..) => [
            -20,-10,-10, -5, -5,-10,-10,-20,
            -10,  0,  0,  0,  0,  0,  0,-10,
            -10,  0,  5,  5,  5,  5,  0,-10,
             -5,  0,  5,  5,  5,  5,  0, -5,
              0,  0,  5,  5,  5,  5,  0, -5,
            -10,  5,  5,  5,  5,  5,  0,-10,
            -10,  0,  5,  0,  0,  0,  0,-10,
            -20,-10,-10, -5, -5,-10,-10,-20
        ],
    };
    table[square as usize]
}

impl EvaluatePosition for PositionEvaluator {
    /// Return evaluation relative to the side to move
    fn evaluate(&self, position: &Position, move_gen: impl GenerateMoves) -> Eval {
        if position.state.half_move_clock == 50 || position.is_threefold_repetition() {
            return Eval::DRAW;
        }
        if move_gen.gen_moves(position).is_empty() {
            if !move_gen.gen_checkers(position).is_empty() {
                return Eval::mate_in(0);
            } else {
                return Eval::DRAW;
            }
        }

        let eval_score = position.piece_locs().fold(0, |acc, (piece, side, square)| {
            // For black, we need to flip index in order to use correct value
            let square = if side == Side::Black {
                square.flip()
            } else {
                square
            };
            let val = piece_value(piece) + get_piece_square_bonus(piece, square, true);

            if side == Side::White {
                acc + val
            } else {
                acc - val
            }
        });
        let eval_score = if position.state.to_move == Side::Black {
            -eval_score
        } else {
            eval_score
        };
        Eval(eval_score)
    }
}

fn piece_value(piece: Piece) -> i32 {
    match piece {
        Piece::Pawn => 100,
        Piece::Knight => 320,
        Piece::Bishop => 330,
        Piece::Rook => 500,
        Piece::Queen => 900,
        Piece::King => 2000, // Don't use i32::MAX in case of overflows
    }
}

pub static POSITION_EVALUATOR: PositionEvaluator = PositionEvaluator {};

#[cfg(test)]
mod tests {
    use crate::move_gen::MOVE_GEN;

    use super::*;

    use test_case::test_case;
    use testresult::TestResult;

    #[test_case(Eval(10), Eval(-10))]
    #[test_case(Eval::mate_in(0), Eval::mate_in(1))]
    #[test_case(Eval::mate_in(1), Eval::mate_in(2))]
    #[test_case(Eval::mate_in(4), Eval::mate_in(5))]
    #[test_case(Eval::DRAW, Eval::DRAW)]
    fn test_eval_flip(eval_input: Eval, eval_want: Eval) {
        let eval_got = eval_input.flip();

        assert_eq!(eval_got, eval_want);
    }

    #[test]
    fn test_eval_ord() {
        let evals_order_want = vec![
            Eval::mate_in(0),
            Eval::mate_in(2),
            Eval(-20),
            Eval(-1),
            Eval::DRAW,
            Eval(1),
            Eval(20),
            Eval::mate_in(3),
            Eval::mate_in(1),
        ];
        let mut evals = vec![
            Eval::mate_in(0),
            Eval::mate_in(1),
            Eval::mate_in(2),
            Eval::mate_in(3),
            Eval(-20),
            Eval(-1),
            Eval::DRAW,
            Eval(1),
            Eval(20),
        ];

        evals.sort();

        assert_eq!(evals, evals_order_want);
    }

    #[test]
    fn test_obvious_eval() -> TestResult {
        let position = Position::from_fen("2k5/Q7/8/8/8/8/8/7K w - - 0 1")?;
        let move_gen = MOVE_GEN;
        let eval = POSITION_EVALUATOR.evaluate(&position, move_gen);

        // Should be at least 5 pawns better than the opponent
        assert!(eval > Eval(500));

        Ok(())
    }

    #[test_case(Eval::mate_in(0), Some(0))]
    #[test_case(Eval::mate_in(2), Some(2))]
    #[test_case(Eval::mate_in(1), Some(1))]
    #[test_case(Eval::mate_in(7), Some(7))]
    #[test_case(Eval::DRAW, None)]
    #[test_case(Eval(10), None)]
    fn test_is_mate(eval: Eval, is_mate_want: Option<u8>) {
        let is_mate_got = eval.is_mate();

        assert_eq!(is_mate_got, is_mate_want);
    }
}

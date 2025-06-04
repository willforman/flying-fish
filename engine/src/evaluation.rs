use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Display;
use std::ops::Neg;

use strum::IntoEnumIterator;

use crate::bitboard::Square;
use crate::position::{Piece, Position, Side};
use crate::GenerateMoves;

/// An evaluation of a position. Is always from the side to move's perspective.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Eval {
    Score(f64),
    Mate(u8),
    Draw,
}

impl Display for Eval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Eval::Score(score) => writeln!(f, "cp {}", score / 100.),
            Eval::Mate(plies) => {
                let sign = if *plies > 0 { '+' } else { '-' };
                writeln!(f, "{}M{}", sign, plies)
            }
            Eval::Draw => writeln!(f, "cp 0"),
        }
    }
}

impl PartialOrd for Eval {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Eval::Score(s1), Eval::Score(s2)) => s1.partial_cmp(s2),
            (Eval::Mate(m1), Eval::Mate(m2)) => match (*m1 % 2, *m2 % 2) {
                (1, 0) => Some(Ordering::Greater),
                (0, 1) => Some(Ordering::Less),
                (0, 0) => m1.partial_cmp(m2),
                (1, 1) => m2.partial_cmp(m1),
                (_, _) => unreachable!(),
            },
            (Eval::Draw, Eval::Draw) => Some(Ordering::Equal),

            (Eval::Score(s), Eval::Draw) => {
                if *s >= 0. {
                    Some(Ordering::Greater)
                } else {
                    Some(Ordering::Less)
                }
            }
            (Eval::Draw, Eval::Score(s)) => {
                if *s >= 0. {
                    Some(Ordering::Less)
                } else {
                    Some(Ordering::Greater)
                }
            }
            (Eval::Mate(m), Eval::Draw) => {
                if *m % 2 == 1 {
                    Some(Ordering::Greater)
                } else {
                    Some(Ordering::Less)
                }
            }
            (Eval::Draw, Eval::Mate(m)) => {
                if *m % 2 == 1 {
                    Some(Ordering::Less)
                } else {
                    Some(Ordering::Greater)
                }
            }

            (Eval::Mate(m), Eval::Score(_)) => {
                if *m % 2 == 1 {
                    Some(Ordering::Greater)
                } else {
                    Some(Ordering::Less)
                }
            }
            (Eval::Score(_), Eval::Mate(m)) => {
                if *m % 2 == 1 {
                    Some(Ordering::Less)
                } else {
                    Some(Ordering::Greater)
                }
            }
        }
    }
}

impl Eval {
    pub fn flip(&self) -> Eval {
        match self {
            Eval::Score(score) => Eval::Score(-score),
            Eval::Mate(mate) => Eval::Mate(mate + 1),
            Eval::Draw => Eval::Draw,
        }
    }
}

//impl Neg for Eval {
//    type Output = Self;
//    fn neg(self) -> Self::Output {
//        match self {
//            Eval::Score(score) => Eval::Score(-score),
//            Eval::Mate(mate) => Eval::Mate(-mate),
//            Eval::Draw => Eval::Draw,
//        }
//    }
//}

pub trait EvaluatePosition {
    fn evaluate(&self, position: &Position, move_gen: impl GenerateMoves) -> Eval;
}

#[derive(Clone, Copy)]
pub struct PositionEvaluator;

// Source: https://www.chessprogramming.org/Simplified_Evaluation_Function
#[inline(always)]
fn get_piece_square_bonus(piece: Piece, square: Square, is_early_or_mid_game: bool) -> f64 {
    #[rustfmt::skip]
    let table = match (piece, is_early_or_mid_game) {
        (Piece::Pawn, ..) => [
            0.,  0.,  0.,  0.,  0.,  0.,  0.,  0.,
             5., 10., 10.,-20.,-20., 10., 10.,  5.,
             5., -5.,-10.,  0.,  0.,-10., -5.,  5.,
             0.,  0.,  0., 20., 20.,  0.,  0.,  0.,
             5.,  5., 10., 25., 25., 10.,  5.,  5.,
            10., 10., 20., 30., 30., 20., 10., 10.,
            50., 50., 50., 50., 50., 50., 50., 50.,
             0.,  0.,  0.,  0.,  0.,  0.,  0.,  0.
        ],
        (Piece::King, true) => [
            -30.,-40.,-40.,-50.,-50.,-40.,-40.,-30.,
            -30.,-40.,-40.,-50.,-50.,-40.,-40.,-30.,
            -30.,-40.,-40.,-50.,-50.,-40.,-40.,-30.,
            -30.,-40.,-40.,-50.,-50.,-40.,-40.,-30.,
            -20.,-30.,-30.,-40.,-40.,-30.,-30.,-20.,
            -10.,-20.,-20.,-20.,-20.,-20.,-20.,-10.,
             20., 20.,  0.,  0.,  0.,  0., 20., 20.,
             20., 30., 10.,  0.,  0., 10., 30., 20.
        ],
        (Piece::King, false) => [
            -50.,-40.,-30.,-20.,-20.,-30.,-40.,-50.,
            -30.,-20.,-10.,  0.,  0.,-10.,-20.,-30.,
            -30.,-10., 20., 30., 30., 20.,-10.,-30.,
            -30.,-10., 30., 40., 40., 30.,-10.,-30.,
            -30.,-10., 30., 40., 40., 30.,-10.,-30.,
            -30.,-10., 20., 30., 30., 20.,-10.,-30.,
            -30.,-30.,  0.,  0.,  0.,  0.,-30.,-30.,
            -50.,-30.,-30.,-30.,-30.,-30.,-30.,-50.
        ],
        (Piece::Knight, ..) => [
            -50.,-40.,-30.,-30.,-30.,-30.,-40.,-50.,
            -40.,-20.,  0.,  0.,  0.,  0.,-20.,-40.,
            -30.,  0., 10., 15., 15., 10.,  0.,-30.,
            -30.,  5., 15., 20., 20., 15.,  5.,-30.,
            -30.,  0., 15., 20., 20., 15.,  0.,-30.,
            -30.,  5., 10., 15., 15., 10.,  5.,-30.,
            -40.,-20.,  0.,  5.,  5.,  0.,-20.,-40.,
            -50.,-40.,-30.,-30.,-30.,-30.,-40.,-50.
        ],
        (Piece::Bishop, ..) => [
            -20.,-10.,-10.,-10.,-10.,-10.,-10.,-20.,
            -10.,  0.,  0.,  0.,  0.,  0.,  0.,-10.,
            -10.,  0.,  5., 10., 10.,  5.,  0.,-10.,
            -10.,  5.,  5., 10., 10.,  5.,  5.,-10.,
            -10.,  0., 10., 10., 10., 10.,  0.,-10.,
            -10., 10., 10., 10., 10., 10., 10.,-10.,
            -10.,  5.,  0.,  0.,  0.,  0.,  5.,-10.,
            -20.,-10.,-10.,-10.,-10.,-10.,-10.,-20.
        ],
        (Piece::Rook, ..) => [
              0.,  0.,  0.,  0.,  0.,  0.,  0.,  0.,
              5., 10., 10., 10., 10., 10., 10.,  5.,
             -5.,  0.,  0.,  0.,  0.,  0.,  0., -5.,
             -5.,  0.,  0.,  0.,  0.,  0.,  0., -5.,
             -5.,  0.,  0.,  0.,  0.,  0.,  0., -5.,
             -5.,  0.,  0.,  0.,  0.,  0.,  0., -5.,
             -5.,  0.,  0.,  0.,  0.,  0.,  0., -5.,
              0.,  0.,  0.,  5.,  5.,  0.,  0.,  0.
        ],
        (Piece::Queen, ..) => [
            -20.,-10.,-10., -5., -5.,-10.,-10.,-20.,
            -10.,  0.,  0.,  0.,  0.,  0.,  0.,-10.,
            -10.,  0.,  5.,  5.,  5.,  5.,  0.,-10.,
             -5.,  0.,  5.,  5.,  5.,  5.,  0., -5.,
              0.,  0.,  5.,  5.,  5.,  5.,  0., -5.,
            -10.,  5.,  5.,  5.,  5.,  5.,  0.,-10.,
            -10.,  0.,  5.,  0.,  0.,  0.,  0.,-10.,
            -20.,-10.,-10., -5., -5.,-10.,-10.,-20.
        ],
    };
    table[square as usize]
}

impl EvaluatePosition for PositionEvaluator {
    fn evaluate(&self, position: &Position, move_gen: impl GenerateMoves) -> Eval {
        // Return evaluation relative to the side to move
        if position.state.half_move_clock == 50 {
            return Eval::Draw;
        }
        if move_gen.gen_moves(position).is_empty() && !move_gen.gen_checkers(position).is_empty() {
            if !move_gen.gen_checkers(position).is_empty() {
                return Eval::Mate(0);
            } else {
                return Eval::Draw;
            }
        }

        let eval_score = position
            .piece_locs()
            .fold(0., |acc, (piece, side, square)| {
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
        Eval::Score(eval_score)
    }
}

fn piece_value(piece: Piece) -> f64 {
    match piece {
        Piece::Pawn => 100.,
        Piece::Knight => 320.,
        Piece::Bishop => 330.,
        Piece::Rook => 500.,
        Piece::Queen => 900.,
        Piece::King => 2000., // Don't use f64::MAX in case of overflows
    }
}

pub static POSITION_EVALUATOR: PositionEvaluator = PositionEvaluator {};

#[cfg(test)]
mod tests {
    use crate::HYPERBOLA_QUINTESSENCE_MOVE_GEN;

    use super::*;

    use test_case::test_case;
    use testresult::TestResult;

    #[test_case(Eval::Score(10.), Eval::Score(-10.))]
    #[test_case(Eval::Mate(1), Eval::Mate(2))]
    #[test_case(Eval::Draw, Eval::Draw)]
    fn test_eval_flip(eval_input: Eval, eval_want: Eval) {
        let eval_got = eval_input.flip();

        assert_eq!(eval_got, eval_want);
    }

    #[test]
    fn test_eval_ord() {
        let evals_order_want = vec![
            Eval::Mate(0),
            Eval::Mate(2),
            Eval::Score(-2.0),
            Eval::Score(-0.5),
            Eval::Draw,
            Eval::Score(0.5),
            Eval::Score(2.0),
            Eval::Mate(3),
            Eval::Mate(1),
        ];
        let evals = vec![
            Eval::Mate(0),
            Eval::Mate(1),
            Eval::Mate(2),
            Eval::Mate(3),
            Eval::Score(-2.0),
            Eval::Score(-0.5),
            Eval::Draw,
            Eval::Score(0.5),
            Eval::Score(2.0),
        ];

        let mut sorted = evals.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        assert_eq!(sorted, evals_order_want);
    }

    #[test]
    fn test_obvious_eval() -> TestResult {
        let position = Position::from_fen("2k5/Q7/8/8/8/8/8/7K w - - 0 1")?;
        let move_gen = HYPERBOLA_QUINTESSENCE_MOVE_GEN;
        let eval = POSITION_EVALUATOR.evaluate(&position, move_gen);

        // Should be at least 5 pawns better than the opponent
        assert!(eval > Eval::Score(500.));

        Ok(())
    }
}

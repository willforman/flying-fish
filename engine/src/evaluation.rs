use std::collections::HashMap;
use std::fmt::Display;

use strum::IntoEnumIterator;

use crate::bitboard::Square;
use crate::position::{Piece, Position, Side};
use crate::GenerateMoves;

pub trait EvaluatePosition {
    fn evaluate(&self, position: &Position, move_gen: impl GenerateMoves) -> f64;
}

#[derive(Clone, Copy)]
pub struct PositionEvaluator;

// Source: https://www.chessprogramming.org/Simplified_Evaluation_Function
fn get_piece_square_bonus(
    piece: Piece,
    side: Side,
    square: Square,
    is_early_or_mid_game: bool,
) -> f64 {
    #[rustfmt::skip]
    let table = match (piece, side, is_early_or_mid_game) {
        (Piece::Pawn, Side::Black, ..) => [
            0.,  0.,  0.,  0.,  0.,  0.,  0.,  0.,
            50., 50., 50., 50., 50., 50., 50., 50.,
            10., 10., 20., 30., 30., 20., 10., 10.,
             5.,  5., 10., 25., 25., 10.,  5.,  5.,
             0.,  0.,  0., 20., 20.,  0.,  0.,  0.,
             5., -5.,-10.,  0.,  0.,-10., -5.,  5.,
             5., 10., 10.,-20.,-20., 10., 10.,  5.,
             0.,  0.,  0.,  0.,  0.,  0.,  0.,  0.
        ],
        (Piece::Pawn, Side::White, ..) => [
            0.,  0.,  0.,  0.,  0.,  0.,  0.,  0.,
             5., 10., 10.,-20.,-20., 10., 10.,  5.,
             5., -5.,-10.,  0.,  0.,-10., -5.,  5.,
             0.,  0.,  0., 20., 20.,  0.,  0.,  0.,
             5.,  5., 10., 25., 25., 10.,  5.,  5.,
            10., 10., 20., 30., 30., 20., 10., 10.,
            50., 50., 50., 50., 50., 50., 50., 50.,
             0.,  0.,  0.,  0.,  0.,  0.,  0.,  0.
        ],
        (Piece::King, Side::White, true) => [
             20., 30., 10.,  0.,  0., 10., 30., 20.,
             20., 20.,  0.,  0.,  0.,  0., 20., 20.,
            -10.,-20.,-20.,-20.,-20.,-20.,-20.,-10.,
            -20.,-30.,-30.,-40.,-40.,-30.,-30.,-20.,
            -30.,-40.,-40.,-50.,-50.,-40.,-40.,-30.,
            -30.,-40.,-40.,-50.,-50.,-40.,-40.,-30.,
            -30.,-40.,-40.,-50.,-50.,-40.,-40.,-30.,
            -30.,-40.,-40.,-50.,-50.,-40.,-40.,-30.,
        ],
        (Piece::King, Side::Black, true) => [
            -30.,-40.,-40.,-50.,-50.,-40.,-40.,-30.,
            -30.,-40.,-40.,-50.,-50.,-40.,-40.,-30.,
            -30.,-40.,-40.,-50.,-50.,-40.,-40.,-30.,
            -30.,-40.,-40.,-50.,-50.,-40.,-40.,-30.,
            -20.,-30.,-30.,-40.,-40.,-30.,-30.,-20.,
            -10.,-20.,-20.,-20.,-20.,-20.,-20.,-10.,
             20., 20.,  0.,  0.,  0.,  0., 20., 20.,
             20., 30., 10.,  0.,  0., 10., 30., 20.
        ],
        (Piece::King, .., false) => [
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
    fn evaluate(&self, position: &Position, move_gen: impl GenerateMoves) -> f64 {
        // Return evaluation relative to the side to move
        if position.state.half_move_clock == 50 {
            return 0.0;
        }
        if move_gen.gen_moves(position).is_empty() && !move_gen.gen_checkers(position).is_empty() {
            return f64::MIN;
        }

        let eval = position
            .get_piece_locs()
            .into_iter()
            .fold(0., |acc, (piece, side, square)| {
                let val = piece_value(piece) + get_piece_square_bonus(piece, side, square, true);

                if side == Side::White {
                    acc + val
                } else {
                    acc - val
                }
            });
        if position.state.to_move == Side::White {
            eval
        } else {
            -eval
        }
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

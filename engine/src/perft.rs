use std::{
    collections::HashMap,
    fmt::Display,
    marker::Copy,
    time::{Duration, Instant},
};

use tabled::{Table, Tabled};

use crate::move_gen::{GenerateMoves, HYPERBOLA_QUINTESSENCE_MOVE_GEN};
use crate::position::{Piece, Position};
use crate::{bitboard::BitBoard, move_gen, Move};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Tabled)]
pub struct PerftDepthResult {
    tot: u64,
    captures: u64,
    en_passants: u64,
    castles: u64,
    promotions: u64,
    checks: u64,
    discovery_checks: u64,
    double_checks: u64,
    checkmates: u64,
}

pub struct PerftResult {
    pub depth_results: Vec<PerftDepthResult>,
    pub tot_nodes: u64,
    pub time_elapsed: Duration,
    pub nodes_per_second: f64,
}

impl PerftDepthResult {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tot: u64,
        captures: u64,
        en_passants: u64,
        castles: u64,
        promotions: u64,
        checks: u64,
        discovery_checks: u64,
        double_checks: u64,
        checkmates: u64,
    ) -> Self {
        PerftDepthResult {
            tot,
            captures,
            en_passants,
            castles,
            promotions,
            checks,
            discovery_checks,
            double_checks,
            checkmates,
        }
    }
    pub fn empty() -> PerftDepthResult {
        PerftDepthResult::new(0, 0, 0, 0, 0, 0, 0, 0, 0)
    }
}

impl Display for PerftResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "total nodes: {}", self.tot_nodes)?;
        writeln!(f, "time elapsed: {}", self.time_elapsed.as_secs_f32())?;
        writeln!(f, "nodes/s: {}", self.nodes_per_second)?;
        writeln!(f, "{}", Table::new(&self.depth_results).to_string())?;
        Ok(())
    }
}

pub fn perft(
    position: &Position,
    depth: usize,
    move_gen: impl GenerateMoves + Copy,
) -> (HashMap<Move, usize>, usize) {
    let moves = move_gen.gen_moves(position);
    let mut perft_results: HashMap<Move, usize> = HashMap::with_capacity(moves.len());

    for mve in moves {
        let mut move_pos = position.clone();
        move_pos.make_move(&mve).unwrap();

        let moves_count = perft_helper(&move_pos, 1, depth, move_gen);
        perft_results.insert(mve, moves_count);
    }

    let tot_moves = perft_results.values().sum();

    (perft_results, tot_moves)
}

fn perft_helper(
    position: &Position,
    curr_depth: usize,
    max_depth: usize,
    move_gen: impl GenerateMoves + Copy,
) -> usize {
    if curr_depth == max_depth {
        return 1;
    }

    let mut moves_count = 0;
    let moves = move_gen.gen_moves(position);
    for mve in moves {
        let mut move_pos = position.clone();
        move_pos.make_move(&mve).unwrap();

        let curr_move_moves_count = perft_helper(&move_pos, curr_depth + 1, max_depth, move_gen);
        moves_count += curr_move_moves_count;
    }
    moves_count
}

pub fn perft_full(
    position: &Position,
    depth: usize,
    move_gen: impl GenerateMoves + Copy,
) -> PerftResult {
    let mut depth_results = vec![PerftDepthResult::empty(); depth];

    let start = Instant::now();

    perft_full_helper(&mut depth_results, position, depth, 0, move_gen);

    let time_elapsed = start.elapsed();

    let tot_nodes = depth_results.iter().fold(0, |tot, curr| tot + curr.tot);

    let nodes_per_second = tot_nodes as f64 / time_elapsed.as_secs_f64();

    PerftResult {
        depth_results,
        tot_nodes,
        time_elapsed,
        nodes_per_second,
    }
}

fn perft_full_helper(
    depth_results: &mut Vec<PerftDepthResult>,
    position: &Position,
    max_depth: usize,
    curr_depth: usize,
    move_gen: impl GenerateMoves + Copy,
) {
    // Must check moves before checking end condition of this recursive function
    // because we need to check for checkmate
    let moves = move_gen.gen_moves(position);

    if moves.is_empty() {
        let prev_res = depth_results.get_mut(curr_depth - 1).unwrap();
        prev_res.checkmates += 1;
        return;
    }

    if curr_depth == max_depth {
        return;
    }

    let curr_res = depth_results.get_mut(curr_depth).unwrap();

    let side = position.state.to_move;
    let opp_pieces = position.sides.get(side.opposite_side());

    curr_res.tot += u64::try_from(moves.len()).unwrap();

    if let Some(ep_target) = position.state.en_passant_target {
        for mve in moves.clone() {
            if mve.dest == ep_target {
                let (piece_type, _) = position.is_piece_at(mve.src).unwrap();
                if piece_type == Piece::Pawn {
                    curr_res.captures += 1;
                }
            } else if !(BitBoard::from_square(mve.dest) & opp_pieces).is_empty() {
                curr_res.captures += 1;
            }
        }
    } else {
        for mve in moves.clone() {
            if !(BitBoard::from_square(mve.dest) & opp_pieces).is_empty() {
                curr_res.captures += 1;
            }
        }
    }

    if let Some(ep_target) = position.state.en_passant_target {
        for pawn_square in position.pieces.get(Piece::Pawn).get(side).to_squares() {
            let has_en_passant = moves
                .iter()
                .any(|&mve| mve.src == pawn_square && mve.dest == ep_target);
            if has_en_passant {
                curr_res.en_passants += 1;
            }
        }
    }

    let castles: u64 = moves
        .iter()
        .filter(|&mve| {
            let (p, _) = position.is_piece_at(mve.src).unwrap();
            if p == Piece::King {
                mve.src.abs_diff(mve.dest) == 2
            } else {
                false
            }
        })
        .count()
        .try_into()
        .unwrap();
    curr_res.castles += castles;

    let promotions: u64 = moves
        .iter()
        .filter(|&mve| mve.promotion.is_some())
        .count()
        .try_into()
        .unwrap();

    curr_res.promotions += promotions;

    let mut tot_checks = 0;
    let mut tot_double_checks = 0;
    let mut tot_discovery_checks = 0;

    for mve in moves {
        let mut move_position = position.clone();
        move_position.make_move(&mve).unwrap();

        let mut checkers = move_gen.gen_checkers(&move_position);
        if !checkers.is_empty() {
            tot_checks += 1;
            if checkers.num_squares_set() > 1 {
                tot_double_checks += 1;
            } else {
                checkers.clear_square(mve.dest);
                if !checkers.is_empty() {
                    tot_discovery_checks += 1;
                }
            }
        }

        perft_full_helper(
            depth_results,
            &move_position,
            max_depth,
            curr_depth + 1,
            move_gen,
        );
    }

    // Reborrow to avoid multiple mutable references
    let curr_res = depth_results.get_mut(curr_depth).unwrap();
    curr_res.checks += tot_checks;
    curr_res.double_checks += tot_double_checks;
    curr_res.discovery_checks += tot_discovery_checks;
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrayvec::ArrayVec;
    use test_case::test_case;

    use crate::bitboard::Square::*;
    use crate::position::Move;

    #[derive(Clone, Copy)]
    struct MoveGenStub<'a> {
        moves: &'a [Move],
    }

    impl GenerateMoves for MoveGenStub<'_> {
        fn gen_moves(&self, _position: &Position) -> ArrayVec<Move, 80> {
            ArrayVec::from_iter(self.moves.iter().cloned())
        }

        fn gen_checkers(&self, _position: &Position) -> BitBoard {
            BitBoard::empty()
        }
    }

    #[test_case(Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/Pp2P3/2N2Q1p/1PPBBPPP/R3K2R b KQkq a3 0 1").unwrap(), 1)]
    fn test_count_en_passant(start_position: Position, want: u64) {
        let move_gen = MoveGenStub {
            moves: &[Move::new(B4, A3)],
        };

        let res = perft_full(&start_position, 1, move_gen);
        assert_eq!(res.depth_results[0].en_passants, want);
    }
}

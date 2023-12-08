use std::{time::{Duration, Instant}, fmt::Display};

use tabled::{Tabled,Table};

use crate::{position::{Position, Piece,Side}, move_gen::GenerateAllMoves};
use crate::bitboard::BitBoard;
use crate::bitboard::Square::*;

#[derive(Clone,Copy,Debug,PartialEq, Eq, Tabled)]
pub struct PerftDepthResult {
    tot: u64,
    captures: u64,
    en_passants: u64,
    castles: u64,
    promotions: u64,
    checks: u64,
    discovery_checks: u64,
    double_checks: u64,
    checkmates: u64
}

pub struct PerftResult {
    pub depth_results: Vec<PerftDepthResult>,
    pub tot_nodes: u64,
    pub time_elapsed: Duration,
    pub nodes_per_second: f64,
}

impl PerftDepthResult {
    pub fn new(
        tot: u64, 
        captures: u64, 
        en_passants: u64, 
        castles: u64, 
        promotions: u64, 
        checks: u64, 
        discovery_checks: u64, 
        double_checks: u64, 
        checkmates: u64
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
            checkmates
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

pub fn perft(position: &Position, move_gen: &impl GenerateAllMoves, depth: usize) -> PerftResult {
    let mut depth_results = vec![PerftDepthResult::empty(); depth];

    let start = Instant::now();

    perft_helper(&mut depth_results, position, move_gen, depth, 0);

    let time_elapsed = start.elapsed();

    let tot_nodes = depth_results.iter()
        .fold(0, |tot, curr| tot + curr.tot);

    let nodes_per_second = tot_nodes as f64 / time_elapsed.as_secs_f64();

    PerftResult { 
        depth_results, 
        tot_nodes,
        time_elapsed,
        nodes_per_second
    }
}

fn perft_helper(depth_results: &mut Vec<PerftDepthResult>, position: &Position, move_gen: &impl GenerateAllMoves, max_depth: usize, curr_depth: usize) {
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

    let captures = moves.iter()
        .filter(|&mve| !(BitBoard::from_square(mve.dest) & opp_pieces).is_empty())
        .count();
    curr_res.captures += u64::try_from(captures).unwrap();

    if let Some(ep_target) = position.state.en_passant_target {
        for pawn_square in position.pieces.get(Piece::Pawn).get(side).to_squares() {
            let has_en_passant = moves.iter()
                .any(|&mve| mve.src == pawn_square && mve.dest == ep_target);
            if has_en_passant {
                curr_res.en_passants += 1;
            }
        }
    }

    let castles: u64 = moves.iter()
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

    let promotions: u64 = moves.iter()
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

        let mut checkers = move_gen.get_checkers(&move_position);
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

        perft_helper(depth_results, &move_position, move_gen, max_depth, curr_depth + 1);
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
    use std::collections::HashSet;

    use test_case::test_case;

    use crate::position::Move;
    
    struct AllPiecesMoveGenStub {
        moves: HashSet<Move>
    }

    impl GenerateAllMoves for AllPiecesMoveGenStub {
        fn gen_moves(&self, _position: &Position) -> HashSet<Move> {
            self.moves.clone()
        }

        fn get_checkers(&self, _position: &Position) -> BitBoard {
            BitBoard::empty()
        }
    }

    #[test_case(Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/Pp2P3/2N2Q1p/1PPBBPPP/R3K2R b KQkq a3 0 1").unwrap(), 1)]
    fn test_count_en_passant(start_position: Position, want: u64) {
        let move_gen = AllPiecesMoveGenStub{
            moves: HashSet::from([
                Move::new(B4, A3)
            ])
        };

        let res = perft(&start_position, &move_gen, 1);
        assert_eq!(res.depth_results[0].en_passants, want);
    }
}

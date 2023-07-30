use std::{time::{Duration, Instant}, fmt::Display};

use tabled::{Tabled,Table};

use crate::{position::Position, move_gen::{AllPiecesMoveGen, MoveCounts}};

pub struct PerftResult {
    pub depth_results: Vec<MoveCounts>,
    pub tot_nodes: u64,
    pub time_elapsed: Duration,
    pub nodes_per_second: f64,
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

pub fn perft(position: &Position, move_gen: &AllPiecesMoveGen, depth: usize) -> PerftResult {
    let mut depth_results = vec![MoveCounts::empty(); depth];

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

fn perft_helper(move_counts: &mut Vec<MoveCounts>, position: &Position, move_gen: &AllPiecesMoveGen, max_depth: usize, curr_depth: usize) {
    if curr_depth == max_depth {
        return;
    }

    let curr_counts = move_counts.get_mut(curr_depth).unwrap();

    let moves = move_gen.gen_moves(position, curr_counts);

    for mve in moves {
        let mut move_position = position.clone();
        move_position.make_move(mve).unwrap();

        perft_helper(move_counts, &move_position, move_gen, max_depth, curr_depth + 1);
    }
}

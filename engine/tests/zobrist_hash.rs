use engine::{GenerateMoves, MOVE_GEN, Position, ZobristHash};
use test_case::test_case;

#[test_case(Position::start(), 4)]
#[test_case(Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 0").unwrap(), 3)]
#[test_case(Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap(), 5)]
fn test_zobrist_hash_perft(mut position: Position, max_depth: usize) {
    zobrist_hash_perft_helper(&mut position, 0, max_depth, MOVE_GEN);
}

#[test_case(Position::start(), 5)]
#[test_case(Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 0").unwrap(), 4)]
#[test_case(Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap(), 6)]
#[ignore]
fn test_zobrist_hash_perft_long(mut position: Position, max_depth: usize) {
    zobrist_hash_perft_helper(&mut position, 0, max_depth, MOVE_GEN);
}

fn zobrist_hash_perft_helper(
    position: &mut Position,
    curr_depth: usize,
    max_depth: usize,
    move_gen: impl GenerateMoves + Copy,
) {
    if curr_depth == max_depth {
        return;
    }

    let moves = move_gen.gen_moves(position);
    for mve in moves {
        let before_hash = position.zobrist_hash;
        let unmake_move_state = position.make_move(mve);

        zobrist_hash_perft_helper(position, curr_depth + 1, max_depth, move_gen);

        // Ensure incremental hash is the same as hash generated from scratch.
        let full_gen_hash = ZobristHash::calculate(&position.pieces, &position.state);
        assert_eq!(
            full_gen_hash,
            position.zobrist_hash,
            "Incremental hash not equal for move: {:?}, fen=`{}`",
            mve,
            position.to_fen()
        );
        position.unmake_move(unmake_move_state);
        assert_eq!(
            position.zobrist_hash,
            before_hash,
            "Couldn't reverse hash for move: {:?}, fen=`{}`",
            mve,
            position.to_fen()
        );
    }
}

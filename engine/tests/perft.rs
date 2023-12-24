use engine::position::Position;
use engine::perft::{PerftDepthResult,perft};
use engine::move_gen::AllPiecesMoveGen;
use engine::move_gen::leaping_pieces::LeapingPiecesMoveGen;
use engine::move_gen::hyperbola_quintessence::HyperbolaQuintessence;

use test_case::test_case;

#[test_case(Position::start(), 6, PerftDepthResult::new(
    2_439_530_234_167,
    125_208_536_153,
    319_496_827,
    1_784_356_000,
    17_334_376,
    36_095_901_903,
    37_101_713,
    5_547_231,
    400_191_963,
    ) ; "starting 6"
)]
#[test_case(Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap(), 4, PerftDepthResult::new(
    4085603,
    757163,
    1929,
    128013,
    15172,
    25523,
    42,
    6,
    43 
    ) ; "kiwipete 4"
)]
#[test_case(Position::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1").unwrap(), 4, PerftDepthResult::new(
    422333,
    131393,
    0,
    7795,
    600032,
    15492,
    0,
    0,
    5
    ) ; "perft results position4 4"
)]
#[test_case(Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap(), 5, PerftDepthResult::new(
    674624,
    52051,
    1165,
    0,
    0,
    52950,
    1292,
    3,
    0) ; "perft results position3 5"
)]
#[ignore]
fn test_perft(starting_position: Position, depth: usize, want: PerftDepthResult) {
    let leaping_pieces = Box::new(LeapingPiecesMoveGen::new());
    let sliding_pieces = Box::new(HyperbolaQuintessence::new());
    let move_gen = AllPiecesMoveGen::new(leaping_pieces, sliding_pieces);

    let res = perft(&starting_position, &move_gen, depth);
    println!("{}", res);

    assert_eq!(res.depth_results.len(), depth.into());
    assert_eq!(res.depth_results.last().unwrap(), &want);
}

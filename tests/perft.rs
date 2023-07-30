use chess::position::Position;
use chess::perft::perft;
use chess::move_gen::{AllPiecesMoveGen, MoveCounts};
use chess::move_gen::leaping_pieces::LeapingPiecesMoveGen;
use chess::move_gen::hyperbola_quintessence::HyperbolaQuintessence;

use test_case::test_case;

#[test_case(Position::start(), 6, MoveCounts::new(
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
#[test_case(Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap(), 4, MoveCounts::new(
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
#[ignore]
fn test_perft(starting_position: Position, depth: usize, want: MoveCounts) {
    let leaping_pieces = Box::new(LeapingPiecesMoveGen::new());
    let sliding_pieces = Box::new(HyperbolaQuintessence::new());
    let move_gen = AllPiecesMoveGen::new(leaping_pieces, sliding_pieces);

    let res = perft(&starting_position, &move_gen, depth);
    println!("{}", res);

    assert_eq!(res.depth_results.len(), depth.into());
    assert_eq!(res.depth_results.last().unwrap(), &want);
}

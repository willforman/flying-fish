use std::collections::{HashMap, HashSet};

use engine::Square::*;
use engine::{
    perft, perft_full, Move, PerftDepthResult, Position, HYPERBOLA_QUINTESSENCE_MOVE_GEN,
};

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
#[test_case(Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap(), 3, PerftDepthResult::new(
    97862,
    17102,
    45,
    3162,
    0,
    993,
    0,
    0,
    1,
    ) ; "kiwipete 3"
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
fn test_perft_full(starting_position: Position, depth: usize, want: PerftDepthResult) {
    let res = perft_full(&starting_position, depth, HYPERBOLA_QUINTESSENCE_MOVE_GEN);
    println!("{}", res);

    assert_eq!(res.depth_results.len(), depth);
    assert_eq!(res.depth_results.last().unwrap(), &want);
}

macro_rules! assert_eq_maps {
    ($map_a:expr, $map_b:expr) => {
        let diff_a_b: HashMap<_, _> = $map_a
            .iter()
            .filter(|(ka, _)| !$map_b.contains_key(ka))
            .collect();
        let diff_b_a: HashMap<_, _> = $map_b
            .iter()
            .filter(|(kb, _)| !$map_a.contains_key(kb))
            .collect();

        let same_values: HashMap<_, _> = $map_a
            .iter()
            .filter(|(key_a, _)| $map_b.contains_key(key_a))
            .collect();

        let diff_values: HashMap<_, (_, _)> = $map_a
            .iter()
            .filter(|(key_a, _)| $map_b.contains_key(key_a))
            .map(|(key_a, val_a)| (key_a, val_a, $map_b.get(key_a).unwrap()))
            .filter(|(_, val_a, val_b)| val_a != val_b)
            .map(|(key, val_a, val_b)| (key, (val_a, val_b)))
            .collect();

        if !diff_a_b.is_empty() || !diff_b_a.is_empty() || !diff_values.is_empty() {
            panic!(
                "maps aren't equal. \
                   \nin {} but not {}: {:?}. \
                   \nin {} but not {}: {:?}. \
                   \nhave same values: {:?}. \
                   \nhave differing values: {:?}.",
                stringify!($map_a),
                stringify!($map_b),
                diff_a_b,
                stringify!($map_a),
                stringify!($map_b),
                diff_b_a,
                same_values,
                diff_values
            );
        }
    };
}

#[test_case(Position::start(), 2, 400, HashMap::from([
    (Move::new(A2, A3), 20),
    (Move::new(B2, B3), 20),
    (Move::new(C2, C3), 20),
    (Move::new(D2, D3), 20),
    (Move::new(E2, E3), 20),
    (Move::new(F2, F3), 20),
    (Move::new(G2, G3), 20),
    (Move::new(H2, H3), 20),
    (Move::new(A2, A4), 20),
    (Move::new(B2, B4), 20),
    (Move::new(C2, C4), 20),
    (Move::new(D2, D4), 20),
    (Move::new(E2, E4), 20),
    (Move::new(F2, F4), 20),
    (Move::new(G2, G4), 20),
    (Move::new(H2, H4), 20),
    (Move::new(B1, A3), 20),
    (Move::new(B1, C3), 20),
    (Move::new(G1, F3), 20),
    (Move::new(G1, H3), 20),
]) ; "starting position 2")]
#[test_case(Position::start(), 3, 8902, HashMap::from([
    (Move::new(A2, A3), 380),
    (Move::new(B2, B3), 420),
    (Move::new(C2, C3), 420),
    (Move::new(D2, D3), 539),
    (Move::new(E2, E3), 599),
    (Move::new(F2, F3), 380),
    (Move::new(G2, G3), 420),
    (Move::new(H2, H3), 380),
    (Move::new(A2, A4), 420),
    (Move::new(B2, B4), 421),
    (Move::new(C2, C4), 441),
    (Move::new(D2, D4), 560),
    (Move::new(E2, E4), 600),
    (Move::new(F2, F4), 401),
    (Move::new(G2, G4), 421),
    (Move::new(H2, H4), 420),
    (Move::new(B1, A3), 400),
    (Move::new(B1, C3), 440),
    (Move::new(G1, F3), 440),
    (Move::new(G1, H3), 400),
]) ; "starting position 3")]
#[test_case(Position::start(), 4, 197281, HashMap::from([
    (Move::new(A2, A3), 8457),
    (Move::new(B2, B3), 9345),
    (Move::new(C2, C3), 9272),
    (Move::new(D2, D3), 11959),
    (Move::new(E2, E3), 13134),
    (Move::new(F2, F3), 8457),
    (Move::new(G2, G3), 9345),
    (Move::new(H2, H3), 8457),
    (Move::new(A2, A4), 9329),
    (Move::new(B2, B4), 9332),
    (Move::new(C2, C4), 9744),
    (Move::new(D2, D4), 12435),
    (Move::new(E2, E4), 13160),
    (Move::new(F2, F4), 8929),
    (Move::new(G2, G4), 9328),
    (Move::new(H2, H4), 9329),
    (Move::new(B1, A3), 8885),
    (Move::new(B1, C3), 9755),
    (Move::new(G1, F3), 9748),
    (Move::new(G1, H3), 8881),
]) ; "starting position 4")]
#[test_case(Position::start(), 5, 4865609, HashMap::from([
    (Move::new(A2, A3), 181046),
    (Move::new(B2, B3), 215255),
    (Move::new(C2, C3), 222861),
    (Move::new(D2, D3), 328511),
    (Move::new(E2, E3), 402988),
    (Move::new(F2, F3), 178889),
    (Move::new(G2, G3), 217210),
    (Move::new(H2, H3), 181044),
    (Move::new(A2, A4), 217832),
    (Move::new(B2, B4), 216145),
    (Move::new(C2, C4), 240082),
    (Move::new(D2, D4), 361790),
    (Move::new(E2, E4), 405385),
    (Move::new(F2, F4), 198473),
    (Move::new(G2, G4), 214048),
    (Move::new(H2, H4), 218829),
    (Move::new(B1, A3), 198572),
    (Move::new(B1, C3), 234656),
    (Move::new(G1, F3), 233491),
    (Move::new(G1, H3), 198502),
]) ; "starting position 5")]
#[test_case(Position::start(), 6, 119060324, HashMap::from([
    (Move::new(A2, A3), 4463267),
    (Move::new(B2, B3), 5310358),
    (Move::new(C2, C3), 5417640),
    (Move::new(D2, D3), 8073082),
    (Move::new(E2, E3), 9726018),
    (Move::new(F2, F3), 4404141),
    (Move::new(G2, G3), 5346260),
    (Move::new(H2, H3), 4463070),
    (Move::new(A2, A4), 5363555),
    (Move::new(B2, B4), 5293555),
    (Move::new(C2, C4), 5866666),
    (Move::new(D2, D4), 8879566),
    (Move::new(E2, E4), 9771632),
    (Move::new(F2, F4), 4890429),
    (Move::new(G2, G4), 5239875),
    (Move::new(H2, H4), 5385554),
    (Move::new(B1, A3), 4856835),
    (Move::new(B1, C3), 5708064),
    (Move::new(G1, F3), 5723523),
    (Move::new(G1, H3), 4877234),
]) ; "starting position 6")]
fn test_perft(
    starting_position: Position,
    depth: usize,
    tot_moves_want: usize,
    moves_want: HashMap<Move, usize>,
) {
    let (moves_got, tot_moves_got) =
        perft(&starting_position, depth, HYPERBOLA_QUINTESSENCE_MOVE_GEN);

    assert_eq_maps!(moves_got, moves_want);
    assert_eq!(tot_moves_got, tot_moves_want);
}

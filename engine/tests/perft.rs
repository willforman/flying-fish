use std::collections::{HashMap, HashSet};

use engine::Square::*;
use engine::{MOVE_GEN, Move, PerftDepthResult, Position, perft, perft_full};

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
    60032,
    15492,
    19,
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
    let res = perft_full(&starting_position, depth, MOVE_GEN);
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
            .map(|(key_a, val_a)| (key_a, val_a, $map_b.get(key_a).unwrap()))
            .filter(|(_, val_a, val_b)| val_a == val_b)
            .map(|(key, val_a, _)| (key, val_a))
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
                   \nhave differing values: {:?}. \
                   \nhave same values: {:?}.",
                stringify!($map_a),
                stringify!($map_b),
                diff_a_b,
                stringify!($map_b),
                stringify!($map_a),
                diff_b_a,
                diff_values,
                same_values,
            );
        }
    };
}

#[test_case(Position::start(), &[], 2, 400, HashMap::from([
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
#[test_case(Position::start(), &[], 3, 8902, HashMap::from([
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
#[test_case(Position::start(), &[], 4, 197281, HashMap::from([
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
#[test_case(Position::start(), &[], 5, 4865609, HashMap::from([
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
#[test_case(Position::start(), &[], 6, 119060324, HashMap::from([
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
#[test_case(Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap(), &[], 3, 97862, HashMap::from([
    (Move::new(A2, A3), 2186),
    (Move::new(B2, B3), 1964),
    (Move::new(G2, G3), 1882),
    (Move::new(D5, D6), 1991),
    (Move::new(A2, A4), 2149),
    (Move::new(G2, G4), 1843),
    (Move::new(G2, H3), 1970),
    (Move::new(D5, E6), 2241),
    (Move::new(C3, B1), 2038),
    (Move::new(C3, D1), 2040),
    (Move::new(C3, A4), 2203),
    (Move::new(C3, B5), 2138),
    (Move::new(E5, D3), 1803),
    (Move::new(E5, C4), 1880),
    (Move::new(E5, G4), 1878),
    (Move::new(E5, C6), 2027),
    (Move::new(E5, G6), 1997),
    (Move::new(E5, D7), 2124),
    (Move::new(E5, F7), 2080),
    (Move::new(D2, C1), 1963),
    (Move::new(D2, E3), 2136),
    (Move::new(D2, F4), 2000),
    (Move::new(D2, G5), 2134),
    (Move::new(D2, H6), 2019),
    (Move::new(E2, D1), 1733),
    (Move::new(E2, F1), 2060),
    (Move::new(E2, D3), 2050),
    (Move::new(E2, C4), 2082),
    (Move::new(E2, B5), 2057),
    (Move::new(E2, A6), 1907),
    (Move::new(A1, B1), 1969),
    (Move::new(A1, C1), 1968),
    (Move::new(A1, D1), 1885),
    (Move::new(H1, F1), 1929),
    (Move::new(H1, G1), 2013),
    (Move::new(F3, D3), 2005),
    (Move::new(F3, E3), 2174),
    (Move::new(F3, G3), 2214),
    (Move::new(F3, H3), 2360),
    (Move::new(F3, F4), 2132),
    (Move::new(F3, G4), 2169),
    (Move::new(F3, F5), 2396),
    (Move::new(F3, H5), 2267),
    (Move::new(F3, F6), 2111),
    (Move::new(E1, D1), 1894),
    (Move::new(E1, F1), 1855),
    (Move::new(E1, G1), 2059),
    (Move::new(E1, C1), 1887),
]) ; "kiwipete 3")]
#[test_case(Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap(), &[], 4, 4085603, HashMap::from([
    (Move::new(A2, A3), 94405),
    (Move::new(B2, B3), 81066),
    (Move::new(G2, G3), 77468),
    (Move::new(D5, D6), 79551),
    (Move::new(A2, A4), 90978),
    (Move::new(G2, G4), 75677),
    (Move::new(G2, H3), 82759),
    (Move::new(D5, E6), 97464),
    (Move::new(C3, B1), 84773),
    (Move::new(C3, D1), 84782),
    (Move::new(C3, A4), 91447),
    (Move::new(C3, B5), 81498),
    (Move::new(E5, D3), 77431),
    (Move::new(E5, C4), 77752),
    (Move::new(E5, G4), 79912),
    (Move::new(E5, C6), 83885),
    (Move::new(E5, G6), 83866),
    (Move::new(E5, D7), 93913),
    (Move::new(E5, F7), 88799),
    (Move::new(D2, C1), 83037),
    (Move::new(D2, E3), 90274),
    (Move::new(D2, F4), 84869),
    (Move::new(D2, G5), 87951),
    (Move::new(D2, H6), 82323),
    (Move::new(E2, D1), 74963),
    (Move::new(E2, F1), 88728),
    (Move::new(E2, D3), 85119),
    (Move::new(E2, C4), 84835),
    (Move::new(E2, B5), 79739),
    (Move::new(E2, A6), 69334),
    (Move::new(A1, B1), 83348),
    (Move::new(A1, C1), 83263),
    (Move::new(A1, D1), 79695),
    (Move::new(H1, F1), 81563),
    (Move::new(H1, G1), 84876),
    (Move::new(F3, D3), 83727),
    (Move::new(F3, E3), 92505),
    (Move::new(F3, G3), 94461),
    (Move::new(F3, H3), 98524),
    (Move::new(F3, F4), 90488),
    (Move::new(F3, G4), 92037),
    (Move::new(F3, F5), 104992),
    (Move::new(F3, H5), 95034),
    (Move::new(F3, F6), 77838),
    (Move::new(E1, D1), 79989),
    (Move::new(E1, F1), 77887),
    (Move::new(E1, G1), 86975),
    (Move::new(E1, C1), 79803),
]) ; "kiwipete 4")]
#[test_case(Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap(), &[Move::new(E1, G1)], 2, 2059, HashMap::from([
    (Move::new(B4, B3), 49),
    (Move::new(G6, G5), 47),
    (Move::new(C7, C6), 49),
    (Move::new(D7, D6), 47),
    (Move::new(C7, C5), 49),
    (Move::new(H3, G2), 48),
    (Move::new(E6, D5), 48),
    (Move::new(B4, C3), 48),
    (Move::new(B6, A4), 47),
    (Move::new(B6, C4), 46),
    (Move::new(B6, D5), 48),
    (Move::new(B6, C8), 48),
    (Move::new(F6, E4), 51),
    (Move::new(F6, G4), 47),
    (Move::new(F6, D5), 49),
    (Move::new(F6, H5), 49),
    (Move::new(F6, H7), 49),
    (Move::new(F6, G8), 49),
    (Move::new(A6, E2), 45),
    (Move::new(A6, D3), 46),
    (Move::new(A6, C4), 46),
    (Move::new(A6, B5), 47),
    (Move::new(A6, B7), 48),
    (Move::new(A6, C8), 48),
    (Move::new(G7, H6), 48),
    (Move::new(G7, F8), 48),
    (Move::new(A8, B8), 48),
    (Move::new(A8, C8), 48),
    (Move::new(A8, D8), 48),
    (Move::new(H8, H4), 48),
    (Move::new(H8, H5), 48),
    (Move::new(H8, H6), 48),
    (Move::new(H8, H7), 48),
    (Move::new(H8, F8), 48),
    (Move::new(H8, G8), 48),
    (Move::new(E7, C5), 48),
    (Move::new(E7, D6), 47),
    (Move::new(E7, D8), 48),
    (Move::new(E7, F8), 48),
    (Move::new(E8, D8), 48),
    (Move::new(E8, F8), 48),
    (Move::new(E8, G8), 48),
    (Move::new(E8, C8), 48),
]) ; "kiwipete castle kingside 2")]
#[ignore]
fn test_perft(
    mut starting_position: Position,
    start_moves: &[Move],
    depth: usize,
    tot_moves_want: usize,
    moves_want: HashMap<Move, usize>,
) {
    for mve in start_moves {
        starting_position.make_move(*mve);
    }

    let (moves_got, tot_moves_got) = perft(&starting_position, depth, MOVE_GEN);

    assert_eq_maps!(moves_got, moves_want);
    assert_eq!(tot_moves_got, tot_moves_want);
}

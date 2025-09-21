use std::collections::{HashMap, HashSet};

use engine::Square::*;
use engine::{MOVE_GEN, Move, PerftDepthResult, Position, perft, perft_full};

use test_case::test_case;

// #[test_case(Position::start(), 6, PerftDepthResult::new(
//     2_439_530_234_167,
//     125_208_536_153,
//     319_496_827,
//     1_784_356_000,
//     17_334_376,
//     36_095_901_903,
//     37_101_713,
//     5_547_231,
//     400_191_963,
//     ) ; "starting 6"
// )]
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
// #[test_case(Position::start(), &[], 6, 119060324, HashMap::from([
//     (Move::new(A2, A3), 4463267),
//     (Move::new(B2, B3), 5310358),
//     (Move::new(C2, C3), 5417640),
//     (Move::new(D2, D3), 8073082),
//     (Move::new(E2, E3), 9726018),
//     (Move::new(F2, F3), 4404141),
//     (Move::new(G2, G3), 5346260),
//     (Move::new(H2, H3), 4463070),
//     (Move::new(A2, A4), 5363555),
//     (Move::new(B2, B4), 5293555),
//     (Move::new(C2, C4), 5866666),
//     (Move::new(D2, D4), 8879566),
//     (Move::new(E2, E4), 9771632),
//     (Move::new(F2, F4), 4890429),
//     (Move::new(G2, G4), 5239875),
//     (Move::new(H2, H4), 5385554),
//     (Move::new(B1, A3), 4856835),
//     (Move::new(B1, C3), 5708064),
//     (Move::new(G1, F3), 5723523),
//     (Move::new(G1, H3), 4877234),
// ]) ; "starting position 6")]
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

// Ethereal perft test cases: https://github.com/AndyGrant/Ethereal/blob/master/src/perft/standard.epd
#[test_case(Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap(), HashMap::from([(1, 48), (2, 2039), (3, 97862), (4, 4085603), (5, 193690690)]) ; "ethereal 2")]
#[test_case(Position::from_fen("4k3/8/8/8/8/8/8/4K2R w K - 0 1").unwrap(), HashMap::from([(1, 15), (2, 66), (3, 1197), (4, 7059), (5, 133987), (6, 764643)]) ; "ethereal 3")]
#[test_case(Position::from_fen("4k3/8/8/8/8/8/8/R3K3 w Q - 0 1").unwrap(), HashMap::from([(1, 16), (2, 71), (3, 1287), (4, 7626), (5, 145232), (6, 846648)]) ; "ethereal 4")]
#[test_case(Position::from_fen("4k2r/8/8/8/8/8/8/4K3 w k - 0 1").unwrap(), HashMap::from([(1, 5), (2, 75), (3, 459), (4, 8290), (5, 47635), (6, 899442)]) ; "ethereal 5")]
#[test_case(Position::from_fen("r3k3/8/8/8/8/8/8/4K3 w q - 0 1").unwrap(), HashMap::from([(1, 5), (2, 80), (3, 493), (4, 8897), (5, 52710), (6, 1001523)]) ; "ethereal 6")]
#[test_case(Position::from_fen("4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1").unwrap(), HashMap::from([(1, 26), (2, 112), (3, 3189), (4, 17945), (5, 532933), (6, 2788982)]) ; "ethereal 7")]
#[test_case(Position::from_fen("r3k2r/8/8/8/8/8/8/4K3 w kq - 0 1").unwrap(), HashMap::from([(1, 5), (2, 130), (3, 782), (4, 22180), (5, 118882), (6, 3517770)]) ; "ethereal 8")]
#[test_case(Position::from_fen("8/8/8/8/8/8/6k1/4K2R w K - 0 1").unwrap(), HashMap::from([(1, 12), (2, 38), (3, 564), (4, 2219), (5, 37735), (6, 185867)]) ; "ethereal 9")]
#[test_case(Position::from_fen("8/8/8/8/8/8/1k6/R3K3 w Q - 0 1").unwrap(), HashMap::from([(1, 15), (2, 65), (3, 1018), (4, 4573), (5, 80619), (6, 413018)]) ; "ethereal 10")]
#[test_case(Position::from_fen("4k2r/6K1/8/8/8/8/8/8 w k - 0 1").unwrap(), HashMap::from([(1, 3), (2, 32), (3, 134), (4, 2073), (5, 10485), (6, 179869)]) ; "ethereal 11")]
#[test_case(Position::from_fen("r3k3/1K6/8/8/8/8/8/8 w q - 0 1").unwrap(), HashMap::from([(1, 4), (2, 49), (3, 243), (4, 3991), (5, 20780), (6, 367724)]) ; "ethereal 12")]
#[test_case(Position::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap(), HashMap::from([(1, 26), (2, 568), (3, 13744), (4, 314346), (5, 7594526), (6, 179862938)]) ; "ethereal 13")]
#[test_case(Position::from_fen("r3k2r/8/8/8/8/8/8/1R2K2R w Kkq - 0 1").unwrap(), HashMap::from([(1, 25), (2, 567), (3, 14095), (4, 328965), (5, 8153719), (6, 195629489)]) ; "ethereal 14")]
#[test_case(Position::from_fen("r3k2r/8/8/8/8/8/8/2R1K2R w Kkq - 0 1").unwrap(), HashMap::from([(1, 25), (2, 548), (3, 13502), (4, 312835), (5, 7736373), (6, 184411439)]) ; "ethereal 15")]
#[test_case(Position::from_fen("r3k2r/8/8/8/8/8/8/R3K1R1 w Qkq - 0 1").unwrap(), HashMap::from([(1, 25), (2, 547), (3, 13579), (4, 316214), (5, 7878456), (6, 189224276)]) ; "ethereal 16")]
#[test_case(Position::from_fen("1r2k2r/8/8/8/8/8/8/R3K2R w KQk - 0 1").unwrap(), HashMap::from([(1, 26), (2, 583), (3, 14252), (4, 334705), (5, 8198901), (6, 198328929)]) ; "ethereal 17")]
#[test_case(Position::from_fen("2r1k2r/8/8/8/8/8/8/R3K2R w KQk - 0 1").unwrap(), HashMap::from([(1, 25), (2, 560), (3, 13592), (4, 317324), (5, 7710115), (6, 185959088)]) ; "ethereal 18")]
#[test_case(Position::from_fen("r3k1r1/8/8/8/8/8/8/R3K2R w KQq - 0 1").unwrap(), HashMap::from([(1, 25), (2, 560), (3, 13607), (4, 320792), (5, 7848606), (6, 190755813)]) ; "ethereal 19")]
#[test_case(Position::from_fen("4k3/8/8/8/8/8/8/4K2R b K - 0 1").unwrap(), HashMap::from([(1, 5), (2, 75), (3, 459), (4, 8290), (5, 47635), (6, 899442)]) ; "ethereal 20")]
#[test_case(Position::from_fen("4k3/8/8/8/8/8/8/R3K3 b Q - 0 1").unwrap(), HashMap::from([(1, 5), (2, 80), (3, 493), (4, 8897), (5, 52710), (6, 1001523)]) ; "ethereal 21")]
#[test_case(Position::from_fen("4k2r/8/8/8/8/8/8/4K3 b k - 0 1").unwrap(), HashMap::from([(1, 15), (2, 66), (3, 1197), (4, 7059), (5, 133987), (6, 764643)]) ; "ethereal 22")]
#[test_case(Position::from_fen("r3k3/8/8/8/8/8/8/4K3 b q - 0 1").unwrap(), HashMap::from([(1, 16), (2, 71), (3, 1287), (4, 7626), (5, 145232), (6, 846648)]) ; "ethereal 23")]
#[test_case(Position::from_fen("4k3/8/8/8/8/8/8/R3K2R b KQ - 0 1").unwrap(), HashMap::from([(1, 5), (2, 130), (3, 782), (4, 22180), (5, 118882), (6, 3517770)]) ; "ethereal 24")]
#[test_case(Position::from_fen("r3k2r/8/8/8/8/8/8/4K3 b kq - 0 1").unwrap(), HashMap::from([(1, 26), (2, 112), (3, 3189), (4, 17945), (5, 532933), (6, 2788982)]) ; "ethereal 25")]
#[test_case(Position::from_fen("8/8/8/8/8/8/6k1/4K2R b K - 0 1").unwrap(), HashMap::from([(1, 3), (2, 32), (3, 134), (4, 2073), (5, 10485), (6, 179869)]) ; "ethereal 26")]
#[test_case(Position::from_fen("8/8/8/8/8/8/1k6/R3K3 b Q - 0 1").unwrap(), HashMap::from([(1, 4), (2, 49), (3, 243), (4, 3991), (5, 20780), (6, 367724)]) ; "ethereal 27")]
#[test_case(Position::from_fen("4k2r/6K1/8/8/8/8/8/8 b k - 0 1").unwrap(), HashMap::from([(1, 12), (2, 38), (3, 564), (4, 2219), (5, 37735), (6, 185867)]) ; "ethereal 28")]
#[test_case(Position::from_fen("r3k3/1K6/8/8/8/8/8/8 b q - 0 1").unwrap(), HashMap::from([(1, 15), (2, 65), (3, 1018), (4, 4573), (5, 80619), (6, 413018)]) ; "ethereal 29")]
#[test_case(Position::from_fen("r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1").unwrap(), HashMap::from([(1, 26), (2, 568), (3, 13744), (4, 314346), (5, 7594526), (6, 179862938)]) ; "ethereal 30")]
#[test_case(Position::from_fen("r3k2r/8/8/8/8/8/8/1R2K2R b Kkq - 0 1").unwrap(), HashMap::from([(1, 26), (2, 583), (3, 14252), (4, 334705), (5, 8198901), (6, 198328929)]) ; "ethereal 31")]
#[test_case(Position::from_fen("r3k2r/8/8/8/8/8/8/2R1K2R b Kkq - 0 1").unwrap(), HashMap::from([(1, 25), (2, 560), (3, 13592), (4, 317324), (5, 7710115), (6, 185959088)]) ; "ethereal 32")]
#[test_case(Position::from_fen("r3k2r/8/8/8/8/8/8/R3K1R1 b Qkq - 0 1").unwrap(), HashMap::from([(1, 25), (2, 560), (3, 13607), (4, 320792), (5, 7848606), (6, 190755813)]) ; "ethereal 33")]
#[test_case(Position::from_fen("1r2k2r/8/8/8/8/8/8/R3K2R b KQk - 0 1").unwrap(), HashMap::from([(1, 25), (2, 567), (3, 14095), (4, 328965), (5, 8153719), (6, 195629489)]) ; "ethereal 34")]
#[test_case(Position::from_fen("2r1k2r/8/8/8/8/8/8/R3K2R b KQk - 0 1").unwrap(), HashMap::from([(1, 25), (2, 548), (3, 13502), (4, 312835), (5, 7736373), (6, 184411439)]) ; "ethereal 35")]
// #[test_case(Position::from_fen("r3k1r1/8/8/8/8/8/8/R3K2R b Qkq - 0 1").unwrap(), HashMap::from([(1, 25), (2, 547), (3, 13579), (4, 316214), (5, 7878456), (6, 189224276)]) ; "ethereal 36")]
#[test_case(Position::from_fen("8/1n4N1/2k5/8/8/5K2/1N4n1/8 w - - 0 1").unwrap(), HashMap::from([(1, 14), (2, 195), (3, 2760), (4, 38675), (5, 570726), (6, 8107539)]) ; "ethereal 37")]
#[test_case(Position::from_fen("8/1k6/8/5N2/8/4n3/8/2K5 w - - 0 1").unwrap(), HashMap::from([(1, 11), (2, 156), (3, 1636), (4, 20534), (5, 223507), (6, 2594412)]) ; "ethereal 38")]
#[test_case(Position::from_fen("8/8/4k3/3Nn3/3nN3/4K3/8/8 w - - 0 1").unwrap(), HashMap::from([(1, 19), (2, 289), (3, 4442), (4, 73584), (5, 1198299), (6, 19870403)]) ; "ethereal 39")]
#[test_case(Position::from_fen("K7/8/2n5/1n6/8/8/8/k6N w - - 0 1").unwrap(), HashMap::from([(1, 3), (2, 51), (3, 345), (4, 5301), (5, 38348), (6, 588695)]) ; "ethereal 40")]
#[test_case(Position::from_fen("k7/8/2N5/1N6/8/8/8/K6n w - - 0 1").unwrap(), HashMap::from([(1, 17), (2, 54), (3, 835), (4, 5910), (5, 92250), (6, 688780)]) ; "ethereal 41")]
#[test_case(Position::from_fen("8/1n4N1/2k5/8/8/5K2/1N4n1/8 b - - 0 1").unwrap(), HashMap::from([(1, 15), (2, 193), (3, 2816), (4, 40039), (5, 582642), (6, 8503277)]) ; "ethereal 42")]
#[test_case(Position::from_fen("8/1k6/8/5N2/8/4n3/8/2K5 b - - 0 1").unwrap(), HashMap::from([(1, 16), (2, 180), (3, 2290), (4, 24640), (5, 288141), (6, 3147566)]) ; "ethereal 43")]
#[test_case(Position::from_fen("8/8/3K4/3Nn3/3nN3/4k3/8/8 b - - 0 1").unwrap(), HashMap::from([(1, 4), (2, 68), (3, 1118), (4, 16199), (5, 281190), (6, 4405103)]) ; "ethereal 44")]
#[test_case(Position::from_fen("K7/8/2n5/1n6/8/8/8/k6N b - - 0 1").unwrap(), HashMap::from([(1, 17), (2, 54), (3, 835), (4, 5910), (5, 92250), (6, 688780)]) ; "ethereal 45")]
#[test_case(Position::from_fen("k7/8/2N5/1N6/8/8/8/K6n b - - 0 1").unwrap(), HashMap::from([(1, 3), (2, 51), (3, 345), (4, 5301), (5, 38348), (6, 588695)]) ; "ethereal 46")]
#[test_case(Position::from_fen("B6b/8/8/8/2K5/4k3/8/b6B w - - 0 1").unwrap(), HashMap::from([(1, 17), (2, 278), (3, 4607), (4, 76778), (5, 1320507), (6, 22823890)]) ; "ethereal 47")]
#[test_case(Position::from_fen("8/8/1B6/7b/7k/8/2B1b3/7K w - - 0 1").unwrap(), HashMap::from([(1, 21), (2, 316), (3, 5744), (4, 93338), (5, 1713368), (6, 28861171)]) ; "ethereal 48")]
#[test_case(Position::from_fen("k7/B7/1B6/1B6/8/8/8/K6b w - - 0 1").unwrap(), HashMap::from([(1, 21), (2, 144), (3, 3242), (4, 32955), (5, 787524), (6, 7881673)]) ; "ethereal 49")]
#[test_case(Position::from_fen("K7/b7/1b6/1b6/8/8/8/k6B w - - 0 1").unwrap(), HashMap::from([(1, 7), (2, 143), (3, 1416), (4, 31787), (5, 310862), (6, 7382896)]) ; "ethereal 50")]
#[test_case(Position::from_fen("B6b/8/8/8/2K5/5k2/8/b6B b - - 0 1").unwrap(), HashMap::from([(1, 6), (2, 106), (3, 1829), (4, 31151), (5, 530585), (6, 9250746)]) ; "ethereal 51")]
#[test_case(Position::from_fen("8/8/1B6/7b/7k/8/2B1b3/7K b - - 0 1").unwrap(), HashMap::from([(1, 17), (2, 309), (3, 5133), (4, 93603), (5, 1591064), (6, 29027891)]) ; "ethereal 52")]
#[test_case(Position::from_fen("k7/B7/1B6/1B6/8/8/8/K6b b - - 0 1").unwrap(), HashMap::from([(1, 7), (2, 143), (3, 1416), (4, 31787), (5, 310862), (6, 7382896)]) ; "ethereal 53")]
#[test_case(Position::from_fen("K7/b7/1b6/1b6/8/8/8/k6B b - - 0 1").unwrap(), HashMap::from([(1, 21), (2, 144), (3, 3242), (4, 32955), (5, 787524), (6, 7881673)]) ; "ethereal 54")]
#[test_case(Position::from_fen("7k/RR6/8/8/8/8/rr6/7K w - - 0 1").unwrap(), HashMap::from([(1, 19), (2, 275), (3, 5300), (4, 104342), (5, 2161211), (6, 44956585)]) ; "ethereal 55")]
#[test_case(Position::from_fen("R6r/8/8/2K5/5k2/8/8/r6R w - - 0 1").unwrap(), HashMap::from([(1, 36), (2, 1027), (3, 29215), (4, 771461), (5, 20506480), (6, 525169084)]) ; "ethereal 56")]
#[test_case(Position::from_fen("7k/RR6/8/8/8/8/rr6/7K b - - 0 1").unwrap(), HashMap::from([(1, 19), (2, 275), (3, 5300), (4, 104342), (5, 2161211), (6, 44956585)]) ; "ethereal 57")]
#[test_case(Position::from_fen("R6r/8/8/2K5/5k2/8/8/r6R b - - 0 1").unwrap(), HashMap::from([(1, 36), (2, 1027), (3, 29227), (4, 771368), (5, 20521342), (6, 524966748)]) ; "ethereal 58")]
#[test_case(Position::from_fen("6kq/8/8/8/8/8/8/7K w - - 0 1").unwrap(), HashMap::from([(1, 2), (2, 36), (3, 143), (4, 3637), (5, 14893), (6, 391507)]) ; "ethereal 59")]
#[test_case(Position::from_fen("6KQ/8/8/8/8/8/8/7k b - - 0 1").unwrap(), HashMap::from([(1, 2), (2, 36), (3, 143), (4, 3637), (5, 14893), (6, 391507)]) ; "ethereal 60")]
#[test_case(Position::from_fen("K7/8/8/3Q4/4q3/8/8/7k w - - 0 1").unwrap(), HashMap::from([(1, 6), (2, 35), (3, 495), (4, 8349), (5, 166741), (6, 3370175)]) ; "ethereal 61")]
#[test_case(Position::from_fen("6qk/8/8/8/8/8/8/7K b - - 0 1").unwrap(), HashMap::from([(1, 22), (2, 43), (3, 1015), (4, 4167), (5, 105749), (6, 419369)]) ; "ethereal 62")]
#[test_case(Position::from_fen("6KQ/8/8/8/8/8/8/7k b - - 0 1").unwrap(), HashMap::from([(1, 2), (2, 36), (3, 143), (4, 3637), (5, 14893), (6, 391507)]) ; "ethereal 63")]
#[test_case(Position::from_fen("K7/8/8/3Q4/4q3/8/8/7k b - - 0 1").unwrap(), HashMap::from([(1, 6), (2, 35), (3, 495), (4, 8349), (5, 166741), (6, 3370175)]) ; "ethereal 64")]
#[test_case(Position::from_fen("8/8/8/8/8/K7/P7/k7 w - - 0 1").unwrap(), HashMap::from([(1, 3), (2, 7), (3, 43), (4, 199), (5, 1347), (6, 6249)]) ; "ethereal 65")]
#[test_case(Position::from_fen("8/8/8/8/8/7K/7P/7k w - - 0 1").unwrap(), HashMap::from([(1, 3), (2, 7), (3, 43), (4, 199), (5, 1347), (6, 6249)]) ; "ethereal 66")]
#[test_case(Position::from_fen("K7/p7/k7/8/8/8/8/8 w - - 0 1").unwrap(), HashMap::from([(1, 1), (2, 3), (3, 12), (4, 80), (5, 342), (6, 2343)]) ; "ethereal 67")]
#[test_case(Position::from_fen("7K/7p/7k/8/8/8/8/8 w - - 0 1").unwrap(), HashMap::from([(1, 1), (2, 3), (3, 12), (4, 80), (5, 342), (6, 2343)]) ; "ethereal 68")]
#[test_case(Position::from_fen("8/2k1p3/3pP3/3P2K1/8/8/8/8 w - - 0 1").unwrap(), HashMap::from([(1, 7), (2, 35), (3, 210), (4, 1091), (5, 7028), (6, 34834)]) ; "ethereal 69")]
#[test_case(Position::from_fen("8/8/8/8/8/K7/P7/k7 b - - 0 1").unwrap(), HashMap::from([(1, 1), (2, 3), (3, 12), (4, 80), (5, 342), (6, 2343)]) ; "ethereal 70")]
#[test_case(Position::from_fen("8/8/8/8/8/7K/7P/7k b - - 0 1").unwrap(), HashMap::from([(1, 1), (2, 3), (3, 12), (4, 80), (5, 342), (6, 2343)]) ; "ethereal 71")]
#[test_case(Position::from_fen("K7/p7/k7/8/8/8/8/8 b - - 0 1").unwrap(), HashMap::from([(1, 3), (2, 7), (3, 43), (4, 199), (5, 1347), (6, 6249)]) ; "ethereal 72")]
#[test_case(Position::from_fen("7K/7p/7k/8/8/8/8/8 b - - 0 1").unwrap(), HashMap::from([(1, 3), (2, 7), (3, 43), (4, 199), (5, 1347), (6, 6249)]) ; "ethereal 73")]
#[test_case(Position::from_fen("8/2k1p3/3pP3/3P2K1/8/8/8/8 b - - 0 1").unwrap(), HashMap::from([(1, 5), (2, 35), (3, 182), (4, 1091), (5, 5408), (6, 34822)]) ; "ethereal 74")]
#[test_case(Position::from_fen("8/8/8/8/8/4k3/4P3/4K3 w - - 0 1").unwrap(), HashMap::from([(1, 2), (2, 8), (3, 44), (4, 282), (5, 1814), (6, 11848)]) ; "ethereal 75")]
#[test_case(Position::from_fen("4k3/4p3/4K3/8/8/8/8/8 b - - 0 1").unwrap(), HashMap::from([(1, 2), (2, 8), (3, 44), (4, 282), (5, 1814), (6, 11848)]) ; "ethereal 76")]
#[test_case(Position::from_fen("8/8/7k/7p/7P/7K/8/8 w - - 0 1").unwrap(), HashMap::from([(1, 3), (2, 9), (3, 57), (4, 360), (5, 1969), (6, 10724)]) ; "ethereal 77")]
#[test_case(Position::from_fen("8/8/k7/p7/P7/K7/8/8 w - - 0 1").unwrap(), HashMap::from([(1, 3), (2, 9), (3, 57), (4, 360), (5, 1969), (6, 10724)]) ; "ethereal 78")]
#[test_case(Position::from_fen("8/8/3k4/3p4/3P4/3K4/8/8 w - - 0 1").unwrap(), HashMap::from([(1, 5), (2, 25), (3, 180), (4, 1294), (5, 8296), (6, 53138)]) ; "ethereal 79")]
#[test_case(Position::from_fen("8/3k4/3p4/8/3P4/3K4/8/8 w - - 0 1").unwrap(), HashMap::from([(1, 8), (2, 61), (3, 483), (4, 3213), (5, 23599), (6, 157093)]) ; "ethereal 80")]
#[test_case(Position::from_fen("8/8/3k4/3p4/8/3P4/3K4/8 w - - 0 1").unwrap(), HashMap::from([(1, 8), (2, 61), (3, 411), (4, 3213), (5, 21637), (6, 158065)]) ; "ethereal 81")]
#[test_case(Position::from_fen("k7/8/3p4/8/3P4/8/8/7K w - - 0 1").unwrap(), HashMap::from([(1, 4), (2, 15), (3, 90), (4, 534), (5, 3450), (6, 20960)]) ; "ethereal 82")]
#[test_case(Position::from_fen("8/8/7k/7p/7P/7K/8/8 b - - 0 1").unwrap(), HashMap::from([(1, 3), (2, 9), (3, 57), (4, 360), (5, 1969), (6, 10724)]) ; "ethereal 83")]
#[test_case(Position::from_fen("8/8/k7/p7/P7/K7/8/8 b - - 0 1").unwrap(), HashMap::from([(1, 3), (2, 9), (3, 57), (4, 360), (5, 1969), (6, 10724)]) ; "ethereal 84")]
#[test_case(Position::from_fen("8/8/3k4/3p4/3P4/3K4/8/8 b - - 0 1").unwrap(), HashMap::from([(1, 5), (2, 25), (3, 180), (4, 1294), (5,  8296), (6, 53138)]) ; "ethereal 85")]
#[test_case(Position::from_fen("8/3k4/3p4/8/3P4/3K4/8/8 b - - 0 1").unwrap(), HashMap::from([(1, 8), (2, 61), (3, 411), (4, 3213), (5, 21637), (6, 158065)]) ; "ethereal 86")]
#[test_case(Position::from_fen("8/8/3k4/3p4/8/3P4/3K4/8 b - - 0 1").unwrap(), HashMap::from([(1, 8), (2, 61), (3, 483), (4, 3213), (5, 23599), (6, 157093)]) ; "ethereal 87")]
#[test_case(Position::from_fen("k7/8/3p4/8/3P4/8/8/7K b - - 0 1").unwrap(), HashMap::from([(1, 4), (2, 15), (3, 89), (4, 537), (5, 3309), (6, 21104)]) ; "ethereal 88")]
#[test_case(Position::from_fen("7k/3p4/8/8/3P4/8/8/K7 w - - 0 1").unwrap(), HashMap::from([(1, 4), (2, 19), (3, 117), (4, 720), (5, 4661), (6, 32191)]) ; "ethereal 89")]
#[test_case(Position::from_fen("7k/8/8/3p4/8/8/3P4/K7 w - - 0 1").unwrap(), HashMap::from([(1, 5), (2, 19), (3, 116), (4, 716), (5, 4786), (6, 30980)]) ; "ethereal 90")]
#[test_case(Position::from_fen("k7/8/8/7p/6P1/8/8/K7 w - - 0 1").unwrap(), HashMap::from([(1, 5), (2, 22), (3, 139), (4, 877), (5, 6112), (6, 41874)]) ; "ethereal 91")]
#[test_case(Position::from_fen("k7/8/7p/8/8/6P1/8/K7 w - - 0 1").unwrap(), HashMap::from([(1, 4), (2, 16), (3, 101), (4, 637), (5, 4354), (6, 29679)]) ; "ethereal 92")]
#[test_case(Position::from_fen("k7/8/8/6p1/7P/8/8/K7 w - - 0 1").unwrap(), HashMap::from([(1, 5), (2, 22), (3, 139), (4, 877), (5, 6112), (6, 41874)]) ; "ethereal 93")]
#[test_case(Position::from_fen("k7/8/6p1/8/8/7P/8/K7 w - - 0 1").unwrap(), HashMap::from([(1, 4), (2, 16), (3, 101), (4, 637), (5, 4354), (6, 29679)]) ; "ethereal 94")]
#[test_case(Position::from_fen("k7/8/8/3p4/4p3/8/8/7K w - - 0 1").unwrap(), HashMap::from([(1, 3), (2, 15), (3, 84), (4, 573), (5, 3013), (6, 22886)]) ; "ethereal 95")]
#[test_case(Position::from_fen("k7/8/3p4/8/8/4P3/8/7K w - - 0 1").unwrap(), HashMap::from([(1, 4), (2, 16), (3, 101), (4, 637), (5, 4271), (6, 28662)]) ; "ethereal 96")]
#[test_case(Position::from_fen("7k/3p4/8/8/3P4/8/8/K7 b - - 0 1").unwrap(), HashMap::from([(1, 5), (2, 19), (3, 117), (4, 720), (5, 5014), (6, 32167)]) ; "ethereal 97")]
#[test_case(Position::from_fen("7k/8/8/3p4/8/8/3P4/K7 b - - 0 1").unwrap(), HashMap::from([(1, 4), (2, 19), (3, 117), (4, 712), (5, 4658), (6, 30749)]) ; "ethereal 98")]
#[test_case(Position::from_fen("k7/8/8/7p/6P1/8/8/K7 b - - 0 1").unwrap(), HashMap::from([(1, 5), (2, 22), (3, 139), (4, 877), (5, 6112), (6, 41874)]) ; "ethereal 99")]
#[test_case(Position::from_fen("k7/8/7p/8/8/6P1/8/K7 b - - 0 1").unwrap(), HashMap::from([(1, 4), (2, 16), (3, 101), (4, 637), (5, 4354), (6, 29679)]) ; "ethereal 100")]
#[test_case(Position::from_fen("k7/8/8/6p1/7P/8/8/K7 b - - 0 1").unwrap(), HashMap::from([(1, 5), (2, 22), (3, 139), (4, 877), (5, 6112), (6, 41874)]) ; "ethereal 101")]
#[test_case(Position::from_fen("k7/8/6p1/8/8/7P/8/K7 b - - 0 1").unwrap(), HashMap::from([(1, 4), (2, 16), (3, 101), (4, 637), (5, 4354), (6, 29679)]) ; "ethereal 102")]
#[test_case(Position::from_fen("k7/8/8/3p4/4p3/8/8/7K b - - 0 1").unwrap(), HashMap::from([(1, 5), (2, 15), (3, 102), (4, 569), (5, 4337), (6, 22579)]) ; "ethereal 103")]
#[test_case(Position::from_fen("k7/8/3p4/8/8/4P3/8/7K b - - 0 1").unwrap(), HashMap::from([(1, 4), (2, 16), (3, 101), (4, 637), (5, 4271), (6, 28662)]) ; "ethereal 104")]
#[test_case(Position::from_fen("7k/8/8/p7/1P6/8/8/7K w - - 0 1").unwrap(), HashMap::from([(1, 5), (2, 22), (3, 139), (4, 877), (5, 6112), (6, 41874)]) ; "ethereal 105")]
#[test_case(Position::from_fen("7k/8/p7/8/8/1P6/8/7K w - - 0 1").unwrap(), HashMap::from([(1, 4), (2, 16), (3, 101), (4, 637), (5, 4354), (6, 29679)]) ; "ethereal 106")]
#[test_case(Position::from_fen("7k/8/8/1p6/P7/8/8/7K w - - 0 1").unwrap(), HashMap::from([(1, 5), (2, 22), (3, 139), (4, 877), (5, 6112), (6, 41874)]) ; "ethereal 107")]
#[test_case(Position::from_fen("7k/8/1p6/8/8/P7/8/7K w - - 0 1").unwrap(), HashMap::from([(1, 4), (2, 16), (3, 101), (4, 637), (5, 4354), (6, 29679)]) ; "ethereal 108")]
#[test_case(Position::from_fen("k7/7p/8/8/8/8/6P1/K7 w - - 0 1").unwrap(), HashMap::from([(1, 5), (2, 25), (3, 161), (4, 1035), (5, 7574), (6, 55338)]) ; "ethereal 109")]
#[test_case(Position::from_fen("k7/6p1/8/8/8/8/7P/K7 w - - 0 1").unwrap(), HashMap::from([(1, 5), (2, 25), (3, 161), (4, 1035), (5, 7574), (6, 55338)]) ; "ethereal 110")]
#[test_case(Position::from_fen("3k4/3pp3/8/8/8/8/3PP3/3K4 w - - 0 1").unwrap(), HashMap::from([(1, 7), (2, 49), (3, 378), (4, 2902), (5, 24122), (6, 199002)]) ; "ethereal 111")]
#[test_case(Position::from_fen("7k/8/8/p7/1P6/8/8/7K b - - 0 1").unwrap(), HashMap::from([(1, 5), (2, 22), (3, 139), (4, 877), (5, 6112), (6, 41874)]) ; "ethereal 112")]
#[test_case(Position::from_fen("7k/8/p7/8/8/1P6/8/7K b - - 0 1").unwrap(), HashMap::from([(1, 4), (2, 16), (3, 101), (4, 637), (5, 4354), (6, 29679)]) ; "ethereal 113")]
#[test_case(Position::from_fen("7k/8/8/1p6/P7/8/8/7K b - - 0 1").unwrap(), HashMap::from([(1, 5), (2, 22), (3, 139), (4, 877), (5, 6112), (6, 41874)]) ; "ethereal 114")]
#[test_case(Position::from_fen("7k/8/1p6/8/8/P7/8/7K b - - 0 1").unwrap(), HashMap::from([(1, 4), (2, 16), (3, 101), (4, 637), (5, 4354), (6, 29679)]) ; "ethereal 115")]
#[test_case(Position::from_fen("k7/7p/8/8/8/8/6P1/K7 b - - 0 1").unwrap(), HashMap::from([(1, 5), (2, 25), (3, 161), (4, 1035), (5, 7574), (6, 55338)]) ; "ethereal 116")]
#[test_case(Position::from_fen("k7/6p1/8/8/8/8/7P/K7 b - - 0 1").unwrap(), HashMap::from([(1, 5), (2, 25), (3, 161), (4, 1035), (5, 7574), (6, 55338)]) ; "ethereal 117")]
#[test_case(Position::from_fen("3k4/3pp3/8/8/8/8/3PP3/3K4 b - - 0 1").unwrap(), HashMap::from([(1, 7), (2, 49), (3, 378), (4, 2902), (5, 24122), (6, 199002)]) ; "ethereal 118")]
#[test_case(Position::from_fen("8/Pk6/8/8/8/8/6Kp/8 w - - 0 1").unwrap(), HashMap::from([(1, 11), (2, 97), (3, 887), (4, 8048), (5, 90606), (6, 1030499)]) ; "ethereal 119")]
#[test_case(Position::from_fen("n1n5/1Pk5/8/8/8/8/5Kp1/5N1N w - - 0 1").unwrap(), HashMap::from([(1, 24), (2, 421), (3, 7421), (4, 124608), (5, 2193768), (6, 37665329)]) ; "ethereal 120")]
#[test_case(Position::from_fen("8/PPPk4/8/8/8/8/4Kppp/8 w - - 0 1").unwrap(), HashMap::from([(1, 18), (2, 270), (3, 4699), (4, 79355), (5, 1533145), (6, 28859283)]) ; "ethereal 121")]
#[test_case(Position::from_fen("n1n5/PPPk4/8/8/8/8/4Kppp/5N1N w - - 0 1").unwrap(), HashMap::from([(1, 24), (2, 496), (3, 9483), (4, 182838), (5, 3605103), (6, 71179139)]) ; "ethereal 122")]
#[test_case(Position::from_fen("8/Pk6/8/8/8/8/6Kp/8 b - - 0 1").unwrap(), HashMap::from([(1, 11), (2, 97), (3, 887), (4, 8048), (5, 90606), (6, 1030499)]) ; "ethereal 123")]
#[test_case(Position::from_fen("n1n5/1Pk5/8/8/8/8/5Kp1/5N1N b - - 0 1").unwrap(), HashMap::from([(1, 24), (2, 421), (3, 7421), (4, 124608), (5, 2193768), (6, 37665329)]) ; "ethereal 124")]
#[test_case(Position::from_fen("8/PPPk4/8/8/8/8/4Kppp/8 b - - 0 1").unwrap(), HashMap::from([(1, 18), (2, 270), (3, 4699), (4, 79355), (5, 1533145), (6, 28859283)]) ; "ethereal 125")]
#[test_case(Position::from_fen("n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1").unwrap(), HashMap::from([(1, 24), (2, 496), (3, 9483), (4, 182838), (5, 3605103), (6, 71179139)]) ; "ethereal 126")]
#[test_case(Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap(), HashMap::from([(4, 43238), (5, 674624), (6, 11030083)]) ; "ethereal 127")]
#[test_case(Position::from_fen("rnbqkb1r/ppppp1pp/7n/4Pp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3").unwrap(), HashMap::from([(5, 11139762)]) ; "ethereal 128")]
#[ignore]
fn test_perft_tot_moves(starting_position: Position, tot_moves_want: HashMap<usize, usize>) {
    for (depth, depth_tot_moves_want) in tot_moves_want {
        // Skip entries that will take too long
        if depth_tot_moves_want > 5_000_000 {
            continue;
        }
        let (_, depth_tot_moves_got) = perft(&starting_position, depth, MOVE_GEN);

        assert_eq!(depth_tot_moves_got, depth_tot_moves_want);
    }
}

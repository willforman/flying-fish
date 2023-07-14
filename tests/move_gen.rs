use std::collections::HashSet;

use chess::position::Position;
use chess::bitboard::Move;
use chess::bitboard::Square::*;
use chess::move_gen::AllPiecesMoveGen;
use chess::move_gen::leaping_pieces::LeapingPiecesMoveGen;
use chess::move_gen::hyperbola_quintessence::HyperbolaQuintessence;

use test_case::test_case;

#[test_case(Position::start(), HashSet::from_iter([
    Move { src: A2, dest: A3 }, Move { src: A2, dest: A4 },
    Move { src: B2, dest: B3 }, Move { src: B2, dest: B4 },
    Move { src: C2, dest: C3 }, Move { src: C2, dest: C4 },
    Move { src: D2, dest: D3 }, Move { src: D2, dest: D4 },
    Move { src: E2, dest: E3 }, Move { src: E2, dest: E4 },
    Move { src: F2, dest: F3 }, Move { src: F2, dest: F4 },
    Move { src: G2, dest: G3 }, Move { src: G2, dest: G4 },
    Move { src: H2, dest: H3 }, Move { src: H2, dest: H4 },
    Move { src: B1, dest: A3 }, Move { src: B1, dest: C3 },
    Move { src: G1, dest: F3 }, Move { src: G1, dest: H3 }
]))]
#[test_case(Position::from_fen("8/8/p7/1p1p4/1P6/P1P3kp/5p2/1b5K w - - 0 51").unwrap(), HashSet::from_iter([
    Move { src: C3, dest: C4 }, Move { src: A3, dest: A4 },
]))]
fn test_gen_moves(position: Position, want: HashSet<Move>) {
    let leaping_pieces = Box::new(LeapingPiecesMoveGen::new());
    let sliding_pieces = Box::new(HyperbolaQuintessence::new());
    let move_gen = AllPiecesMoveGen::new(leaping_pieces, sliding_pieces);

    let got = move_gen.gen_moves(&position);

    assert_eq!(got, want);
}

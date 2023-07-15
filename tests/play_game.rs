use chess::position::Position;
use chess::bitboard::Move;
use chess::bitboard::Square::*;

use test_case::test_case;
use testresult::TestResult;

#[test_case(Position::start(), vec![
    Move { src: D2, dest: D4 },
    Move { src: D7, dest: D5 },
    Move { src: C2, dest: C4 },
    Move { src: D5, dest: C4 },
    Move { src: E2, dest: E3 },
    Move { src: B7, dest: B5 },
    Move { src: A2, dest: A4 },
    Move { src: C7, dest: C6 },
    Move { src: A4, dest: B5 },
    Move { src: C6, dest: B5 },
    Move { src: D1, dest: F3 },
    Move { src: B8, dest: C6 },
    Move { src: F3, dest: C6 },
    Move { src: C8, dest: D7 },
], Position::from_fen("r2qkbnr/p2bpppp/2Q5/1p6/2pP4/4P3/1P3PPP/RNB1KBNR w KQkq - 1 8").unwrap())]
fn test_play_game(mut starting_position: Position, moves: Vec<Move>, want: Position) -> TestResult {
    for mve in moves {
        starting_position.make_move(mve)?;
    }
    
    assert_eq!(starting_position, want);
    Ok(())
}

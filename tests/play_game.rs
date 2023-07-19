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
], Position::from_fen("r2qkbnr/p2bpppp/2Q5/1p6/2pP4/4P3/1P3PPP/RNB1KBNR w KQkq - 1 8").unwrap() ; "normal")]
#[test_case(Position::start(), vec![
    Move { src: E2, dest: E4 },
    Move { src: E7, dest: E5 },
    Move { src: G1, dest: F3 },
    Move { src: B8, dest: C6 },
    Move { src: F1, dest: B5 },
    Move { src: A7, dest: A6 },
    Move { src: B5, dest: A4 },
    Move { src: F8, dest: E7 },
    Move { src: E1, dest: G1 },
], Position::from_fen("r1bqk1nr/1pppbppp/p1n5/4p3/B3P3/5N2/PPPP1PPP/RNBQ1RK1 b kq - 3 5").unwrap() ; "castling")]
#[test_case(Position::start(), vec![
    Move { src: D2, dest: D4 },
    Move { src: E7, dest: E5 },
    Move { src: D4, dest: D5 },
    Move { src: E5, dest: E4 },
    Move { src: D5, dest: D6 },
    Move { src: E4, dest: E3 },
    Move { src: D6, dest: C7 },
    Move { src: E3, dest: F2 },
    Move { src: E1, dest: F2 },
    Move { src: D7, dest: D5 },
    Move { src: C7, dest: D8 },
], Position::from_fen("rnbQkbnr/pp3ppp/8/3p4/8/8/PPP1PKPP/RNBQ1BNR b kq - 0 6").unwrap() ; "promotion")]
fn test_play_game(mut starting_position: Position, moves: Vec<Move>, want: Position) -> TestResult {
    for mve in moves {
        starting_position.make_move(mve)?;
    }
    
    assert_eq!(starting_position, want);
    Ok(())
}

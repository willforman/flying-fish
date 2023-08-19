use chess::position::{Position,Move};
use chess::bitboard::Square::*;

use test_case::test_case;
use testresult::TestResult;

#[test_case(Position::start(), vec![
    Move { src: D2, dest: D4, promotion: None },
    Move { src: D7, dest: D5, promotion: None },
    Move { src: C2, dest: C4, promotion: None },
    Move { src: D5, dest: C4, promotion: None },
    Move { src: E2, dest: E3, promotion: None },
    Move { src: B7, dest: B5, promotion: None },
    Move { src: A2, dest: A4, promotion: None },
    Move { src: C7, dest: C6, promotion: None },
    Move { src: A4, dest: B5, promotion: None },
    Move { src: C6, dest: B5, promotion: None },
    Move { src: D1, dest: F3, promotion: None },
    Move { src: B8, dest: C6, promotion: None },
    Move { src: F3, dest: C6, promotion: None },
    Move { src: C8, dest: D7, promotion: None },
], Position::from_fen("r2qkbnr/p2bpppp/2Q5/1p6/2pP4/4P3/1P3PPP/RNB1KBNR w KQkq - 1 8").unwrap() ; "normal")]
#[test_case(Position::start(), vec![
    Move { src: E2, dest: E4, promotion: None },
    Move { src: E7, dest: E5, promotion: None },
    Move { src: G1, dest: F3, promotion: None },
    Move { src: B8, dest: C6, promotion: None },
    Move { src: F1, dest: B5, promotion: None },
    Move { src: A7, dest: A6, promotion: None },
    Move { src: B5, dest: A4, promotion: None },
    Move { src: F8, dest: E7, promotion: None },
    Move { src: E1, dest: G1, promotion: None },
], Position::from_fen("r1bqk1nr/1pppbppp/p1n5/4p3/B3P3/5N2/PPPP1PPP/RNBQ1RK1 b kq - 3 5").unwrap() ; "castling")]
#[test_case(Position::start(), vec![
    Move { src: D2, dest: D4, promotion: None },
    Move { src: E7, dest: E5, promotion: None },
    Move { src: D4, dest: D5, promotion: None },
    Move { src: E5, dest: E4, promotion: None },
    Move { src: D5, dest: D6, promotion: None },
    Move { src: E4, dest: E3, promotion: None },
    Move { src: D6, dest: C7, promotion: None },
    Move { src: E3, dest: F2, promotion: None },
    Move { src: E1, dest: F2, promotion: None },
    Move { src: D7, dest: D5, promotion: None },
    Move { src: C7, dest: D8, promotion: None },
], Position::from_fen("rnbQkbnr/pp3ppp/8/3p4/8/8/PPP1PKPP/RNBQ1BNR b kq - 0 6").unwrap() ; "promotion")]
#[test_case(Position::start(), vec![
    Move { src: E2, dest: E4, promotion: None },
    Move { src: E7, dest: E5, promotion: None },
    Move { src: D1, dest: H5, promotion: None },
    Move { src: B8, dest: C6, promotion: None },
    Move { src: F1, dest: C4, promotion: None },
    Move { src: G8, dest: F6, promotion: None },
    Move { src: H5, dest: F7, promotion: None },
], Position::from_fen("r1bqkb1r/pppp1Qpp/2n2n2/4p3/2B1P3/8/PPPP1PPP/RNB1K1NR b KQkq - 0 4").unwrap() ; "scholars mate")]
fn test_play_game(mut starting_position: Position, moves: Vec<Move>, want: Position) -> TestResult {
    for mve in moves {
        starting_position.make_move(&mve)?;
    }
    
    assert_eq!(starting_position, want);
    Ok(())
}

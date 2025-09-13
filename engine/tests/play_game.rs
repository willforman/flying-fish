use engine::Square::*;
use engine::{Move, Piece, Position};

use test_case::test_case;
use testresult::TestResult;

#[test_case(Position::start(), vec![
    Move::new(D2, D4),
    Move::new(D7, D5),
    Move::new(C2, C4),
    Move::new(D5, C4),
    Move::new(E2, E3),
    Move::new(B7, B5),
    Move::new(A2, A4),
    Move::new(C7, C6),
    Move::new(A4, B5),
    Move::new(C6, B5),
    Move::new(D1, F3),
    Move::new(B8, C6),
    Move::new(F3, C6),
    Move::new(C8, D7),
], Position::from_fen("r2qkbnr/p2bpppp/2Q5/1p6/2pP4/4P3/1P3PPP/RNB1KBNR w KQkq - 1 8").unwrap() ; "normal")]
#[test_case(Position::start(), vec![
    Move::new(E2, E4),
    Move::new(E7, E5),
    Move::new(G1, F3),
    Move::new(B8, C6),
    Move::new(F1, B5),
    Move::new(A7, A6),
    Move::new(B5, A4),
    Move::new(F8, E7),
    Move::new(E1, G1),
], Position::from_fen("r1bqk1nr/1pppbppp/p1n5/4p3/B3P3/5N2/PPPP1PPP/RNBQ1RK1 b kq - 3 5").unwrap() ; "castling")]
#[test_case(Position::start(), vec![
    Move::new(D2, D4),
    Move::new(E7, E5),
    Move::new(D4, D5),
    Move::new(E5, E4),
    Move::new(D5, D6),
    Move::new(E4, E3),
    Move::new(D6, C7),
    Move::new(E3, F2),
    Move::new(E1, F2),
    Move::new(D7, D5),
    Move::with_promotion(C7, D8, Piece::Queen),
], Position::from_fen("rnbQkbnr/pp3ppp/8/3p4/8/8/PPP1PKPP/RNBQ1BNR b kq - 0 6").unwrap() ; "promotion")]
#[test_case(Position::start(), vec![
    Move::new(E2, E4),
    Move::new(E7, E5),
    Move::new(D1, H5),
    Move::new(B8, C6),
    Move::new(F1, C4),
    Move::new(G8, F6),
    Move::new(H5, F7),
], Position::from_fen("r1bqkb1r/pppp1Qpp/2n2n2/4p3/2B1P3/8/PPPP1PPP/RNB1K1NR b KQkq - 0 4").unwrap() ; "scholars mate")]
fn test_play_game(mut starting_position: Position, moves: Vec<Move>, want: Position) -> TestResult {
    for mve in moves {
        starting_position.make_move(mve);
    }

    assert_eq!(starting_position, want);
    Ok(())
}

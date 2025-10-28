use std::u64;

use arrayvec::ArrayVec;

use crate::position::{Move, Position};

pub(super) fn order_moves(
    moves: &mut ArrayVec<Move, 218>,
    position: &Position,
    maybe_tt_best_move: Option<Move>,
) {
    moves.sort_by_key(|&mve| -(get_move_sort_key(mve, position, maybe_tt_best_move) as i64))
}

fn get_move_sort_key(mve: Move, position: &Position, maybe_tt_best_move: Option<Move>) -> i64 {
    if let Some(tt_best_move) = maybe_tt_best_move {
        if mve == tt_best_move {
            return i64::MAX;
        }
    }

    if position.is_capture(mve) {
        return get_mvv_lva_value(mve, position);
    }

    return 0;
}

pub(super) fn get_mvv_lva_value(mve: Move, position: &Position) -> i64 {
    let attacker = position
        .is_piece_at(mve.src, position.state.to_move)
        .expect("No piece at attacker square");
    let victim = position
        .is_piece_at(mve.dest, position.state.to_move.opposite_side())
        .expect("No piece at victim square");
    (victim.index() * 10 + (5 - attacker.index())).try_into().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Square::*;
    use test_case::test_case;

    #[test_case(Position::from_fen("7k/8/8/8/5q1b/3q1pP1/2r3b1/K3N3 w - - 0 1").unwrap(), 
        Some(Move::new(A1, B1)),
        [
            Move::new(E1, C2), Move::new(E1, D3), Move::new(E1, F3), Move::new(E1, G2), 
            Move::new(G3, F4), Move::new(G3, G4), Move::new(A1, B1), Move::new(G3, H4), 
        ].into_iter().collect(),
        [
            Move::new(A1, B1), Move::new(G3, F4), Move::new(E1, D3), Move::new(E1, C2), 
            Move::new(G3, H4), Move::new(E1, G2), Move::new(E1, F3), Move::new(G3, G4), 
        ].into_iter().collect() ; "test"
    )]
    fn test_order_moves(
        position: Position,
        maybe_tt_best_move: Option<Move>,
        mut moves_input: ArrayVec<Move, 218>,
        moves_want: ArrayVec<Move, 218>,
    ) {
        order_moves(&mut moves_input, &position, maybe_tt_best_move);

        assert_eq!(moves_input, moves_want);
    }

    #[test_case(Position::from_fen("7k/8/8/8/5q1b/3q1pP1/2r3b1/K3N3 w - - 0 1").unwrap(),
        [
            Move::new(E1, C2), Move::new(E1, D3), Move::new(E1, F3), Move::new(E1, G2),
            Move::new(G3, F4), Move::new(G3, H4)
        ].into_iter().collect(),
        [
            Move::new(G3, F4), Move::new(E1, D3), Move::new(E1, C2), Move::new(G3, H4),
            Move::new(E1, G2), Move::new(E1, F3) 
        ].into_iter().collect() ; "simple"
    )]
    fn test_mvv_lva(
        position: Position,
        mut moves_input: ArrayVec<Move, 218>,
        moves_want: ArrayVec<Move, 218>,
    ) {
        moves_input.sort_by_key(|&mve| -(get_mvv_lva_value(mve, &position) as i64));

        assert_eq!(moves_input, moves_want);
    }
}

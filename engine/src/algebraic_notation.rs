use thiserror;

use crate::move_gen::GenerateMoves;
use crate::position::{Move, Piece, Position};

#[derive(thiserror::Error, Debug)]
pub enum AlgebraicNotationError {
    #[error("no piece at move src {0}")]
    NoPieceAtSrc(String),

    #[error("invalid move {0}")]
    InvalidMove(String),
}

pub fn move_to_algebraic_notation(
    position: &Position,
    mve: Move,
    move_gen: impl GenerateMoves,
) -> Result<String, AlgebraicNotationError> {
    let side = position.state.to_move;
    let opp_side = side.opposite_side();
    let src_piece = position
        .is_piece_at(mve.src, position.state.to_move)
        .ok_or(AlgebraicNotationError::NoPieceAtSrc(mve.src.to_string()))?;

    let move_abs_diff = mve.src.abs_diff(mve.dest);

    // Castling
    if src_piece == Piece::King && move_abs_diff == 2 {
        let res = if mve.src < mve.dest { "O-O" } else { "O-O-O" };
        return Ok(res.to_string());
    }

    let mut res = String::with_capacity(5);
    if src_piece != Piece::Pawn {
        let src_piece_char: char = src_piece.into();
        res.push(src_piece_char.to_ascii_uppercase());
    }

    if position.is_piece_at(mve.dest, opp_side).is_some() {
        if src_piece == Piece::Pawn {
            let src_str = mve.src.to_string();
            let file_char = src_str.chars().next().unwrap().to_ascii_lowercase();
            res.push(file_char);
        }
        res.push('x');
    }

    // En passant
    if src_piece == Piece::Pawn && (move_abs_diff != 8 && move_abs_diff != 16) {
        if position.is_piece_at(mve.dest, opp_side).is_none() {
            let src_str = mve.src.to_string();
            let file_char = src_str.chars().next().unwrap().to_ascii_lowercase();
            res.push(file_char);
            res.push('x');
        }
    }

    // Remove ambiguous moves
    if src_piece == Piece::Rook || src_piece == Piece::Queen {
        let pos_moves = move_gen.gen_moves(&position);
        let filtered_pos_moves: Vec<Move> = pos_moves
            .into_iter()
            .filter(|&other_mve| other_mve.dest == mve.dest) // Only care about moves with the same dest
            .filter(|&other_mve| other_mve.src != mve.src) // Filter moves from the piece we are
            // looking at
            .filter(|&other_mve| {
                // Filter out moves not from the same piece type
                let move_piece = position.is_piece_at(other_mve.src, side).unwrap();
                move_piece == src_piece
            })
            .collect();
        println!("src={:?}, others={:?}", mve, filtered_pos_moves);

        let ambiguous_rank = filtered_pos_moves
            .iter()
            .any(|&other_mve| other_mve.src as u8 / 8 == mve.src as u8 / 8);
        let ambiguous_file = filtered_pos_moves
            .iter()
            .any(|&other_mve| other_mve.src as u8 % 8 == mve.src as u8 % 8);

        if ambiguous_rank || filtered_pos_moves.len() > 1 {
            let src_str = mve.src.to_string();
            let file_char = src_str.chars().next().unwrap().to_ascii_lowercase();
            res.push(file_char);
        }
        if ambiguous_file || filtered_pos_moves.len() > 1 {
            let src_str = mve.src.to_string();
            let rank_char = src_str.chars().nth(1).unwrap().to_ascii_lowercase();
            res.push(rank_char);
        }
    }

    res.push_str(&mve.dest.to_string().to_ascii_lowercase());

    // Pawn promotion
    if let Some(promotion) = mve.promotion {
        let prom_char: char = promotion.into();
        res.push(prom_char.to_ascii_uppercase());
    }

    let mut move_pos = position.clone();
    move_pos.make_move(mve);

    if !move_gen.gen_checkers(&move_pos).is_empty() {
        res.push('+');
    } else {
        let possible_moves_after = move_gen.gen_moves(&move_pos);
        if possible_moves_after.is_empty() {
            res.push('#');
        }
    }

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use test_case::test_case;
    use testresult::TestResult;

    use crate::bitboard::BitBoard;
    use crate::bitboard::Square::*;
    use crate::move_gen::MOVE_GEN;

    #[test_case(Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap(), Move::new(C3, B5), "Nb5".to_string() ; "no capture non pawn")]
    #[test_case(Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap(), Move::new(B2, B3), "b3".to_string() ; "no capture pawn")]
    #[test_case(Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap(), Move::new(E5, G6), "Nxg6".to_string() ; "capture non pawn")]
    #[test_case(Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap(), Move::new(D5, E6), "dxe6".to_string() ; "capture pawn")]
    #[test_case(Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap(), Move::new(E1, G1), "O-O".to_string() ; "castle king side white")]
    #[test_case(Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap(), Move::new(E1, C1), "O-O-O".to_string() ; "castle queen side white")]
    #[test_case(Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQkq - 0 1").unwrap(), Move::new(E8, G8), "O-O".to_string() ; "castle king side black")]
    #[test_case(Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQkq - 0 1").unwrap(), Move::new(E8, C8), "O-O-O".to_string() ; "castle queen side black")]
    #[test_case(Position::from_fen("8/8/8/8/k2Pp3/8/8/7K b - d3 0 1").unwrap(), Move::new(E4, D3), "exd3".to_string() ; "en passant")]
    #[test_case(Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap(), Move::new(D2, D4), "d4".to_string() ; "pawn double push")]
    #[test_case(Position::from_fen("8/8/3P4/8/k7/8/4p2K/8 b - - 0 3").unwrap(), Move::with_promotion(E2, E1, Piece::Queen), "e1Q".to_string() ; "promotion")]
    #[test_case(Position::from_fen("3R3R/8/8/8/8/8/8/K1k5 w - - 0 1").unwrap(), Move::new(D8, F8), "Rdf8".to_string() ; "ambiguous rank")]
    #[test_case(Position::from_fen("7R/8/8/8/7R/8/8/K1k5 w - - 0 1").unwrap(), Move::new(H4, H6), "R4h6".to_string() ; "ambiguous file")]
    #[test_case(Position::from_fen("5Q1Q/8/7Q/8/8/8/8/K2k4 w - - 0 1").unwrap(), Move::new(F8, F6), "Qf8f6".to_string() ; "ambiguous rank file 1")]
    #[test_case(Position::from_fen("5Q1Q/8/7Q/8/8/8/8/K2k4 w - - 0 1").unwrap(), Move::new(H8, F6), "Qh8f6".to_string() ; "ambiguous rank file 2")]
    #[test_case(Position::from_fen("5Q1Q/8/7Q/8/8/8/8/K2k4 w - - 0 1").unwrap(), Move::new(H6, F6), "Qh6f6".to_string() ; "ambiguous rank file 3")]
    fn test_move_to_algebraic_notation(pos: Position, mve: Move, want: String) -> TestResult {
        let move_gen = MOVE_GEN;
        let got = move_to_algebraic_notation(&pos, mve, move_gen)?;

        assert_eq!(got, want);
        Ok(())
    }
}

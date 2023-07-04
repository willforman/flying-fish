use crate::position::{Position,Side,Piece,CastlingRights,Sides,Pieces};
use crate::bitboard::Square;
use crate::bitboard::Square::*;
use std::str::FromStr;

use strum::IntoEnumIterator;

#[derive(thiserror::Error, Debug)]
pub enum FenParseError {
    #[error("num fields: want 6 got {0}")]
    NumFields(usize),

    #[error("piece placement: got {0}, err at {1}")]
    PiecePlacement(String, usize),

    #[error("side to move: want 'w'|'b' got {0}")]
    SideToMove(String),

    #[error("castling rights given: got {0}, err at idx {1}")]
    CastlingRights(String, usize),

    #[error("en passant target: got {0}")]
    EnPassantTarget(String),

    #[error("halfmove clock: want 0 <= x < 50 got {0}")]
    HalfmoveClock(String)
}

impl Position {
    pub fn from_fen(fen: &str) -> Result<Self, FenParseError> {
        let fields = fen.split(' ').collect::<Vec<&str>>(); 

        if fields.len() != 6 {
            Err(FenParseError::NumFields(fields.len()))?
        }

        let to_move = match fields[1] {
            "w" => Side::White,
            "b" => Side::Black,
            _ => Err(FenParseError::SideToMove(String::from(fields[1])))?
        };

        let castling_rights = castling_rights_from_fen(fields[2])?;

        let half_move_clock = fields[4].parse::<u8>()
            .map_err(|_| FenParseError::HalfmoveClock(fields[4].to_string()))?;

        // Half move counter must be in 0..=49
        // Don't have to check if less than zero because u8 min value = 0
        if half_move_clock >= 50 {
            Err(FenParseError::HalfmoveClock(fields[4].to_string()))?
        }

        Ok(Position::start())
    }
}

fn castling_rights_from_fen(castling_rights_str: &str) -> Result<CastlingRights, FenParseError> {
    if castling_rights_str.is_empty() || castling_rights_str == "-" {
        return Ok(CastlingRights::new(false, false, false, false));
    }

    let mut white_king_side = false;
    let mut white_queen_side = false;
    let mut black_king_side = false;
    let mut black_queen_side = false;

    for (idx, ch) in castling_rights_str.chars().enumerate() {
        match ch {
            'K' => {
                if white_king_side {
                    return Err(FenParseError::CastlingRights(castling_rights_str.to_string(), idx))
                }
                white_king_side = true;
            }
            'Q' => {
                if white_queen_side {
                    return Err(FenParseError::CastlingRights(castling_rights_str.to_string(), idx));
                }
                white_queen_side = true;
            }
            'k' => {
                if black_king_side {
                    return Err(FenParseError::CastlingRights(castling_rights_str.to_string(), idx));
                }
                black_king_side = true;
            }
            'q' => {
                if black_queen_side {
                    return Err(FenParseError::CastlingRights(castling_rights_str.to_string(), idx));
                }
                black_queen_side = true;
            }
            _ => return Err(FenParseError::CastlingRights(castling_rights_str.to_string(), idx)),
        }
    }

    Ok(CastlingRights::new(white_king_side, white_queen_side, black_king_side, black_queen_side))
}

fn en_passant_target_from_fen(en_passant_target_str: &str) -> Result<Option<Square>, FenParseError> {
    if en_passant_target_str == "-" {
        return Ok(None);
    }

    // FEN uses lowercase letter for square names, Square uses uppercase
    Square::from_str(&en_passant_target_str.to_uppercase())
        .map_err(|_| FenParseError::EnPassantTarget(en_passant_target_str.to_string()))
        .map(Some)
}

const FEN_SQUARE_ORDER: [Square; 64] = [
    A8, B8, C8, D8, E8, F8, G8, H8,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A1, B1, C1, D1, E1, F1, G1, H1
];

fn pieces_from_fen(pieces_str: &str) -> Result<(Sides, Pieces), FenParseError> {
    let mut sides = Sides::new();
    let mut pieces = Pieces::new();
    let mut sq_idx = 0;

    for (ch_idx, ch) in pieces_str.chars().enumerate() {
        if let Ok(piece) = Piece::try_from(ch.to_ascii_lowercase()) {
            let square = FEN_SQUARE_ORDER[sq_idx];
            let side = if ch.is_uppercase() { Side::White } else { Side::Black }; 

            sides[&side].add_piece(&square);
            pieces[&piece][&side].add_piece(&square);

            sq_idx += 1;
        } else if let Some(digit) = ch.to_digit(10){
            sq_idx += digit as usize;
        } else if ch == '/' {
            // pass
        } else {
            Err(FenParseError::PiecePlacement(pieces_str.to_string(), ch_idx))?
        }
    }

    println!("WHITE");
    println!("pawns:");
    println!("{:?}", pieces.pawns.white);
    println!("knights:");
    println!("{:?}", pieces.knights.white);
    println!("bishops:");
    println!("{:?}", pieces.bishops.white);
    println!("rooks:");
    println!("{:?}", pieces.rooks.white);
    println!("queeens:");
    println!("{:?}", pieces.queens.white);
    println!("kings:");
    println!("{:?}", pieces.kings.white);

    Ok((sides, pieces))
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;
    use testresult::TestResult;

    #[test_case("-", CastlingRights::new(false, false, false, false) ; "empty")]
    #[test_case("KQkq", CastlingRights::new(true, true, true, true)  ; "KQkq")]
    #[test_case("Qk", CastlingRights::new(false, true, true, false)  ; "Qk")]
    #[test_case("K", CastlingRights::new(true, false, false, false)  ; "K")]
    fn test_castling_rights_from_fen(inp: &str, want: CastlingRights) -> TestResult {
        let got = castling_rights_from_fen(inp)?;
        assert_eq!(got, want);
        Ok(())
    }

    #[test_case("abc")]
    fn test_castling_rights_from_fen_invalid(inp: &str) {
        let got = castling_rights_from_fen(inp);
        assert!(matches!(got, Err(FenParseError::CastlingRights(_, _))));
    }

    #[test_case("-", None      ; "empty")]
    #[test_case("e3", Some(E3) ; "e3")]
    #[test_case("c6", Some(C6) ; "c6")]
    fn test_en_passant_target_from_fen(inp: &str, want: Option<Square>) -> TestResult {
        let got = en_passant_target_from_fen(inp)?;
        assert_eq!(got, want);
        Ok(())
    }

    #[test_case("abc")]
    fn test_en_passant_target_from_fen_invalid(inp: &str) {
        let got = en_passant_target_from_fen(inp);
        assert!(matches!(got, Err(FenParseError::EnPassantTarget(_))));
    }

    // 1R2k3/2Q5/8/8/7p/8/5P1P/6K1 b - - 7 42
    #[test_case("1R2k3/2Q5/8/8/7p/8/5P1P/6K1", [
        (B8, Piece::Rook, Side::White),
        (E8, Piece::King, Side::Black),
        (C7, Piece::Queen, Side::White),
        (H4, Piece::Pawn, Side::Black),
        (F2, Piece::Pawn, Side::White),
        (H2, Piece::Pawn, Side::White),
        (G1, Piece::King, Side::White),
    ] ; "first")]
    fn test_pieces_from_fen(inp: &str, expected_pieces: [( Square, Piece, Side ); 7]) -> TestResult {
        let (sides, pieces) = pieces_from_fen(inp)?;

        println!("WHITE");
        println!("pawns:");
        println!("{:?}", pieces.pawns.white);
        println!("knights:");
        println!("{:?}", pieces.knights.white);
        println!("bishops:");
        println!("{:?}", pieces.bishops.white);
        println!("rooks:");
        println!("{:?}", pieces.rooks.white);
        println!("queeens:");
        println!("{:?}", pieces.queens.white);
        println!("kings:");
        println!("{:?}", pieces.kings.white);

        println!("BLACK");
        println!("pawns:");
        println!("{:?}", pieces.pawns.black);
        println!("knights:");
        println!("{:?}", pieces.knights.black);
        println!("bishops:");
        println!("{:?}", pieces.bishops.black);
        println!("rooks:");
        println!("{:?}", pieces.rooks.black);
        println!("queeens:");
        println!("{:?}", pieces.queens.black);
        println!("kings:");
        println!("{:?}", pieces.kings.black);

        for square in Square::iter() {
            let maybe_piece_here = expected_pieces.iter()
                .find(|&&(piece_square, _, _)| square == piece_square);
            if let Some((_, piece, piece_side)) = maybe_piece_here {
                let opp_piece_side = if piece_side == &Side::White { Side::Black } else { Side::White };
                assert!(sides[piece_side].is_piece_at(&square));
                assert!(!sides[&opp_piece_side].is_piece_at(&square));

                // Check if the piece is at this square, and make sure other
                // piece types aren't also at this square. Also make sure 
                // a piece from the other side isn't there.
                for check_piece in Piece::iter() {
                    let is_piece_here = pieces[&check_piece][piece_side].is_piece_at(&square);
                    if piece == &check_piece {
                        assert!(is_piece_here);
                    } else {
                        assert!(!is_piece_here);
                    }
                    assert!(!pieces[piece][&opp_piece_side].is_piece_at(&square));
                }
            } else {
                for side in Side::iter() {
                    assert!(!sides[&side].is_piece_at(&square));
                    for piece in Piece::iter() {
                        assert!(!pieces[&piece][&side].is_piece_at(&square));
                    }
                }
            }
        }

        Ok(())
    }
 }

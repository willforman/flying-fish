use crate::{position::{Position,Side,CastlingRights}, bitboard::Square};
use std::str::FromStr;

#[derive(thiserror::Error, Debug)]
pub enum FenParseError {
    #[error("num fields: want 6 got {0}")]
    NumFields(usize),

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

#[cfg(test)]
mod tests {
    use super::*;
    use super::Square::*;
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
 }

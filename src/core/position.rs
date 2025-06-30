use crate::utility::*;
use crate::core::{
    bitboard::*,
    player::Player,
    chess_move::*,
    piece::Piece,
    zobrist::zobrist_hash,
};

/// Uses [Little-Endian Rank-File Mapping](https://www.chessprogramming.org/Square_Mapping_Considerations#Little-Endian_Rank-File_Mapping)
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Position {
    pub w: BitboardSet,
    pub b: BitboardSet,
    pub occupied: u64,
    pub player_to_move: Player,
    pub en_passant_square: Option<u8>,
    pub castling: CastlingRights,
    pub zobrist_hash: u64,
}

#[derive(Debug)]
pub enum FenParseError {
    BadFieldCount,
    InvalidPieceChar(char),
    BadRankCount,
    BadFileCount,
    InvalidSide(String),
    InvalidCastling(String),
    InvalidEnPassant(String),
    InvalidHalfmove(String),
    InvalidFullmove(String),
}

impl Default for Position {
    fn default() -> Self {
        Position::start()
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut board = [['.'; 8]; 8];

        for rank in 0..8 {
            for file in 0..8 {
                let sq_idx = rank * 8 + file;
                let what = self.what(sq_idx);

                if let Some((player, piece)) = what {
                    let uppercase = player == Player::White;
                    let letter = piece.to_char();

                    board[7 - rank as usize][file as usize] = if uppercase {
                        letter.to_ascii_uppercase()
                    } else {
                        letter
                    };
                }
            }
        }

        for (rank, row) in board.iter().enumerate() {
            write!(f, "{} ", 8 - rank)?;
            for square in row {
                write!(f, "{} ", square)?;
            }
            writeln!(f)?;
        }
        writeln!(f, "  a b c d e f g h")?;
        if let Some(ep_sq) = self.en_passant_square {
            writeln!(f, "En passant square: {:?}", square_idx_to_string(ep_sq))?;
        }
        writeln!(f, "Player to move: {:?}", self.player_to_move)?;
        write!(f, "Castling rights: {}", self.castling)?;
        Ok(())
    }
}

impl Position {
    pub fn start() -> Self {
        let mut pos = Position {
            w: BitboardSet {
                all:     0x000000000000FFFF,
                pawns:   0x000000000000FF00,
                knights: 0x0000000000000042,
                bishops: 0x0000000000000024,
                rooks:   0x0000000000000081,
                queens:  0x0000000000000008,
                king:    0x0000000000000010,
            },
            b: BitboardSet {
                all:     0xFFFF000000000000,
                pawns:   0x00FF000000000000,
                knights: 0x4200000000000000,
                bishops: 0x2400000000000000,
                rooks:   0x8100000000000000,
                queens:  0x0800000000000000,
                king:    0x1000000000000000,
            },
            occupied: 0xFFFF00000000FFFF,
            player_to_move: Player::White,
            en_passant_square: None,
            castling: CastlingRights::default(),
            zobrist_hash: 0,
        };
        pos.zobrist_hash = zobrist_hash(&pos);
        pos
    }

    fn validate_fen(fen: &str) -> Result<(), FenParseError> {
        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.len() != 6 {
            return Err(FenParseError::BadFieldCount);
        }

        let (placement, side, castling, en_passant, halfmove, fullmove) =
            (parts[0], parts[1], parts[2], parts[3], parts[4], parts[5]);

        // Validate placement
        let ranks: Vec<&str> = placement.split('/').collect();
        if ranks.len() != 8 {
            return Err(FenParseError::BadRankCount);
        }
        for rank in ranks {
            let mut file_count = 0;
            for c in rank.chars() {
                if c.is_ascii_digit() {
                    file_count += c.to_digit(10).unwrap();
                } else if "pnbrqkPNBRQK".contains(c) {
                    file_count += 1;
                } else {
                    return Err(FenParseError::InvalidPieceChar(c));
                }
            }
            if file_count != 8 {
                return Err(FenParseError::BadFileCount);
            }
        }

        if side != "w" && side != "b" {
            return Err(FenParseError::InvalidSide(side.into()));
        }

        if castling != "-" && !castling.chars().all(|c| "KQkq".contains(c)) {
            return Err(FenParseError::InvalidCastling(castling.into()));
        }

        if en_passant != "-" && square_string_to_idx(en_passant).is_none() {
            return Err(FenParseError::InvalidEnPassant(en_passant.into()));
        }

        if halfmove.parse::<u32>().is_err() {
            return Err(FenParseError::InvalidHalfmove(halfmove.into()));
        }

        if fullmove.parse::<u32>().ok().filter(|n| *n >= 1).is_none() {
            return Err(FenParseError::InvalidFullmove(fullmove.into()));
        }

        Ok(())
    }

    // Returns (position, halfmove_clock)
    pub fn from_fen(fen: &str) -> Result<(Self, usize), FenParseError> {
        Self::validate_fen(fen)?;

        let mut w = BitboardSet::default();
        let mut b = BitboardSet::default();

        let parts: Vec<&str> = fen.split_whitespace().collect();
        let board = parts[0];
        let side_to_move = parts[1];
        let castling = CastlingRights::from_string(parts[2]);
        let en_passant_square = match parts[3] {
            "-" => None,
            _ => square_string_to_idx(parts[3])
        };
        let halfmove_clock = parts[4].parse::<usize>().unwrap();

        // Starting from the top-left, 0-indexed [0; 7]
        let mut rank = 7;
        let mut file = 0;

        for c in board.chars() {
            if c == '/' {
                rank -= 1;
                file = 0;
                continue;
            }
            if c.is_ascii_digit() {
                file += c.to_digit(10).unwrap() as usize;
                continue;
            }

            let square = 1u64 << (rank * 8 + file);
            match c {
                'P' => w.pawns   |= square,
                'N' => w.knights |= square,
                'B' => w.bishops |= square,
                'R' => w.rooks   |= square,
                'Q' => w.queens  |= square,
                'K' => w.king    |= square,
                'p' => b.pawns   |= square,
                'n' => b.knights |= square,
                'b' => b.bishops |= square,
                'r' => b.rooks   |= square,
                'q' => b.queens  |= square,
                'k' => b.king    |= square,
                _ => (),
            }
            file += 1;
        }

        w.update();
        b.update();
        let occupied: u64 = w.all | b.all;

        let mut pos = Position {
            w, b, occupied,
            player_to_move: match side_to_move {
                "w" => Player::White,
                "b" => Player::Black,
                _   => unreachable!()
            },
            en_passant_square,
            castling,
            zobrist_hash: 0,
        };
        pos.zobrist_hash = zobrist_hash(&pos);
        Ok((pos, halfmove_clock))
    }

    // Mutate fields `w`, `b` and `occupied` so they are correct
    pub fn update(&mut self) {
        self.w.update();
        self.b.update();
        self.occupied = self.w.all | self.b.all;
    }

    pub fn what(&self, sq_idx: u8) -> Option<(Player, Piece)> {
        let bb = bit(sq_idx);

        if self.w.pawns & bb != 0 {
            Some((Player::White, Piece::Pawn))
        } else if self.w.knights & bb != 0 {
            Some((Player::White, Piece::Knight))
        } else if self.w.bishops & bb != 0 {
            Some((Player::White, Piece::Bishop))
        } else if self.w.rooks & bb != 0 {
            Some((Player::White, Piece::Rook))
        } else if self.w.queens & bb != 0 {
            Some((Player::White, Piece::Queen))
        } else if self.w.king & bb != 0 {
            Some((Player::White, Piece::King))
        } else if self.b.pawns & bb != 0 {
            Some((Player::Black, Piece::Pawn))
        } else if self.b.knights & bb != 0 {
            Some((Player::Black, Piece::Knight))
        } else if self.b.bishops & bb != 0 {
            Some((Player::Black, Piece::Bishop))
        } else if self.b.rooks & bb != 0 {
            Some((Player::Black, Piece::Rook))
        } else if self.b.queens & bb != 0 {
            Some((Player::Black, Piece::Queen))
        } else if self.b.king & bb != 0 {
            Some((Player::Black, Piece::King))
        } else {
            None
        }
    }

    pub fn perspective_mut(&mut self, player: Player) -> (&mut BitboardSet, &mut BitboardSet) {
        match player {
            Player::White => (&mut self.w, &mut self.b),
            Player::Black => (&mut self.b, &mut self.w),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fen_start() -> Result<(), FenParseError> {
        let (pos, _) = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")?;
        assert_eq!(pos.w.pawns,   0x000000000000FF00);
        assert_eq!(pos.w.rooks,   0x0000000000000081);
        assert_eq!(pos.w.knights, 0x0000000000000042);
        assert_eq!(pos.w.bishops, 0x0000000000000024);
        assert_eq!(pos.w.queens,  0x0000000000000008);
        assert_eq!(pos.w.king,    0x0000000000000010);
        assert_eq!(pos.w.all,     0x000000000000FFFF);
        assert_eq!(pos.b.pawns,   0x00FF000000000000);
        assert_eq!(pos.b.rooks,   0x8100000000000000);
        assert_eq!(pos.b.knights, 0x4200000000000000);
        assert_eq!(pos.b.bishops, 0x2400000000000000);
        assert_eq!(pos.b.queens,  0x0800000000000000);
        assert_eq!(pos.b.king,    0x1000000000000000);
        assert_eq!(pos.b.all,     0xFFFF000000000000);
        assert_eq!(pos.occupied,  0xFFFF00000000FFFF);
        assert_eq!(pos.player_to_move, Player::White);
        Ok(())
    }

    #[test]
    fn fen_empty() -> Result<(), FenParseError> {
        let (pos, _) = Position::from_fen("8/8/8/8/8/8/8/8 b - - 0 1")?;
        assert_eq!(pos.w.pawns,   0x0);
        assert_eq!(pos.w.rooks,   0x0);
        assert_eq!(pos.w.knights, 0x0);
        assert_eq!(pos.w.bishops, 0x0);
        assert_eq!(pos.w.queens,  0x0);
        assert_eq!(pos.w.king,    0x0);
        assert_eq!(pos.w.all,     0x0);
        assert_eq!(pos.b.pawns,   0x0);
        assert_eq!(pos.b.rooks,   0x0);
        assert_eq!(pos.b.knights, 0x0);
        assert_eq!(pos.b.bishops, 0x0);
        assert_eq!(pos.b.queens,  0x0);
        assert_eq!(pos.b.king,    0x0);
        assert_eq!(pos.b.all,     0x0);
        assert_eq!(pos.occupied,  0x0);
        assert_eq!(pos.player_to_move, Player::Black);
        Ok(())
    }

    #[test]
    fn fen_endgame() -> Result<(), FenParseError> {
        let (pos, _) = Position::from_fen("4r3/2n5/8/6R1/3k4/8/1B6/4K3 w - - 0 1")?;
        assert_eq!(pos.w.pawns,   0x0);
        assert_eq!(pos.w.rooks,   bit(38));
        assert_eq!(pos.w.knights, 0x0);
        assert_eq!(pos.w.bishops, bit(9));
        assert_eq!(pos.w.queens,  0x0);
        assert_eq!(pos.w.king,    bit(4));
        assert_eq!(pos.w.all,     bit(4) | bit(9) | bit(38));

        assert_eq!(pos.b.pawns,   0x0);
        assert_eq!(pos.b.rooks,   bit(60));
        assert_eq!(pos.b.knights, bit(50));
        assert_eq!(pos.b.bishops, 0x0);
        assert_eq!(pos.b.queens,  0x0);
        assert_eq!(pos.b.king,    bit(27));
        assert_eq!(pos.b.all,     bit(27) | bit(50) | bit(60));

        assert_eq!(pos.occupied,  bit(4) | bit(9) | bit(27) | bit(38) | bit(50) | bit(60));
        assert_eq!(pos.player_to_move, Player::White);
        Ok(())
    }
}

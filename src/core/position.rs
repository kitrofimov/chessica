use crate::utility::*;
use crate::core::{
    bitboard::*,
    player::Player,
    chess_move::*,
};

/// Uses [Little-Endian Rank-File Mapping](https://www.chessprogramming.org/Square_Mapping_Considerations#Little-Endian_Rank-File_Mapping)
#[derive(Copy, Clone)]
pub struct Position {
    pub w: BitboardSet,
    pub b: BitboardSet,
    pub occupied: u64,
    pub player_to_move: Player,
    pub en_passant_square: Option<u8>,
    pub castling: CastlingRights,
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
                let square = 1u64 << (rank * 8 + file);

                if self.w.pawns & square != 0 {
                    board[7 - rank][file] = 'P';
                } else if self.w.knights & square != 0 {
                    board[7 - rank][file] = 'N';
                } else if self.w.bishops & square != 0 {
                    board[7 - rank][file] = 'B';
                } else if self.w.rooks & square != 0 {
                    board[7 - rank][file] = 'R';
                } else if self.w.queens & square != 0 {
                    board[7 - rank][file] = 'Q';
                } else if self.w.king & square != 0 {
                    board[7 - rank][file] = 'K';
                } else if self.b.pawns & square != 0 {
                    board[7 - rank][file] = 'p';
                } else if self.b.knights & square != 0 {
                    board[7 - rank][file] = 'n';
                } else if self.b.bishops & square != 0 {
                    board[7 - rank][file] = 'b';
                } else if self.b.rooks & square != 0 {
                    board[7 - rank][file] = 'r';
                } else if self.b.queens & square != 0 {
                    board[7 - rank][file] = 'q';
                } else if self.b.king & square != 0 {
                    board[7 - rank][file] = 'k';
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
        writeln!(f, "Turn: {:?}", self.player_to_move)?;
        writeln!(f, "Castling rights: {}", self.castling.to_string())?;
        if let Some(ep_sq) = self.en_passant_square {
            writeln!(f, "En passant square: {:?}", square_idx_to_string(ep_sq))?;
        }
        Ok(())
    }
}

impl Position {
    pub fn start() -> Self {
        Position {
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
        }
    }

    // TODO: not validating FEN in any way. regular expressions?
    pub fn from_fen(fen: &str) -> Self {
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

        // Starting from the top-left, 0-indexed [0; 7]
        let mut rank = 7;
        let mut file = 0;

        for c in board.chars() {
            if c == '/' {
                rank -= 1;
                file = 0;
                continue;
            }
            if c.is_digit(10) {
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

        Position {
            w, b, occupied,
            player_to_move: match side_to_move {
                "w" => Player::White,
                "b" => Player::Black,
                _ => Player::White  // default to white
            },
            en_passant_square,
            castling,
        }
    }

    // Mutate fields `w`, `b` and `occupied` so they are correct
    pub fn update(&mut self) {
        self.w.update();
        self.b.update();
        self.occupied = self.w.all | self.b.all;
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fen_start() {
        let pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
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
    }

    #[test]
    fn fen_empty() {
        let pos = Position::from_fen("8/8/8/8/8/8/8/8 b - - 0 1");
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
    }

    #[test]
    fn fen_endgame() {
        let pos = Position::from_fen("4r3/2n5/8/6R1/3k4/8/1B6/4K3 w - - 0 1");
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
    }
}

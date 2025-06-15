#[derive(Default)]
struct Bitboards {
    all:     u64,
    pawns:   u64,
    knights: u64,
    bishops: u64,
    rooks:   u64,
    queens:  u64,
    king:    u64,  // only one king per side, so not plural :)
}

impl Bitboards {
    fn update_all(&mut self) {
        self.all = self.pawns | self.knights | self.bishops | self.rooks | self.queens | self.king;
    }
}

/// Uses [Little-Endian Rank-File Mapping](https://www.chessprogramming.org/Square_Mapping_Considerations#Little-Endian_Rank-File_Mapping)
struct Position {
    w: Bitboards,
    b: Bitboards,
    occupied: u64,
    whites_turn: bool,
}

impl Default for Position {
    fn default() -> Self {
        Position::start()
    }
}

impl std::fmt::Debug for Position {
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

        writeln!(f, "\n  a b c d e f g h")?;
        for (rank, row) in board.iter().enumerate() {
            write!(f, "{} ", 8 - rank)?;
            for square in row {
                write!(f, "{} ", square)?;
            }
            writeln!(f)?;
        }
        writeln!(f, "Turn: {}", if self.whites_turn { "White" } else { "Black" })
    }
}

impl Position {
    fn start() -> Self {
        Position {
            w: Bitboards {
                all:     0x000000000000FFFF,
                pawns:   0x000000000000FF00,
                knights: 0x0000000000000042,
                bishops: 0x0000000000000024,
                rooks:   0x0000000000000081,
                queens:  0x0000000000000010,
                king:    0x0000000000000008,
            },
            b: Bitboards {
                all:     0xFFFF000000000000,
                pawns:   0x00FF000000000000,
                knights: 0x4200000000000000,
                bishops: 0x2400000000000000,
                rooks:   0x8100000000000000,
                queens:  0x1000000000000000,
                king:    0x0800000000000000,
            },
            occupied: 0xFFFF00000000FFFF,
            whites_turn: true,
        }
    }

    fn from_fen(fen: &str) -> Self {
        let mut w = Bitboards::default();
        let mut b = Bitboards::default();

        let parts: Vec<&str> = fen.split_whitespace().collect();
        let board = parts[0];
        let side_to_move = parts[1];

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

        w.update_all();
        b.update_all();
        let occupied: u64 = w.all | b.all;

        Position {
            w, b, occupied,
            whites_turn: side_to_move == "w"
        }
    }
}


fn main() {
    let pos = Position::start();
    println!("{:?}", pos);
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
        assert_eq!(pos.whites_turn, true);
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
        assert_eq!(pos.whites_turn, false);
    }

    #[test]
    fn fen_endgame() {
        let pos = Position::from_fen("4r3/2n5/8/6R1/3k4/8/1B6/4K3 w - - 0 1");
        assert_eq!(pos.w.pawns,   0x0);
        assert_eq!(pos.w.rooks,   1 << 38);
        assert_eq!(pos.w.knights, 0x0);
        assert_eq!(pos.w.bishops, 1 << 9);
        assert_eq!(pos.w.queens,  0x0);
        assert_eq!(pos.w.king,    1 << 4);
        assert_eq!(pos.w.all,     (1 << 4) | (1 << 9) | (1 << 38));

        assert_eq!(pos.b.pawns,   0x0);
        assert_eq!(pos.b.rooks,   1 << 60);
        assert_eq!(pos.b.knights, 1 << 50);
        assert_eq!(pos.b.bishops, 0x0);
        assert_eq!(pos.b.queens,  0x0);
        assert_eq!(pos.b.king,    1 << 27);
        assert_eq!(pos.b.all,     (1 << 27) | (1 << 50) | (1 << 60));

        assert_eq!(pos.occupied,  (1 << 4) | (1 << 9) | (1 << 27) | (1 << 38) | (1 << 50) | (1 << 60));
        assert_eq!(pos.whites_turn, true);
    }
}

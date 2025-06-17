use chess_engine::constants::*;
use chess_engine::utility::*;

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

#[derive(Debug, PartialEq, Eq, Hash)]
enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Move {
    from: u8,
    to: u8,
    piece: Piece,
    promotion: Option<Piece>,
    en_passant: bool,
}

/// Uses [Little-Endian Rank-File Mapping](https://www.chessprogramming.org/Square_Mapping_Considerations#Little-Endian_Rank-File_Mapping)
struct Position {
    w: Bitboards,
    b: Bitboards,
    occupied: u64,
    whites_turn: bool,
    en_passant_square: Option<u8>,
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

        for (rank, row) in board.iter().enumerate() {
            write!(f, "{} ", 8 - rank)?;
            for square in row {
                write!(f, "{} ", square)?;
            }
            writeln!(f)?;
        }
        writeln!(f, "  a b c d e f g h")?;
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
            en_passant_square: None,
        }
    }

    fn from_fen(fen: &str) -> Self {
        let mut w = Bitboards::default();
        let mut b = Bitboards::default();

        let parts: Vec<&str> = fen.split_whitespace().collect();
        let board = parts[0];
        let side_to_move = parts[1];
        let castling = parts[2];
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

        w.update_all();
        b.update_all();
        let occupied: u64 = w.all | b.all;

        Position {
            w, b, occupied,
            whites_turn: side_to_move == "w",
            en_passant_square,
        }
    }

    fn add_pawn_moves(moves: &mut Vec<Move>, to_mask: u64, offset: i8, promotion: bool, en_passant: bool) {
        let mut bb = to_mask;
        while bb != 0 {
            let to = pop_lsb(&mut bb) as i8;
            let from = to - offset;
            // TODO: if inside a loop? too slow?
            if promotion {
                for promo in [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight] {
                    moves.push(Move {
                        from: from as u8,
                        to: to as u8,
                        piece: Piece::Pawn,
                        promotion: Some(promo),
                        en_passant: false,
                    });
                }
            } else if en_passant {
                moves.push(Move {
                    from: from as u8,
                    to: to as u8,
                    piece: Piece::Pawn,
                    promotion: None,
                    en_passant: true,
                });
            } else {
                moves.push(Move {
                    from: from as u8,
                    to: to as u8,
                    piece: Piece::Pawn,
                    promotion: None,
                    en_passant: false,
                });
            }
        }
    }

    fn generate_pseudo_pawn_moves(&self, moves: &mut Vec<Move>) {
        let empty = !self.occupied;
        let en_passant_bb = self.en_passant_square.map(|sq| 1u64 << sq).unwrap_or(0);
        let (pawns, enemy, left_offset, forward_offset, right_offset,
             start_rank, promo_rank, mask_right, mask_left) =
            if self.whites_turn {
                (
                    self.w.pawns,
                    self.b.all,
                    7,
                    8,
                    9,
                    RANK[2],
                    RANK[8],
                    !FILE_H,
                    !FILE_A,
                )
            } else {
                (
                    self.b.pawns,
                    self.w.all,
                    -7,
                    -8,
                    -9,
                    RANK[7],
                    RANK[1],
                    !FILE_A,
                    !FILE_H,
                )
            };

        let single = signed_shift(pawns, forward_offset) & empty;
        let double = signed_shift(signed_shift(pawns & start_rank, forward_offset) & empty, forward_offset) & empty;
        let left   = signed_shift(pawns & mask_left, left_offset) & enemy;
        let right  = signed_shift(pawns & mask_right, right_offset) & enemy;

        // Handle promotions separately
        let promo_push  = single & promo_rank;
        let promo_left  = left & promo_rank;
        let promo_right = right & promo_rank;

        let non_promo_push  = single & !promo_rank;
        let non_promo_left  = left & !promo_rank;
        let non_promo_right = right & !promo_rank;

        // En passant
        let ep_left = signed_shift(pawns & mask_left, left_offset) & en_passant_bb;
        let ep_right = signed_shift(pawns & mask_right, right_offset) & en_passant_bb;

        Self::add_pawn_moves(moves, non_promo_push,  forward_offset,     false, false);
        Self::add_pawn_moves(moves, double,          2 * forward_offset, false, false);
        Self::add_pawn_moves(moves, non_promo_left,  left_offset,        false, false);
        Self::add_pawn_moves(moves, non_promo_right, right_offset,       false, false);

        Self::add_pawn_moves(moves, promo_push,  forward_offset, true, false);
        Self::add_pawn_moves(moves, promo_left,  left_offset,    true, false);
        Self::add_pawn_moves(moves, promo_right, right_offset,   true, false);

        Self::add_pawn_moves(moves, ep_left,  left_offset,  false, true);
        Self::add_pawn_moves(moves, ep_right, right_offset, false, true);
    }

    fn generate_pseudo_knight_moves(&self, moves: &mut Vec<Move>) {
        let (mut knights, friendly) = if self.whites_turn {
            (self.w.knights, self.w.all)
        } else {
            (self.b.knights, self.b.all)
        };

        while knights != 0 {
            let from = pop_lsb(&mut knights);
            let mut attacks = KNIGHT_ATTACKS[from as usize] & !friendly;
            while attacks != 0 {
                let to = pop_lsb(&mut attacks);
                moves.push(Move {
                    from: from as u8,
                    to: to as u8,
                    piece: Piece::Knight,
                    promotion: None,
                    en_passant: false,
                });
            }
        }
    }

    fn generate_pseudo_bishop_moves(&self, moves: &mut Vec<Move>) {
        let (mut bishops, friendly) = if self.whites_turn {
            (self.w.bishops, self.w.all)
        } else {
            (self.b.bishops, self.b.all)
        };

        while bishops != 0 {
            let from = pop_lsb(&mut bishops) as usize;
            let mask = BISHOP_MASKS[from];
            let magic = BISHOP_MAGICS[from];
            let shift = BISHOP_MAGICS_SHIFT[from];
            let blockers = self.occupied & mask;
            let hash = (blockers.wrapping_mul(magic) >> shift) as usize;
            let mut attacks = BISHOP_ATTACK_TABLES[from][hash] & !friendly;
            while attacks != 0 {
                let to = pop_lsb(&mut attacks);
                println!("{}", to);
                moves.push(Move {
                    from: from as u8,
                    to: to as u8,
                    piece: Piece::Bishop,
                    promotion: None,
                    en_passant: false,
                });
            }
        }
    }

    fn generate_pseudo_rook_moves(&self, moves: &mut Vec<Move>) {
        let (mut rooks, friendly) = if self.whites_turn {
            (self.w.rooks, self.w.all)
        } else {
            (self.b.rooks, self.b.all)
        };

        while rooks != 0 {
            let from = pop_lsb(&mut rooks) as usize;
            let mask = ROOK_MASKS[from];
            let magic = ROOK_MAGICS[from];
            let shift = ROOK_MAGICS_SHIFT[from];
            let blockers = self.occupied & mask;
            let hash = (blockers.wrapping_mul(magic) >> shift) as usize;
            let mut attacks = ROOK_ATTACK_TABLES[from][hash] & !friendly;
            while attacks != 0 {
                let to = pop_lsb(&mut attacks);
                moves.push(Move {
                    from: from as u8,
                    to: to as u8,
                    piece: Piece::Rook,
                    promotion: None,
                    en_passant: false,
                });
            }
        }
    }

    fn generate_pseudo_queen_moves(&self, moves: &mut Vec<Move>) {
        let (mut queens, friendly) = if self.whites_turn {
            (self.w.queens, self.w.all)
        } else {
            (self.b.queens, self.b.all)
        };

        while queens != 0 {
            let from = pop_lsb(&mut queens) as usize;

            let mask = ROOK_MASKS[from];
            let blockers = self.occupied & mask;
            let magic = ROOK_MAGICS[from];
            let shift = ROOK_MAGICS_SHIFT[from];
            let hash = (blockers.wrapping_mul(magic) >> shift) as usize;
            let rook_attacks = ROOK_ATTACK_TABLES[from][hash] & !friendly;

            let mask = BISHOP_MASKS[from];
            let blockers = self.occupied & mask;
            let magic = BISHOP_MAGICS[from];
            let shift = BISHOP_MAGICS_SHIFT[from];
            let hash = (blockers.wrapping_mul(magic) >> shift) as usize;
            let bishop_attacks = BISHOP_ATTACK_TABLES[from][hash] & !friendly;

            let mut attacks = rook_attacks | bishop_attacks;

            while attacks != 0 {
                let to = pop_lsb(&mut attacks);
                moves.push(Move {
                    from: from as u8,
                    to: to as u8,
                    piece: Piece::Queen,
                    promotion: None,
                    en_passant: false,
                });
            }
        }
    }

    fn generate_pseudo_king_moves(&self, moves: &mut Vec<Move>) {
        let (mut king, friendly) = if self.whites_turn {
            (self.w.king, self.w.all)
        } else {
            (self.b.king, self.b.all)
        };

        while king != 0 {
            let from = pop_lsb(&mut king);
            let mut attacks = KING_ATTACKS[from as usize] & !friendly;
            while attacks != 0 {
                let to = pop_lsb(&mut attacks);
                moves.push(Move {
                    from: from as u8,
                    to: to as u8,
                    piece: Piece::King,
                    promotion: None,
                    en_passant: false,
                });
            }
        }
    }

    fn generate_pseudo_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        self.generate_pseudo_pawn_moves  (&mut moves);
        self.generate_pseudo_knight_moves(&mut moves);
        self.generate_pseudo_bishop_moves(&mut moves);
        self.generate_pseudo_rook_moves  (&mut moves);
        self.generate_pseudo_queen_moves (&mut moves);
        self.generate_pseudo_king_moves  (&mut moves);
        moves
    }
}


fn main() {
    let pos = Position::start();
    println!("{:?}", pos);

    let moves = pos.generate_pseudo_moves();
    for m in moves {
        println!(
            "Move: {:?} {} to {}",
            m.piece,
            square_idx_to_string(m.from),
            square_idx_to_string(m.to)
        );
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn sq_to_bb(lst: &[u8]) -> u64 {
        lst.iter().fold(0u64, |s, &a| s | bit(a.into()))
    }

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
        assert_eq!(pos.whites_turn, true);
    }

    #[test]
    fn pop_lsb_test() {
        let mut bb = 0b101010;
        assert_eq!(pop_lsb(&mut bb), 1);
        assert_eq!(bb, 0b101000);
        assert_eq!(pop_lsb(&mut bb), 3);
        assert_eq!(bb, 0b100000);
        assert_eq!(pop_lsb(&mut bb), 5);
        assert_eq!(bb, 0b000000);
    }

    #[test]
    fn square_idx_to_string_test() {
        assert_eq!(square_idx_to_string(0), "a1");
        assert_eq!(square_idx_to_string(7), "h1");
        assert_eq!(square_idx_to_string(56), "a8");
        assert_eq!(square_idx_to_string(63), "h8");
        assert_eq!(square_idx_to_string(27), "d4");
    }

    #[test]
    fn signed_shift_test() {
        assert_eq!(signed_shift(0b01100001, 1), 0b11000010);
        assert_eq!(signed_shift(0b01100001, -1), 0b00110000);
        assert_eq!(signed_shift(0b00000010, 2), 0b00001000);
        assert_eq!(signed_shift(0b00001000, -2), 0b00000010);
        assert_eq!(signed_shift(0b10000000, -7), 0b00000001);
    }

    #[test]
    fn pseudo_pawn_moves_start_position() {
        let pos = Position::start();
        let mut moves = Vec::new();
        pos.generate_pseudo_pawn_moves(&mut moves);

        let expected: HashSet<Move> = [
            Move { from: 8,  to: 16, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 9,  to: 17, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 10, to: 18, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 11, to: 19, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 12, to: 20, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 13, to: 21, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 14, to: 22, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 15, to: 23, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 8,  to: 24, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 9,  to: 25, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 10, to: 26, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 11, to: 27, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 12, to: 28, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 13, to: 29, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 14, to: 30, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 15, to: 31, piece: Piece::Pawn, promotion: None, en_passant: false },
        ].into();

        let moves_set: HashSet<Move> = moves.into_iter().collect();
        assert_eq!(moves_set, expected);
    }

    #[test]
    fn pseudo_pawn_moves_endgame() {
        let pos = Position::from_fen("8/p1pp3p/BN6/3R4/1k2K3/8/1p3pp1/2Q1B3 b - - 0 1");
        let mut moves = Vec::new();
        pos.generate_pseudo_pawn_moves(&mut moves);

        let expected: HashSet<Move> = [
            Move { from: 48, to: 41, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 50, to: 41, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 50, to: 42, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 50, to: 34, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 51, to: 43, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 55, to: 47, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 55, to: 39, piece: Piece::Pawn, promotion: None, en_passant: false },

            Move { from: 9,  to: 1, piece: Piece::Pawn, promotion: Some(Piece::Knight), en_passant: false },
            Move { from: 9,  to: 2, piece: Piece::Pawn, promotion: Some(Piece::Knight), en_passant: false },
            Move { from: 13, to: 5, piece: Piece::Pawn, promotion: Some(Piece::Knight), en_passant: false },
            Move { from: 13, to: 4, piece: Piece::Pawn, promotion: Some(Piece::Knight), en_passant: false },
            Move { from: 14, to: 6, piece: Piece::Pawn, promotion: Some(Piece::Knight), en_passant: false },

            Move { from: 9,  to: 1, piece: Piece::Pawn, promotion: Some(Piece::Bishop), en_passant: false },
            Move { from: 9,  to: 2, piece: Piece::Pawn, promotion: Some(Piece::Bishop), en_passant: false },
            Move { from: 13, to: 5, piece: Piece::Pawn, promotion: Some(Piece::Bishop), en_passant: false },
            Move { from: 13, to: 4, piece: Piece::Pawn, promotion: Some(Piece::Bishop), en_passant: false },
            Move { from: 14, to: 6, piece: Piece::Pawn, promotion: Some(Piece::Bishop), en_passant: false },

            Move { from: 9,  to: 1, piece: Piece::Pawn, promotion: Some(Piece::Rook), en_passant: false },
            Move { from: 9,  to: 2, piece: Piece::Pawn, promotion: Some(Piece::Rook), en_passant: false },
            Move { from: 13, to: 5, piece: Piece::Pawn, promotion: Some(Piece::Rook), en_passant: false },
            Move { from: 13, to: 4, piece: Piece::Pawn, promotion: Some(Piece::Rook), en_passant: false },
            Move { from: 14, to: 6, piece: Piece::Pawn, promotion: Some(Piece::Rook), en_passant: false },

            Move { from: 9,  to: 1, piece: Piece::Pawn, promotion: Some(Piece::Queen), en_passant: false },
            Move { from: 9,  to: 2, piece: Piece::Pawn, promotion: Some(Piece::Queen), en_passant: false },
            Move { from: 13, to: 5, piece: Piece::Pawn, promotion: Some(Piece::Queen), en_passant: false },
            Move { from: 13, to: 4, piece: Piece::Pawn, promotion: Some(Piece::Queen), en_passant: false },
            Move { from: 14, to: 6, piece: Piece::Pawn, promotion: Some(Piece::Queen), en_passant: false }
        ].into();

        let moves_set: HashSet<Move> = moves.into_iter().collect();
        assert_eq!(moves_set, expected);
    }

    #[test]
    fn pseudo_pawn_moves_en_passant() {
        let pos = Position::from_fen("8/6k1/8/1pP1Pp2/8/8/5K2/8 w - b6 0 1");
        let mut moves = Vec::new();
        pos.generate_pseudo_pawn_moves(&mut moves);

        let expected: HashSet<Move> = [
            Move { from: 34, to: 42, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 34, to: 41, piece: Piece::Pawn, promotion: None, en_passant: true },
            Move { from: 36, to: 44, piece: Piece::Pawn, promotion: None, en_passant: false },
        ].into();

        let moves_set: HashSet<Move> = moves.into_iter().collect();
        assert_eq!(moves_set, expected);
    }

    #[test]
    fn knight_attack_table() {
        // See https://www.chessprogramming.org/File:Lerf.JPG
        assert_eq!(KNIGHT_ATTACKS[0],  sq_to_bb(&[10, 17]));
        assert_eq!(KNIGHT_ATTACKS[1],  sq_to_bb(&[11, 16, 18]));
        assert_eq!(KNIGHT_ATTACKS[8],  sq_to_bb(&[2, 18, 25]));

        assert_eq!(KNIGHT_ATTACKS[6],  sq_to_bb(&[12, 21, 23]));
        assert_eq!(KNIGHT_ATTACKS[7],  sq_to_bb(&[13, 22]));
        assert_eq!(KNIGHT_ATTACKS[15], sq_to_bb(&[5, 21, 30]));

        assert_eq!(KNIGHT_ATTACKS[48], sq_to_bb(&[33, 42, 58]));
        assert_eq!(KNIGHT_ATTACKS[56], sq_to_bb(&[41, 50]));
        assert_eq!(KNIGHT_ATTACKS[57], sq_to_bb(&[40, 42, 51]));

        assert_eq!(KNIGHT_ATTACKS[55], sq_to_bb(&[38, 45, 61]));
        assert_eq!(KNIGHT_ATTACKS[62], sq_to_bb(&[45, 47, 52]));
        assert_eq!(KNIGHT_ATTACKS[63], sq_to_bb(&[46, 53]));

        assert_eq!(KNIGHT_ATTACKS[11], sq_to_bb(&[1, 5, 17, 21, 26, 28]));
        assert_eq!(KNIGHT_ATTACKS[25], sq_to_bb(&[8, 10, 19, 35, 40, 42]));
        assert_eq!(KNIGHT_ATTACKS[36], sq_to_bb(&[19, 21, 26, 30, 42, 46, 51, 53]));
    }

    #[test]
    fn pseudo_knight_moves_start_position() {
        let pos = Position::start();
        let mut moves = Vec::new();
        pos.generate_pseudo_knight_moves(&mut moves);

        let expected: HashSet<Move> = [
            Move { from: 1, to: 16, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 1, to: 18, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 6, to: 21, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 6, to: 23, piece: Piece::Knight, promotion: None, en_passant: false },
        ].into();

        let moves_set: HashSet<Move> = moves.into_iter().collect();
        assert_eq!(moves_set, expected);
    }

    #[test]
    fn pseudo_knight_moves_endgame() {
        let pos = Position::from_fen("8/3nk3/1N3R2/3n2n1/3N4/8/3K1N2/1r6 w - - 0 1");
        let mut moves = Vec::new();
        pos.generate_pseudo_knight_moves(&mut moves);

        let expected: HashSet<Move> = [
            Move { from: 13, to:  3, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 13, to:  7, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 13, to: 19, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 13, to: 23, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 13, to: 28, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 13, to: 30, piece: Piece::Knight, promotion: None, en_passant: false },

            Move { from: 27, to: 10, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 27, to: 12, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 27, to: 17, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 27, to: 21, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 27, to: 33, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 27, to: 37, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 27, to: 42, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 27, to: 44, piece: Piece::Knight, promotion: None, en_passant: false },

            Move { from: 41, to: 24, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 41, to: 26, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 41, to: 35, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 41, to: 51, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 41, to: 56, piece: Piece::Knight, promotion: None, en_passant: false },
            Move { from: 41, to: 58, piece: Piece::Knight, promotion: None, en_passant: false },
        ].into();

        let moves_set: HashSet<Move> = moves.into_iter().collect();
        assert_eq!(moves_set, expected);
    }

    #[test]
    fn king_attack_table() {
        // See https://www.chessprogramming.org/File:Lerf.JPG
        assert_eq!(KING_ATTACKS[0],  sq_to_bb(&[1, 8, 9]));
        assert_eq!(KING_ATTACKS[7],  sq_to_bb(&[6, 14, 15])); 
        assert_eq!(KING_ATTACKS[56], sq_to_bb(&[48, 49, 57]));
        assert_eq!(KING_ATTACKS[63], sq_to_bb(&[54, 55, 62]));

        assert_eq!(KING_ATTACKS[3],  sq_to_bb(&[2, 4, 10, 11, 12]));
        assert_eq!(KING_ATTACKS[16], sq_to_bb(&[8, 9, 17, 24, 25]));
        assert_eq!(KING_ATTACKS[39], sq_to_bb(&[30, 31, 38, 46, 47]));
        assert_eq!(KING_ATTACKS[58], sq_to_bb(&[49, 50, 51, 57, 59]));

        assert_eq!(KING_ATTACKS[42], sq_to_bb(&[33, 34, 35, 41, 43, 49, 50, 51]));
        assert_eq!(KING_ATTACKS[19], sq_to_bb(&[10, 11, 12, 18, 20, 26, 27, 28]));
    }

    #[test]
    fn pseudo_king_moves_start_position() {
        let pos = Position::start();
        let mut moves = Vec::new();
        pos.generate_pseudo_king_moves(&mut moves);
        assert_eq!(moves.len(), 0);
    }

    #[test]
    fn pseudo_king_moves_endgame() {
        let pos = Position::from_fen("8/8/8/8/7P/6K1/1r6/k7 w - - 0 1");
        let mut moves = Vec::new();
        pos.generate_pseudo_king_moves(&mut moves);

        let expected: HashSet<Move> = [
            Move { from: 22, to: 13, piece: Piece::King, promotion: None, en_passant: false },
            Move { from: 22, to: 14, piece: Piece::King, promotion: None, en_passant: false },
            Move { from: 22, to: 15, piece: Piece::King, promotion: None, en_passant: false },
            Move { from: 22, to: 21, piece: Piece::King, promotion: None, en_passant: false },
            Move { from: 22, to: 23, piece: Piece::King, promotion: None, en_passant: false },
            Move { from: 22, to: 29, piece: Piece::King, promotion: None, en_passant: false },
            Move { from: 22, to: 30, piece: Piece::King, promotion: None, en_passant: false },
        ].into();

        let moves_set: HashSet<Move> = moves.into_iter().collect();
        assert_eq!(moves_set, expected);
    }

    #[test]
    fn rook_masks() {
        assert_eq!(ROOK_MASKS[0],  (FILE_A | RANK[1]) & !sq_to_bb(&[0, 7, 56]));
        assert_eq!(ROOK_MASKS[3],  (FILE_D | RANK[1]) & !sq_to_bb(&[3, 0, 7, 59]));
        assert_eq!(ROOK_MASKS[9],  (FILE_B | RANK[2]) & !sq_to_bb(&[9, 8, 15, 1, 57]));
        assert_eq!(ROOK_MASKS[19], (FILE_D | RANK[3]) & !sq_to_bb(&[19, 16, 23, 3, 59]));
        assert_eq!(ROOK_MASKS[24], (FILE_A | RANK[4]) & !sq_to_bb(&[24, 0, 56, 31]));
        assert_eq!(ROOK_MASKS[38], (FILE_G | RANK[5]) & !sq_to_bb(&[38, 32, 39, 6, 62]));
        assert_eq!(ROOK_MASKS[55], (FILE_H | RANK[7]) & !sq_to_bb(&[55, 48, 7, 63]));
    }

    #[test]
    fn bishop_masks() {
        assert_eq!(BISHOP_MASKS[0],  sq_to_bb(&[9, 18, 27, 36, 45, 54]));
        assert_eq!(BISHOP_MASKS[3],  sq_to_bb(&[10, 17, 12, 21, 30]));
        assert_eq!(BISHOP_MASKS[13], sq_to_bb(&[20, 27, 34, 41, 22]));
        assert_eq!(BISHOP_MASKS[24], sq_to_bb(&[17, 10, 33, 42, 51]));
        assert_eq!(BISHOP_MASKS[38], sq_to_bb(&[29, 20, 11, 45, 52]));
        assert_eq!(BISHOP_MASKS[55], sq_to_bb(&[46, 37, 28, 19, 10]));
        assert_eq!(BISHOP_MASKS[56], sq_to_bb(&[49, 42, 35, 28, 21, 14]));
    }

    #[test]
    fn pseudo_rook_moves_start_position() {
        let pos = Position::start();
        let mut moves = Vec::new();
        pos.generate_pseudo_rook_moves(&mut moves);
        assert_eq!(moves.len(), 0);
    }

    #[test]
    fn pseudo_rook_moves_endgame() {
        let pos = Position::from_fen("8/3k4/8/R3p3/6P1/1P6/3K2R1/8 w - - 0 1");
        let mut moves = Vec::new();
        pos.generate_pseudo_rook_moves(&mut moves);

        let expected: HashSet<Move> = [
            Move { from: 32, to: 40, piece: Piece::Rook, promotion: None, en_passant: false },
            Move { from: 32, to: 48, piece: Piece::Rook, promotion: None, en_passant: false },
            Move { from: 32, to: 56, piece: Piece::Rook, promotion: None, en_passant: false },
            Move { from: 32, to: 24, piece: Piece::Rook, promotion: None, en_passant: false },
            Move { from: 32, to: 16, piece: Piece::Rook, promotion: None, en_passant: false },
            Move { from: 32, to: 8,  piece: Piece::Rook, promotion: None, en_passant: false },
            Move { from: 32, to: 0,  piece: Piece::Rook, promotion: None, en_passant: false },
            Move { from: 32, to: 33, piece: Piece::Rook, promotion: None, en_passant: false },
            Move { from: 32, to: 34, piece: Piece::Rook, promotion: None, en_passant: false },
            Move { from: 32, to: 35, piece: Piece::Rook, promotion: None, en_passant: false },
            Move { from: 32, to: 36, piece: Piece::Rook, promotion: None, en_passant: false },

            Move { from: 14, to: 6,  piece: Piece::Rook, promotion: None, en_passant: false },
            Move { from: 14, to: 22, piece: Piece::Rook, promotion: None, en_passant: false },
            Move { from: 14, to: 15, piece: Piece::Rook, promotion: None, en_passant: false },
            Move { from: 14, to: 13, piece: Piece::Rook, promotion: None, en_passant: false },
            Move { from: 14, to: 12, piece: Piece::Rook, promotion: None, en_passant: false },
        ].into();

        let moves_set: HashSet<Move> = moves.into_iter().collect();
        assert_eq!(moves_set, expected);
    }

    #[test]
    fn pseudo_bishop_moves_start_position() {
        let pos = Position::start();
        let mut moves = Vec::new();
        pos.generate_pseudo_bishop_moves(&mut moves);
        assert_eq!(moves.len(), 0);
    }

    #[test]
    fn pseudo_bishop_moves_endgame() {
        let pos = Position::from_fen("8/8/8/3b4/5P1b/1k6/3b3K/b7 b - - 0 1");
        let mut moves = Vec::new();
        pos.generate_pseudo_bishop_moves(&mut moves);

        let expected: HashSet<Move> = [
            Move { from: 0,  to: 9,  piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 0,  to: 18, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 0,  to: 27, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 0,  to: 36, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 0,  to: 45, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 0,  to: 54, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 0,  to: 63, piece: Piece::Bishop, promotion: None, en_passant: false },

            Move { from: 11, to: 2,  piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 11, to: 4,  piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 11, to: 18, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 11, to: 25, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 11, to: 32, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 11, to: 20, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 11, to: 29, piece: Piece::Bishop, promotion: None, en_passant: false },

            Move { from: 31, to: 22, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 31, to: 13, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 31, to: 4,  piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 31, to: 38, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 31, to: 45, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 31, to: 52, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 31, to: 59, piece: Piece::Bishop, promotion: None, en_passant: false },

            Move { from: 35, to: 26, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 35, to: 44, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 35, to: 53, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 35, to: 62, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 35, to: 42, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 35, to: 49, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 35, to: 56, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 35, to: 28, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 35, to: 21, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 35, to: 14, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 35, to: 7,  piece: Piece::Bishop, promotion: None, en_passant: false },
        ].into();

        let moves_set: HashSet<Move> = moves.into_iter().collect();
        assert_eq!(moves_set, expected);
    }

    #[test]
    fn pseudo_bishop_moves_blocking_friendly() {
        let pos = Position::from_fen("1k3K2/8/1P3P2/8/3B4/8/1P3P2/8 w - - 0 1");
        let mut moves = Vec::new();
        pos.generate_pseudo_bishop_moves(&mut moves);

        let expected: HashSet<Move> = [
            Move { from: 27, to: 18, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 27, to: 20, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 27, to: 34, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 27, to: 36, piece: Piece::Bishop, promotion: None, en_passant: false },
        ].into();

        let moves_set: HashSet<Move> = moves.into_iter().collect();
        assert_eq!(moves_set, expected);
    }

    #[test]
    fn pseudo_bishop_moves_blocking_hostile() {
        let pos = Position::from_fen("1k3K2/8/1p3p2/8/3B4/8/1p3p2/8 w - - 0 1");
        let mut moves = Vec::new();
        pos.generate_pseudo_bishop_moves(&mut moves);

        let expected: HashSet<Move> = [
            Move { from: 27, to: 18, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 27, to: 9,  piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 27, to: 20, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 27, to: 13, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 27, to: 34, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 27, to: 41, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 27, to: 36, piece: Piece::Bishop, promotion: None, en_passant: false },
            Move { from: 27, to: 45, piece: Piece::Bishop, promotion: None, en_passant: false },
        ].into();

        let moves_set: HashSet<Move> = moves.into_iter().collect();
        assert_eq!(moves_set, expected);
    }

    #[test]
    fn pseudo_queen_moves_start_position() {
        let pos = Position::start();
        let mut moves = Vec::new();
        pos.generate_pseudo_queen_moves(&mut moves);
        assert_eq!(moves.len(), 0);
    }

    #[test]
    fn pseudo_queen_moves_endgame() {
        let pos = Position::from_fen("8/k3b3/2r5/8/4Q1N1/8/2K5/8 w - - 0 1");
        let mut moves = Vec::new();
        pos.generate_pseudo_queen_moves(&mut moves);

        let expected: HashSet<Move> = [
            Move { from: 28, to: 4,  piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 7,  piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 12, piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 14, piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 19, piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 20, piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 21, piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 24, piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 25, piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 26, piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 27, piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 29, piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 35, piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 36, piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 37, piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 42, piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 44, piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 46, piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 52, piece: Piece::Queen, promotion: None, en_passant: false },
            Move { from: 28, to: 55, piece: Piece::Queen, promotion: None, en_passant: false },
        ].into();

        println!("gen");
        for m in &moves {
            println!("{:?}", m);
        }
        println!("exp");
        for m in &expected {
            println!("{:?}", m);
        }

        let moves_set: HashSet<Move> = moves.into_iter().collect();
        assert_eq!(moves_set, expected);
    }
}

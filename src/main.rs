fn pop_lsb(bitboard: &mut u64) -> u8 {
    let result = bitboard.trailing_zeros() as u8;
    *bitboard &= *bitboard - 1;
    result
}

fn square_idx_to_string(sq: u8) -> String {
    let file = (sq % 8) as u8;
    let rank = (sq / 8) as u8;
    format!("{}{}", (file + b'a') as char, rank + 1)
}

fn square_string_to_idx(sq: &str) -> Option<u8> {
    if sq.len() != 2 {
        return None;
    }
    let file = sq.chars().nth(0).unwrap() as u8 - b'a';
    let rank = sq.chars().nth(1).unwrap().to_digit(10).unwrap() as u8 - 1;
    if file > 7 || rank > 7 {
        return None;
    }
    Some(rank * 8 + file)
}

fn signed_shift(bb: u64, offset: i8) -> u64 {
    if offset >= 0 {
        bb << offset
    } else {
        bb >> (-offset)
    }
}

fn print_bitboard(bb: u64) {
    for rank in (0..8).rev() {
        print!("{} ", rank + 1);
        for file in 0..8 {
            let square = 1u64 << (rank * 8 + file);
            if bb & square != 0 {
                print!("+ ")
            } else {
                print!(". ")
            }
        }
        println!();
    }
    println!("  a b c d e f g h");
}

const RANK: [u64; 8+1] = [
    0,                  // Rank 0 (unused, for convenience)
    0x00000000000000FF, // Rank 1
    0x000000000000FF00, // Rank 2
    0x0000000000FF0000, // Rank 3
    0x00000000FF000000, // Rank 4
    0x000000FF00000000, // Rank 5
    0x0000FF0000000000, // Rank 6
    0x00FF000000000000, // Rank 7
    0xFF00000000000000, // Rank 8
];

const FILE_A: u64 = 0x0101010101010101; // 0b00000001...
const FILE_B: u64 = 0x0202020202020202; // 0b00000010...
const FILE_C: u64 = 0x0404040404040404; // 0b00000100...
const FILE_D: u64 = 0x0808080808080808; // 0b00001000...
const FILE_E: u64 = 0x1010101010101010; // 0b00010000...
const FILE_F: u64 = 0x2020202020202020; // 0b00100000...
const FILE_G: u64 = 0x4040404040404040; // 0b01000000...
const FILE_H: u64 = 0x8080808080808080; // 0b10000000...

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
        unimplemented!();
    }

    fn generate_pseudo_bishop_moves(&self, moves: &mut Vec<Move>) {
        unimplemented!();
    }

    fn generate_pseudo_rook_moves(&self, moves: &mut Vec<Move>) {
        unimplemented!();
    }

    fn generate_pseudo_queen_moves(&self, moves: &mut Vec<Move>) {
        unimplemented!();
    }

    fn generate_pseudo_king_moves(&self, moves: &mut Vec<Move>) {
        unimplemented!();
    }

    fn generate_pseudo_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        self.generate_pseudo_pawn_moves  (&mut moves);
        // self.generate_pseudo_knight_moves(&mut moves);
        // self.generate_pseudo_bishop_moves(&mut moves);
        // self.generate_pseudo_rook_moves  (&mut moves);
        // self.generate_pseudo_queen_moves (&mut moves);
        // self.generate_pseudo_king_moves  (&mut moves);
        moves
    }
}


fn main() {
    let pos = Position::start();
    println!("{:?}", pos);

    let moves = pos.generate_pseudo_moves();
    for m in moves {
        println!(
            "Move: {} to {}", 
            square_idx_to_string(m.from), 
            square_idx_to_string(m.to)
        );
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

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
        let pos = Position::from_fen("8/6k1/8/1pP5/8/8/5K2/8 w - b6 0 1");
        let mut moves = Vec::new();
        pos.generate_pseudo_pawn_moves(&mut moves);

        let expected: HashSet<Move> = [
            Move { from: 34, to: 42, piece: Piece::Pawn, promotion: None, en_passant: false },
            Move { from: 34, to: 41, piece: Piece::Pawn, promotion: None, en_passant: true },
        ].into();

        let moves_set: HashSet<Move> = moves.into_iter().collect();
        assert_eq!(moves_set, expected);
    }
}

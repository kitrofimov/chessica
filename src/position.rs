use crate::utility::*;
use crate::constants::*;
use crate::movegen::*;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
#[repr(u8)]
pub enum Piece {
    Pawn = 0,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}


pub type Bitboard = u64;

trait BitOps {
    fn set_bit(self, n: u8) -> Self;
    fn unset_bit(self, n: u8) -> Self;
}

impl BitOps for Bitboard {
    fn set_bit(self, n: u8) -> Self {
        self | (1 << n)
    }

    fn unset_bit(self, n: u8) -> Self {
        self & !(1 << n)
    }
}


#[derive(Default, Copy, Clone)]
pub struct BitboardSet {
    pub all:     Bitboard,
    pub pawns:   Bitboard,
    pub knights: Bitboard,
    pub bishops: Bitboard,
    pub rooks:   Bitboard,
    pub queens:  Bitboard,
    pub king:    Bitboard,  // only one king per side, so not plural :)
}

impl BitboardSet {
    fn update(&mut self) {
        self.all = self.pawns | self.knights | self.bishops |
                   self.rooks | self.queens  | self.king;
    }

    fn piece_to_bb(&mut self, piece: Piece) -> &mut Bitboard {
        match piece {
            Piece::Knight => &mut self.knights,
            Piece::Bishop => &mut self.bishops,
            Piece::Rook   => &mut self.rooks,
            Piece::Queen  => &mut self.queens,
            Piece::King   => &mut self.king,
            Piece::Pawn   => &mut self.pawns,
        }
    }

    fn unset_bit(&mut self, n: u8) {
        self.all     = self.all.unset_bit(n);
        self.pawns   = self.pawns.unset_bit(n);
        self.knights = self.knights.unset_bit(n);
        self.bishops = self.bishops.unset_bit(n);
        self.rooks   = self.rooks.unset_bit(n);
        self.queens  = self.queens.unset_bit(n);
        self.king    = self.king.unset_bit(n);
    }
}


#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Move {
    pub from: u8,
    pub to: u8,
    pub piece: Piece,
    pub capture: bool,
    pub promotion: Option<Piece>,
    pub en_passant: bool,
}

impl Move {
    pub fn new(from: u8, to: u8, piece: Piece, capture: bool) -> Self {
        Move {
            from,
            to,
            piece,
            capture,
            promotion: None,
            en_passant: false
        }
    }

    pub fn pawn(from: u8, to: u8, capture: bool, promotion: Option<Piece>, en_passant: bool) -> Self {
        Move {
            from,
            to,
            piece: Piece::Pawn,
            capture,
            promotion,
            en_passant,
        }
    }
}


#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CastlingRights {
    white_kingside: bool,
    white_queenside: bool,
    black_kingside: bool,
    black_queenside: bool,
}

impl Default for CastlingRights {
    fn default() -> Self {
        CastlingRights {
            white_kingside: true,
            white_queenside: true,
            black_kingside: true,
            black_queenside: true
        }
    }
}

impl ToString for CastlingRights {
    fn to_string(&self) -> String {
        let mut s = String::new();
        if self.white_kingside {
            s += "K";
        }
        if self.white_queenside {
            s += "Q";
        }
        if self.black_kingside {
            s += "k";
        }
        if self.black_queenside {
            s += "q";
        }
        s
    }
}

impl CastlingRights {
    fn no() -> Self {
        CastlingRights {
            white_kingside: false,
            white_queenside: false,
            black_kingside: false,
            black_queenside: false
        }
    }

    fn from_string(s: &str) -> Self {
        let mut rights = CastlingRights {
            white_kingside: false,
            white_queenside: false,
            black_kingside: false,
            black_queenside: false,
        };
        if s.contains("K") {
            rights.white_kingside = true;
        }
        if s.contains("Q") {
            rights.white_queenside = true;
        }
        if s.contains("k") {
            rights.black_kingside = true;
        }
        if s.contains("q") {
            rights.black_queenside = true;
        }
        rights
    }
}


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Player {
    White, Black
}


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
        writeln!(f, "Turn: {:?}", self.player_to_move)
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
                queens:  0x0000000000000010,
                king:    0x0000000000000008,
            },
            b: BitboardSet {
                all:     0xFFFF000000000000,
                pawns:   0x00FF000000000000,
                knights: 0x4200000000000000,
                bishops: 0x2400000000000000,
                rooks:   0x8100000000000000,
                queens:  0x1000000000000000,
                king:    0x0800000000000000,
            },
            occupied: 0xFFFF00000000FFFF,
            player_to_move: Player::White,
            en_passant_square: None,
            castling: CastlingRights::default(),
        }
    }

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

    pub fn make_move(&mut self, m: &Move) {
        let (friendly, hostile, kingside, queenside) = match self.player_to_move {
            Player::White => (
                &mut self.w, &mut self.b,
                &mut self.castling.white_kingside,&mut self.castling.white_queenside
            ),
            Player::Black => (
                &mut self.b, &mut self.w,
                &mut self.castling.black_kingside, &mut self.castling.black_queenside
            ),
        };

        let bb = friendly.piece_to_bb(m.piece);

        if m.piece == Piece::King {
            self.castling = CastlingRights::no()
        } else if m.piece == Piece::Rook && !(*kingside == false && *queenside == false) {
            if *bb & bit(0) > 0 {  // a1
                *kingside = false;
            } else if *bb & bit(7) > 0 {  // h1
                *queenside = false
            }
        }

        *bb = bb.unset_bit(m.from).set_bit(m.to);

        if m.capture {
            hostile.unset_bit(m.to);
        }

        self.update();
        self.player_to_move = match self.player_to_move {
            Player::White => Player::Black,
            Player::Black => Player::White,
        };
    }

    fn update(&mut self) {
        self.w.update();
        self.b.update();
        self.occupied = self.w.all | self.b.all;
    }

    pub fn is_square_attacked(&self, sq: usize, by_player: Player) -> bool {
        let friend = match by_player {
            Player::White => &self.w,
            Player::Black => &self.b,
        };

        // All the possible pieces' positions, which could attack this square
        let pawn = match by_player {
            Player::White => PAWN_ATTACKS_BLACK[sq],  // reversing intentionally, questioning:
            Player::Black => PAWN_ATTACKS_WHITE[sq],  // "what could have attacked this square?"
        };
        let knight = knight_attacks(self, sq, 0x0);
        let bishop = bishop_attacks(self, sq, 0x0);
        let rook = rook_attacks(self, sq, 0x0);
        let queen = queen_attacks(self, sq, 0x0);
        let king = king_attacks(self, sq, 0x0);

        if pawn & friend.pawns > 0 || knight & friend.knights > 0 ||
           bishop & friend.bishops > 0 || rook & friend.rooks > 0 ||
           queen & friend.queens > 0 || king & friend.king > 0 {
            return true;
        }
        false
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

    #[test]
    fn make_move_knight() {
        let mut pos = Position::from_fen("8/1k6/3r4/8/4N3/8/1K6/8 w - - 0 1");
        let m = Move::new(28, 43, Piece::Knight, true);
        pos.make_move(&m);
        assert_eq!(pos.w.king, bit(9));
        assert_eq!(pos.w.knights, bit(43));
        assert_eq!(pos.w.all, bit(9) | bit(43));

        assert_eq!(pos.b.king, bit(49));
        assert_eq!(pos.b.rooks, 0x0);
        assert_eq!(pos.b.all, bit(49));

        assert_eq!(pos.occupied, bit(9) | bit(43) | bit(49));
    }

    #[test]
    fn make_move_rook() {
        let mut pos = Position::from_fen("8/8/8/5r2/8/1k6/5Q2/1K6 b - - 0 1");
        let m = Move::new(37, 13, Piece::Rook, true);
        pos.make_move(&m);
        assert_eq!(pos.w.king, bit(1));
        assert_eq!(pos.w.queens, 0x0);
        assert_eq!(pos.w.all, bit(1));

        assert_eq!(pos.b.king, bit(17));
        assert_eq!(pos.b.rooks, bit(13));
        assert_eq!(pos.b.all, bit(13) | bit(17));
    }

    #[test]
    fn make_move_king() {
        let mut pos = Position::from_fen("8/5kq1/1R6/8/3K4/8/8/8 w - - 0 1");
        let m = Move::new(27, 35, Piece::King, false);
        pos.make_move(&m);
        assert_eq!(pos.w.rooks, bit(41));
        assert_eq!(pos.w.king, bit(35));
        assert_eq!(pos.w.all, bit(35) | bit(41));

        assert_eq!(pos.b.king, bit(53));
        assert_eq!(pos.b.queens, bit(54));
        assert_eq!(pos.b.all, bit(53) | bit(54));
    }

    #[test]
    fn make_move_bishop() {
        let mut pos = Position::from_fen("8/2k5/8/4K3/1r6/8/3B4/8 w - - 0 1");
        let m = Move::new(11, 25, Piece::Bishop, true);
        pos.make_move(&m);
        assert_eq!(pos.w.king, bit(36));
        assert_eq!(pos.w.bishops, bit(25));
        assert_eq!(pos.w.all, bit(25) | bit(36));

        assert_eq!(pos.b.king, bit(50));
        assert_eq!(pos.b.rooks, 0x0);
        assert_eq!(pos.b.all, bit(50));
    }

    #[test]
    fn make_move_queen() {
        let mut pos = Position::from_fen("8/8/1kq5/8/5K2/2R5/8/8 b - - 0 1");
        let m = Move::new(42, 18, Piece::Queen, true);
        pos.make_move(&m);
        assert_eq!(pos.w.king, bit(29));
        assert_eq!(pos.w.rooks, 0x0);
        assert_eq!(pos.w.all, bit(29));

        assert_eq!(pos.b.king, bit(41));
        assert_eq!(pos.b.queens, bit(18));
        assert_eq!(pos.b.all, bit(18) | bit(41));
    }

    #[test]
    fn is_square_attacked_endgame() {
        let pos = Position::from_fen("8/3r1k2/8/4N3/1Q5q/8/2K5/8 b - - 0 1");
        assert_eq!(pos.is_square_attacked(53, Player::White), true);
        assert_eq!(pos.is_square_attacked(51, Player::White), true);
        assert_eq!(pos.is_square_attacked(20, Player::White), false);
        assert_eq!(pos.is_square_attacked(25, Player::Black), true);
        assert_eq!(pos.is_square_attacked(52, Player::Black), true);
        assert_eq!(pos.is_square_attacked(10, Player::Black), false);
    }
}

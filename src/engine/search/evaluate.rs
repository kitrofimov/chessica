use crate::constants::{evaluation::*, board::*};
use crate::utility::*;
use crate::engine::{
    base::{player::Player, piece::Piece},
    board::position::Position,
};

pub type Evaluation = i32;

/// Evaluate the given position and return a score from white's perspective
impl Position {
    pub fn evaluate(&self) -> i32 {
        let mut score = 0;

        score += self.eval_material();
        score += self.eval_piece_square_tables();
        score += self.eval_mobility();
        score += self.eval_pawn_structure();
        score += self.eval_king_safety();
        score += self.eval_space_control();

        if self.player_to_move == Player::White {
            score
        } else {
            -score
        }
    }

    /// Returns a value in range [0; 24], where 0 is "true endgame" and 24 is "true midgame"
    fn game_phase(&self) -> i32 {
        let mut phase = 0;

        phase += self.w.count(Piece::Bishop);
        phase += self.b.count(Piece::Bishop);
        phase += self.w.count(Piece::Knight);
        phase += self.b.count(Piece::Knight);

        phase += 2 * self.w.count(Piece::Rook);
        phase += 2 * self.b.count(Piece::Rook);

        phase += 4 * self.w.count(Piece::Queen);
        phase += 4 * self.b.count(Piece::Queen);

        phase as i32
    }

    /// Linearly interpolate between `mid` and `end` based on the game's current phase
    fn interpolate_phase(&self, mid: i32, end: i32) -> i32 {
        let phase_max = 24;
        let phase = self.game_phase();
        (mid * phase + end * (phase_max - phase)) / phase_max
    }

    fn get_pst_index(&self, sq: u8, player: Player) -> usize {
        if player == Player::White {
            sq as usize
        } else {  // Reversing the PST for black
            (sq ^ 56) as usize
        }
    }

    fn eval_material(&self) -> i32 {
        let mut material = 0;
        for piece in Piece::all_variants() {
            material += piece.value() * self.w.count(piece) as i32;
            material -= piece.value() * self.b.count(piece) as i32;
        }
        material
    }

    fn eval_piece_square_tables(&self) -> i32 {
        let mut score = 0;
        for square in 0..64 {
            if let Some((player, piece)) = self.piece_at(square) {
                let pst_index = self.get_pst_index(square, player);

                let value = match piece {
                    Piece::Pawn   => PAWN_PST  [pst_index],
                    Piece::Knight => KNIGHT_PST[pst_index],
                    Piece::Bishop => BISHOP_PST[pst_index],
                    Piece::Rook   => ROOK_PST  [pst_index],
                    Piece::Queen  => QUEEN_PST [pst_index],
                    Piece::King   => {
                        let endgame = KING_PST_ENDGAME[pst_index];
                        let midgame = KING_PST_MIDDLEGAME[pst_index];
                        self.interpolate_phase(midgame, endgame)
                    }
                };

                if player == Player::White {
                    score += value;
                } else {
                    score -= value;
                }
            }
        }
        score
    }

    fn eval_mobility(&self) -> i32 {
        let mut score: i32 = 0;
        for square in 0..64 {
            if let Some((player, piece)) = self.piece_at(square) {
                if piece == Piece::Pawn || piece == Piece::King {
                    continue;
                }
                // TODO: count not pseudomoves, but legal moves here?
                let value = self.count_n_pseudo_moves(square, player, piece) as i32;
                if player == Player::White {
                    score += value;
                } else {
                    score -= value;
                }
            }
        }
        score
    }

    /// For a pawn on (file, rank), computre squares ahead on same file and adjacent files
    fn pawn_forward_mask(&self, file: u8, rank: u8, player: Player) -> u64 {
        assert!(rank != 0);
        assert!(rank != 7);

        let mut mask = 0u64;
        let dir: i8 = if player == Player::White { 1 } else { -1 };

        let mut r = rank as i8 + dir;
        while 0 <= r && r <= 7 {
            for df in [-1, 0, 1] {
                let f = file as i8 + df;
                if 0 <= f && f <= 7 {
                    mask |= bit(coord_to_sq(f as u8, r as u8))
                }
            }
            r += dir;
        }

        mask
    }

    // Returns a signed value, so you should just sum it up with your existing score
    fn eval_doubled_and_isolated_pawns(&self, player: Player) -> i32 {
        let pawns_bb = self.perspective(player).0.pawns;
        let mut score = 0;
        for f in 0u8..8 {
            let mask = FILE[f as usize];
            let cnt = (pawns_bb & mask).count_ones() as i32;
            if cnt > 1 {
                score -= (cnt-1) * PAWN_DOUBLED_PENALTY;
            }

            if cnt > 0 {
                // isolated: no friendly pawn on adjacent files
                let adj_mask = match f {
                    0 => FILE[1],
                    7 => FILE[6],
                    _ => FILE[(f-1) as usize] | FILE[(f+1) as usize],
                };

                if (pawns_bb & adj_mask) == 0 {
                    // every pawn on this file is isolated -> penalize per pawn
                    score -= cnt * PAWN_ISOLATED_PENALTY;
                }
            }
        }
        score
    }

    fn eval_passed_and_protected_passed_pawns(&self, player: Player) -> i32 {
        let (f, e) = self.perspective(player);  // friend, enemy
        let mut fp_mut = f.pawns;
        let mut score = 0;

        while fp_mut != 0 {
            let sq = pop_lsb(&mut fp_mut);
            let (file, rank) = sq_to_coord(sq);
            let ahead = self.pawn_forward_mask(file, rank, player);

            if (e.pawns & ahead) == 0 {  // No enemy pawns ahead => passed
                let advancement = match player {
                    Player::White => rank as i32,
                    Player::Black => (7 - rank) as i32,
                };
                let bonus = PAWN_PASSED_BASE_BONUS + advancement * PAWN_PASSED_RANK_BONUS;
                score += bonus;

                // Check if a friendly pawn exists on adjacent file behind it (one rank behind)
                let protected = match rank {
                    1..=6 => {
                        let behind_rank = match player {
                            Player::White => rank - 1,
                            Player::Black => rank + 1,
                        };
                        let behind_mask = sq_to_bb(&[
                            coord_to_sq(behind_rank, (rank-1).max(0)),
                            coord_to_sq(behind_rank, (rank+1).min(7))
                        ]);
                        if (f.pawns & behind_mask) != 0 {
                            true
                        } else {
                            false
                        }
                    },
                    _ => false,
                };
                if protected {
                    score += PAWN_PROTECTED_PASSED_BONUS;
                }
            }
        }
        score
    }

    fn eval_pawn_structure(&self) -> i32 {
        let mut white_score: i32 = 0;
        white_score += self.eval_doubled_and_isolated_pawns(Player::White);
        white_score += self.eval_passed_and_protected_passed_pawns(Player::White);

        let mut black_score: i32 = 0;
        black_score += self.eval_doubled_and_isolated_pawns(Player::Black);
        black_score += self.eval_passed_and_protected_passed_pawns(Player::Black);

        white_score - black_score
    }

    fn eval_king_safety_for_side(&self, player: Player) -> i32 {
        let king_sq = self.perspective(player).0.king.trailing_zeros() as u8;
        let (file, rank) = sq_to_coord(king_sq);
        let mut score = 0;

        if rank <= 1 && (file <= 2 || file >= 5) {  // Castling bonus
            score += KING_CASTLED_BONUS;
        } else if rank <= 1 && (3 <= file && file <= 4) {  // Still in center (d1/e1)
            score -= KING_CENTER_PENALTY;
        }

        if rank <= 1 {  // Check for pawn shield in front of the king
            let next_rank = rank + 1;
            for df in [-1, 0, 1] {
                let f = file as i8 + df;
                if f < 0 || f > 7 { continue; }
                let sq = coord_to_sq(f as u8, next_rank);
                if (self.w.pawns & bit(sq)) == 0 {
                    score -= KING_NO_PAWN_SHIELD_PENALTY;
                }
                if (self.w.pawns & FILE[f as usize]) == 0 {
                    score -= KING_OPEN_FILE_PENALTY;
                }
            }
        }
        score
    }

    pub fn eval_king_safety(&self) -> i32 {
        let white_score = self.eval_king_safety_for_side(Player::White);
        let black_score = self.eval_king_safety_for_side(Player::Black);
        white_score - black_score
    }

    fn eval_space_control(&self) -> i32 {
        let center_mask: u64 = sq_to_bb(&[D4, E4, D5, E5]);
        let white_half: u64 = FILE[0] | FILE[1] | FILE[2] | FILE[3];
        let black_half: u64 = FILE[4] | FILE[5] | FILE[6] | FILE[7];

        let white_attacks = self.attack_map(Player::White);
        let black_attacks = self.attack_map(Player::Black);

        let white_center = (white_attacks & center_mask).count_ones() as i32;
        let black_center = (black_attacks & center_mask).count_ones() as i32;

        let white_enemy_half = (white_attacks & black_half).count_ones() as i32;
        let black_enemy_half = (black_attacks & white_half).count_ones() as i32;

        SPACE_CONTROL_CENTER_BONUS     * (white_center     - black_center) +
        SPACE_CONTROL_ENEMY_HALF_BONUS * (white_enemy_half - black_enemy_half)
    }
}

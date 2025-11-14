use serde::{Deserialize, Serialize};
use crate::constants::{evaluation::*, board::*};
use crate::utility::*;
use crate::engine::{
    base::{player::Player, piece::Piece},
    board::position::Position,
};

pub type Evaluation = i32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalParams {
    pub pawn_isolated_penalty:       i32,
    pub pawn_doubled_penalty:        i32,
    pub pawn_passed_base_bonus:      i32,
    pub pawn_passed_rank_bonus:      i32,
    pub pawn_protected_passed_bonus: i32,

    pub king_castled_bonus:          i32,
    pub king_center_penalty:         i32,
    pub king_open_file_penalty:      i32,
    pub king_no_pawn_shield_penalty: i32,

    pub space_control_center_bonus:     i32,
    pub space_control_enemy_half_bonus: i32,
}

impl Default for EvalParams {
    fn default() -> Self {
        Self {
            pawn_isolated_penalty:       15,
            pawn_doubled_penalty:        10,
            pawn_passed_base_bonus:      20,
            pawn_passed_rank_bonus:      10,
            pawn_protected_passed_bonus: 10,

            king_castled_bonus:          20,
            king_center_penalty:         30,
            king_open_file_penalty:      15,
            king_no_pawn_shield_penalty: 10,

            space_control_center_bonus:     3,
            space_control_enemy_half_bonus: 1,
        }
    }
}

impl EvalParams {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let params = serde_json::from_str(&json)?;
        Ok(params)
    }
}

impl Position {
    /// Evaluate the given position and return a score from the perspective of the player to move
    pub fn evaluate(&self, params: &EvalParams) -> Evaluation {
        let mut score = 0;

        // Count the score in "white - black" fashion
        score += self.eval_material();
        score += self.eval_piece_square_tables();
        score += self.eval_mobility();
        score += self.eval_pawn_structure(params);
        score += self.eval_king_safety(params);
        score += self.eval_space_control(params);

        // Convert to "player-to-move" perspective
        if self.player_to_move == Player::White {
            score
        } else {
            -score
        }
    }

    fn get_pst_index(&self, sq: u8, player: Player) -> usize {
        if player == Player::White {
            sq as usize
        } else {  // Reversing the PST for black
            (sq ^ 56) as usize
        }
    }

    fn eval_material(&self) -> Evaluation {
        let mut material = 0;
        for piece in Piece::all_variants() {
            material += piece.value() * self.w.count(piece) as i32;
            material -= piece.value() * self.b.count(piece) as i32;
        }
        material
    }

    fn eval_piece_square_tables(&self) -> Evaluation {
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

    fn eval_mobility(&self) -> Evaluation {
        let mut score: i32 = 0;
        for square in 0..64 {
            if let Some((player, piece)) = self.piece_at(square) {
                if piece == Piece::Pawn || piece == Piece::King {
                    continue;
                }
                // TODO: count not pseudomoves, but legal moves here?
                // TODO: scale by a constant here?
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
    fn eval_doubled_and_isolated_pawns(&self, player: Player, p: &EvalParams) -> Evaluation {
        let pawns_bb = self.perspective(player).0.pawns;
        let mut score = 0;
        for f in 0u8..8 {
            let mask = FILE[f as usize];
            let cnt = (pawns_bb & mask).count_ones() as i32;
            if cnt > 1 {
                score -= (cnt-1) * p.pawn_doubled_penalty;
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
                    score -= cnt * p.pawn_isolated_penalty;
                }
            }
        }
        score
    }

    fn eval_passed_and_protected_passed_pawns(&self, player: Player, p: &EvalParams) -> Evaluation {
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
                let bonus = p.pawn_passed_base_bonus + advancement * p.pawn_passed_rank_bonus;
                score += bonus;

                // Check if a friendly pawn exists on adjacent file behind it (one rank behind)
                let protected = match rank {
                    1..=6 => {
                        let behind_rank = match player {
                            Player::White => rank - 1,
                            Player::Black => rank + 1,
                        };
                        let behind_mask = sq_to_bb(&[
                            coord_to_sq((rank-1).max(0), behind_rank),
                            coord_to_sq((rank+1).min(7), behind_rank)
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
                    score += p.pawn_protected_passed_bonus;
                }
            }
        }
        score
    }

    fn eval_pawn_structure(&self, p: &EvalParams) -> Evaluation {
        let mut white_score: i32 = 0;
        white_score += self.eval_doubled_and_isolated_pawns(Player::White, p);
        white_score += self.eval_passed_and_protected_passed_pawns(Player::White, p);

        let mut black_score: i32 = 0;
        black_score += self.eval_doubled_and_isolated_pawns(Player::Black, p);
        black_score += self.eval_passed_and_protected_passed_pawns(Player::Black, p);

        white_score - black_score
    }

    fn eval_king_safety_for_side(&self, player: Player, p: &EvalParams) -> Evaluation {
        let (friend, _enemy) = self.perspective(player);
        let king_sq = friend.king.trailing_zeros() as u8;
        let (file, rank) = sq_to_coord(king_sq);
        let mut score = 0;

        if rank <= 1 && (file <= 2 || file >= 5) {  // Castling bonus
            score += p.king_castled_bonus;
        } else if rank <= 1 && (3 <= file && file <= 4) {  // Still in center (d1/e1)
            score -= p.king_center_penalty;
        }

        if rank <= 1 {  // Check for pawn shield in front of the king
            let next_rank = rank + 1;
            for df in [-1, 0, 1] {
                let f = file as i8 + df;
                if f < 0 || f > 7 { continue; }
                let sq = coord_to_sq(f as u8, next_rank);
                if (friend.pawns & bit(sq)) == 0 {
                    score -= p.king_no_pawn_shield_penalty;
                }
                if (friend.pawns & FILE[f as usize]) == 0 {
                    score -= p.king_open_file_penalty;
                }
            }
        }
        score
    }

    pub fn eval_king_safety(&self, p: &EvalParams) -> Evaluation {
        let white_score = self.eval_king_safety_for_side(Player::White, p);
        let black_score = self.eval_king_safety_for_side(Player::Black, p);
        white_score - black_score
    }

    fn eval_space_control(&self, p: &EvalParams) -> Evaluation {
        let center_mask: u64 = sq_to_bb(&[D4, E4, D5, E5]);
        let white_half: u64 = FILE[0] | FILE[1] | FILE[2] | FILE[3];
        let black_half: u64 = FILE[4] | FILE[5] | FILE[6] | FILE[7];

        let white_attacks = self.attack_map(Player::White);
        let black_attacks = self.attack_map(Player::Black);

        let white_center = (white_attacks & center_mask).count_ones() as i32;
        let black_center = (black_attacks & center_mask).count_ones() as i32;

        let white_enemy_half = (white_attacks & black_half).count_ones() as i32;
        let black_enemy_half = (black_attacks & white_half).count_ones() as i32;

        p.space_control_center_bonus     * (white_center     - black_center) +
        p.space_control_enemy_half_bonus * (white_enemy_half - black_enemy_half)
    }
}

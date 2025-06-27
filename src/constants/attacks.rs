use super::board::*;

const fn generate_knight_attack(square: u8) -> u64 {
    let bb = 1u64 << square;

    let no_ab = bb & !(FILE_A | FILE_B);
    let no_gh = bb & !(FILE_G | FILE_H);
    let no_12 = bb & !(RANK[1] | RANK[2]);
    let no_78 = bb & !(RANK[7] | RANK[8]);

    ((no_ab & !RANK[8]) << 6)  | // left-left-up
    ((no_ab & !RANK[1]) >> 10) | // left-left-down
    ((no_gh & !RANK[8]) << 10) | // right-right-up
    ((no_gh & !RANK[1]) >> 6)  | // right-right-down
    ((no_12 & !FILE_H) >> 15)  | // right-down-down
    ((no_12 & !FILE_A) >> 17)  | // left-down-down
    ((no_78 & !FILE_H) << 17)  | // right-up-up
    ((no_78 & !FILE_A) << 15)    // left-up-up
}

const fn generate_king_attack(square: u8) -> u64 {
    let bb = 1u64 << square;

    let no_a = bb & !FILE_A;
    let no_h = bb & !FILE_H;
    let no_1 = bb & !RANK[1];
    let no_8 = bb & !RANK[8];

    (no_a >> 1)        | // right
    (no_h << 1)        | // left
    (no_1 >> 8)        | // down
    (no_8 << 8)        | // up
    (no_a & no_8) << 7 | // up-left
    (no_h & no_8) << 9 | // up-right
    (no_a & no_1) >> 9 | // down-left
    (no_h & no_1) >> 7   // down-right
}

pub static PAWN_ATTACKS_WHITE: [u64; 64] = {
    let mut table = [0u64; 64];
    let mut sq = 0;
    while sq < 64 {
        let rank = sq / 8;
        let file = sq % 8;
        let mut attacks = 0u64;

        if file > 0 && rank < 7 {
            attacks |= 1u64 << (sq + 7);
        }
        if file < 7 && rank < 7 {
            attacks |= 1u64 << (sq + 9);
        }

        table[sq] = attacks;
        sq += 1;
    }
    table
};

pub static PAWN_ATTACKS_BLACK: [u64; 64] = {
    let mut table = [0u64; 64];
    let mut sq = 0;
    while sq < 64 {
        let rank = sq / 8;
        let file = sq % 8;
        let mut attacks = 0u64;

        if file > 0 && rank > 0 {
            attacks |= 1u64 << (sq - 9);
        }
        if file < 7 && rank > 0 {
            attacks |= 1u64 << (sq - 7);
        }

        table[sq] = attacks;
        sq += 1;
    }
    table
};

pub static KNIGHT_ATTACKS: [u64; 64] = {
    let mut table = [0u64; 64];
    let mut i = 0;
    while i < 64 {
        table[i] = generate_knight_attack(i as u8);
        i += 1;
    }
    table
};

pub static KING_ATTACKS: [u64; 64] = {
    let mut table = [0u64; 64];
    let mut i = 0;
    while i < 64 {
        table[i] = generate_king_attack(i as u8);
        i += 1;
    }
    table
};


#[cfg(test)]
mod tests {
    use super::*;
    use crate::utility::sq_to_bb;

    #[test]
    fn white_pawn_attack_table() {
        assert_eq!(PAWN_ATTACKS_WHITE[10], sq_to_bb(&[17, 19]));
        assert_eq!(PAWN_ATTACKS_WHITE[21], sq_to_bb(&[28, 30]));
        assert_eq!(PAWN_ATTACKS_WHITE[31], sq_to_bb(&[38]));
        assert_eq!(PAWN_ATTACKS_WHITE[35], sq_to_bb(&[42, 44]));
        assert_eq!(PAWN_ATTACKS_WHITE[40], sq_to_bb(&[49]));
    }

    #[test]
    fn black_pawn_attack_table() {
        assert_eq!(PAWN_ATTACKS_BLACK[52], sq_to_bb(&[43, 45]));
        assert_eq!(PAWN_ATTACKS_BLACK[47], sq_to_bb(&[38]));
        assert_eq!(PAWN_ATTACKS_BLACK[29], sq_to_bb(&[20, 22]));
        assert_eq!(PAWN_ATTACKS_BLACK[24], sq_to_bb(&[17]));
        assert_eq!(PAWN_ATTACKS_BLACK[19], sq_to_bb(&[10, 12]));
    }

    #[test]
    fn knight_attack_table() {
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
    fn king_attack_table() {
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
}

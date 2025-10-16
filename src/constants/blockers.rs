use super::board::*;
use crate::utility::bit;

pub static ROOK_MASKS: [u64; 64] = {
    let mut table = [0u64; 64];

    table[0]  = (RANK[1] | FILE_A) & !(RANK[8] | FILE_H | (1 << 0));  // a1
    table[7]  = (RANK[1] | FILE_H) & !(RANK[8] | FILE_A | (1 << 7));  // h1
    table[56] = (RANK[8] | FILE_A) & !(RANK[1] | FILE_H | (1 << 56)); // a8
    table[63] = (RANK[8] | FILE_H) & !(RANK[1] | FILE_A | (1 << 63)); // h8

    let mut i = 0;
    let edges = FILE_A | FILE_H | RANK[1] | RANK[8];
    while i < 64 {
        if i == 0 || i == 7 || i == 56 || i == 63 {
            i += 1;
            continue;
        }

        let rank = i / 8;
        let file = i % 8;

        let bb = 1 << i;
        table[i] = if bb & FILE_A != 0 {  // if on file A
            (RANK[rank + 1] | FILE_A) & !bb & !FILE_H & !RANK[1] & !RANK[8]
        } else if bb & FILE_H != 0 {  // if on file H
            (RANK[rank + 1] | FILE_H) & !bb & !FILE_A & !RANK[1] & !RANK[8]
        } else if bb & RANK[1] != 0 {  // if on rank 1
            (RANK[1] | FILE[file + 1]) & !bb & !RANK[8] & !FILE_A & !FILE_H
        } else if bb & RANK[8] != 0 {  // if on rank 8
            (RANK[8] | FILE[file + 1]) & !bb & !RANK[1] & !FILE_A & !FILE_H
        } else {  // if in the middle
            (RANK[rank + 1] | FILE[file + 1]) & !bb & !edges
        };

        i += 1;
    }

    table
};

pub static BISHOP_MASKS: [u64; 64] = {
    let mut masks = [0u64; 64];
    let mut sq = 0;

    while sq < 64 {
        let rank = sq / 8;
        let file = sq % 8;
        let mut mask = 0u64;

        // top-right
        let mut tr = 1;
        while rank + tr < 7 && file + tr < 7 {
            mask |= bit((rank + tr) * 8 + (file + tr));
            tr += 1;
        }

        // top-left
        let mut tl = 1;
        while rank + tl < 7 && file > tl {
            mask |= bit((rank + tl) * 8 + (file - tl));
            tl += 1;
        }

        // bottom-right
        let mut br = 1;
        while rank > br && file + br < 7 {
            mask |= bit((rank - br) * 8 + (file + br));
            br += 1;
        }

        // bottom-left
        let mut bl = 1;
        while rank > bl && file >= bl {
            mask |= bit((rank - bl) * 8 + (file - bl));
            bl += 1;
        }

        masks[sq as usize] = mask;
        sq += 1;
    }
    masks
};


#[cfg(test)]
mod tests {
    use super::*;
    use crate::utility::sq_to_bb;

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
}

pub const fn bit(sq: usize) -> u64 {
    1u64 << sq
}

pub fn pop_lsb(bitboard: &mut u64) -> u8 {
    let result = bitboard.trailing_zeros() as u8;
    *bitboard &= *bitboard - 1;
    result
}

pub fn square_idx_to_string(sq: u8) -> String {
    let file = (sq % 8) as u8;
    let rank = (sq / 8) as u8;
    format!("{}{}", (file + b'a') as char, rank + 1)
}

pub fn square_string_to_idx(sq: &str) -> Option<u8> {
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

pub fn signed_shift(bb: u64, offset: i8) -> u64 {
    if offset >= 0 {
        bb << offset
    } else {
        bb >> (-offset)
    }
}

pub fn print_bitboard(bb: u64) {
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

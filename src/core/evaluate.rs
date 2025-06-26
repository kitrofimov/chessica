use crate::core::{
    position::Position,
    piece::Piece,
};

pub fn evaluate(pos: &Position) -> i32 {
    let mut score = 0;
    for piece in Piece::all_variants() {
        score += piece.value() * pos.w.count(*piece) as i32;
        score -= piece.value() * pos.b.count(*piece) as i32;
    }
    score
}

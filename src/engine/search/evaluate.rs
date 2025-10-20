use crate::engine::{
    base::piece::Piece,
    board::position::Position,
};

/// Evaluate the given position and return a score from white's perspective
// A dumb evaluation function only counting material; to be improved
pub fn evaluate(pos: &Position) -> i32 {
    let mut score = 0;
    for piece in Piece::all_variants() {
        score += piece.value() * pos.w.count(piece) as i32;
        score -= piece.value() * pos.b.count(piece) as i32;
    }
    score
}

use crate::utility::{is_square_color_white, lsb};
use crate::core::{
    position::*,
    piece::Piece,
};

pub fn is_insufficient_material(pos: &Position) -> bool {
    let total_pieces = pos.w.count_all() + pos.b.count_all();

    let w_bishops = pos.w.count(Piece::Bishop);
    let w_knights = pos.w.count(Piece::Knight);
    let b_bishops = pos.b.count(Piece::Bishop);
    let b_knights = pos.b.count(Piece::Knight);

    // King vs King
    if total_pieces == 2 {
        return true;
    }

    // King and single minor piece vs King
    let white_minors = w_bishops + w_knights;
    let black_minors = b_bishops + b_knights;

    if total_pieces == 3 {
        return (white_minors == 1 && black_minors == 0) ||
               (white_minors == 0 && black_minors == 1);
    }

    // King + Bishop vs King + Bishop (same color bishops)
    if total_pieces == 4
        && w_bishops == 1
        && b_bishops == 1
        && w_knights == 0
        && b_knights == 0
    {
        let wb_sq = lsb(pos.w.bishops);
        let bb_sq = lsb(pos.b.bishops);
        let same_color = is_square_color_white(wb_sq) == is_square_color_white(bb_sq);
        return same_color;
    }

    false
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insufficient_material_king_vs_king() {
        let (pos, _) = Position::from_fen("8/8/8/8/8/8/8/K2k4 w - - 0 1").unwrap();
        assert_eq!(is_insufficient_material(&pos), true);
    }

    #[test]
    fn test_insufficient_material_king_and_bishop_vs_king() {
        let (pos, _) = Position::from_fen("8/8/8/1K2k3/8/8/5B2/8 w - - 0 1").unwrap();
        assert_eq!(is_insufficient_material(&pos), true);
    }

    #[test]
    fn test_insufficient_material_king_and_knight_vs_king() {
        let (pos, _) = Position::from_fen("8/8/5N2/2K5/8/6k1/8/8 w - - 0 1").unwrap();
        assert_eq!(is_insufficient_material(&pos), true);
    }

    #[test]
    fn test_insufficient_material_king_bishop_vs_king_bishop_same_color() {
        let (pos, _) = Position::from_fen("8/8/3k2b1/8/8/1K3B2/8/8 w - - 0 1").unwrap();
        assert_eq!(is_insufficient_material(&pos), true);
    }

    #[test]
    fn test_sufficient_material_king_bishop_vs_king_bishop_opposite_color() {
        let (pos, _) = Position::from_fen("8/6b1/3k4/8/8/1K3B2/8/8 w - - 0 1").unwrap();
        assert_eq!(is_insufficient_material(&pos), false);
    }

    #[test]
    fn test_sufficient_material_pawn() {
        let (pos, _) = Position::from_fen("8/8/3k4/8/8/4P3/1K6/8 w - - 0 1").unwrap();
        assert_eq!(is_insufficient_material(&pos), false);
    }
}

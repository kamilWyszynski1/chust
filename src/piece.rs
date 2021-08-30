use std::cmp::min;

#[derive(Clone, Copy, PartialEq, Hash, Eq)]
pub enum Color {
    NONE,
    BLACK,
    WHITE,
}

impl Color {
    pub fn opposite(&self) -> Self {
        match self {
            Color::NONE => Color::NONE,
            Color::BLACK => Color::WHITE,
            Color::WHITE => Color::BLACK,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum PieceType {
    NONE,
    KING,
    PAWN,
    KNIGHT,
    BISHOP,
    ROOK,
    QUEEN,
}

#[derive(Clone, Copy)]
pub struct Piece {
    pub p_type: PieceType,
    pub color: Color,
    pub has_moved: bool,
    position: usize,
}

impl Piece {
    pub(crate) fn default() -> Self {
        Piece {
            p_type: PieceType::NONE,
            color: Color::NONE,
            has_moved: false,
            position: 0,
        }
    }

    pub fn new(p_type: PieceType, color: Color, position: usize) -> Self {
        Piece {
            p_type,
            color,
            position,
            has_moved: false,
        }
    }

    pub fn visualize(&self) -> String {
        let ch = match self.p_type {
            PieceType::NONE => "",
            PieceType::KING => "k",
            PieceType::PAWN => "p",
            PieceType::KNIGHT => "n",
            PieceType::BISHOP => "b",
            PieceType::ROOK => "r",
            PieceType::QUEEN => "q",
        };

        return match self.color {
            Color::NONE => "x".to_string(),
            Color::BLACK => ch.to_string(),
            Color::WHITE => ch.to_uppercase(),
        };
    }

    pub fn is_none(&self) -> bool {
        self.p_type == PieceType::NONE
    }

    pub fn is_sliding(&self) -> bool {
        return match self.p_type {
            PieceType::BISHOP | PieceType::ROOK | PieceType::QUEEN => true,
            _ => false,
        };
    }

    fn get_moves_for_rook(&self) -> Vec<i32> {
        let mut rook_moves = Vec::<i32>::new();
        let ptcr = position_to_row_col(self.position);
        if ptcr.is_none() {
            return Vec::new();
        }
        let (row, col) = ptcr.unwrap();
        for i in 1..row {
            rook_moves.push(-8 * i as i32); // to left
        }
        for i in 1..(9 - row) {
            rook_moves.push(i as i32 * 8);
        }
        for i in 1..col {
            rook_moves.push(-1 * i as i32); // to left
        }
        for i in 1..(9 - col) {
            rook_moves.push(i as i32);
        }
        return rook_moves;
    }

    fn get_moves_for_bishop(&self) -> Vec<i32> {
        let mut bishop_moves = Vec::<i32>::new();
        let ptcr = position_to_row_col(self.position);
        if ptcr.is_none() {
            return Vec::new();
        }
        let (row, col) = ptcr.unwrap();

        // up left
        for i in 1..min(9 - row, col) {
            bishop_moves.push(7 * i as i32);
        }
        // up right
        for i in 1..min(9 - row, 9 - col) {
            bishop_moves.push(9 * i as i32);
        }
        // down left
        for i in 1..min(row, col) {
            bishop_moves.push(-9 * i as i32);
        }
        // up right
        for i in 1..min(row, 9 - col) {
            bishop_moves.push(-7 * i as i32);
        }

        return bishop_moves;
    }

    pub fn get_moves(&self) -> Vec<i32> {
        let mut modifier = 1;
        if self.color == Color::BLACK {
            modifier = -1;
        }

        // TODO: handle taking pieces
        let mut pawn_moves = vec![8 * modifier, 7 * modifier, 9 * modifier];
        if !self.has_moved {
            pawn_moves.push(16 * modifier);
        }

        return match self.p_type {
            PieceType::NONE => Vec::new(),
            PieceType::KING => vec![-1, 7, 8, 9, 1, -7, -8, -9],
            PieceType::PAWN => pawn_moves,
            PieceType::KNIGHT => vec![6, 15, 17, 10, -6, -15, -17, -10],
            PieceType::BISHOP => self.get_moves_for_bishop(),
            PieceType::ROOK => self.get_moves_for_rook(),
            PieceType::QUEEN => {
                let r = self.get_moves_for_rook();
                let b = self.get_moves_for_bishop();
                let mut q = Vec::new();
                q.extend_from_slice(&r);
                q.extend_from_slice(&b);
                q
            }
        };
    }

    pub fn get_sliding_moves(&self) -> Vec<i32> {
        return match self.p_type {
            PieceType::BISHOP => vec![9, 7, -9, -7],
            PieceType::ROOK => vec![8, 1, -8, -1],
            PieceType::QUEEN => vec![9, 7, -9, -7, 8, 1, -8, -1],
            _ => Vec::new(),
        };
    }
}

fn position_to_row_col(position: usize) -> Option<(usize, usize)> {
    for i in 0..8 {
        if position >= 8 * i && position < 8 * (i + 1) {
            if (position + 1) % 8 == 0 {
                return Some((i + 1, 8));
            } else {
                return Some((i + 1, (position + 1) % 8 as usize));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use crate::piece::{position_to_row_col, Color, Piece, PieceType};

    #[test]
    fn test_position_to_row_col_test() {
        assert_eq!(position_to_row_col(2).unwrap(), (1, 3));
        assert_eq!(position_to_row_col(25).unwrap(), (4, 2));
        assert_eq!(position_to_row_col(61).unwrap(), (8, 6));
        assert_eq!(position_to_row_col(12).unwrap(), (2, 5));
        assert_eq!(position_to_row_col(52).unwrap(), (7, 5));
        assert_eq!(position_to_row_col(33).unwrap(), (5, 2));
    }

    #[test]
    fn test_get_moves() {
        let p = Piece::new(PieceType::ROOK, Color::WHITE, 25);
        let mut moves = p.get_moves();
        moves.sort();
        assert_eq!(
            moves,
            vec![-24, -16, -8, -1, 1, 2, 3, 4, 5, 6, 8, 16, 24, 32]
        );

        let p = Piece::new(PieceType::ROOK, Color::WHITE, 37);
        let mut moves = p.get_moves();
        moves.sort();
        assert_eq!(
            moves,
            vec![-32, -24, -16, -8, -5, -4, -3, -2, -1, 1, 2, 8, 16, 24]
        );

        let p = Piece::new(PieceType::ROOK, Color::WHITE, 60);
        let mut moves = p.get_moves();
        moves.sort();
        assert_eq!(
            moves,
            vec![-56, -48, -40, -32, -24, -16, -8, -4, -3, -2, -1, 1, 2, 3]
        );

        let p = Piece::new(PieceType::ROOK, Color::WHITE, 0);
        let mut moves = p.get_moves();
        moves.sort();
        assert_eq!(moves, vec![1, 2, 3, 4, 5, 6, 7, 8, 16, 24, 32, 40, 48, 56]);
    }
    #[test]
    fn test_get_moves_for_bishop() {
        let p = Piece::new(PieceType::BISHOP, Color::WHITE, 53);
        let mut moves = p.get_moves_for_bishop();
        moves.sort();
        assert_eq!(moves, vec![-45, -36, -27, -18, -14, -9, -7, 7, 9]);

        let p = Piece::new(PieceType::BISHOP, Color::WHITE, 33);
        let mut moves = p.get_moves_for_bishop();
        moves.sort();
        let mut wanted_moves = vec![-9, -7, -14, -21, -28, 7, 9, 18, 27];
        wanted_moves.sort();
        assert_eq!(moves, wanted_moves);

        let p = Piece::new(PieceType::BISHOP, Color::WHITE, 9);
        let mut moves = p.get_moves_for_bishop();
        moves.sort();
        let mut wanted_moves = vec![-9, -7, 7, 9, 18, 27, 36, 45, 54];
        wanted_moves.sort();
        assert_eq!(moves, wanted_moves);

        let p = Piece::new(PieceType::BISHOP, Color::WHITE, 30);
        let mut moves = p.get_moves_for_bishop();
        moves.sort();
        let mut wanted_moves = vec![-27, -18, -9, -7, 9, 7, 14, 21, 28];
        wanted_moves.sort();
        assert_eq!(moves, wanted_moves);
    }
}
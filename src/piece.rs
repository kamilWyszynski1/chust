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

    pub fn get_moves(&self) -> Vec<i32> {
        if self.p_type == PieceType::NONE {
            return Vec::new();
        }
        // TODO store as static
        let mut bishop_moves = Vec::new();
        let mut rook_moves = Vec::new();

        for i in 1..8 {
            if self.color == Color::BLACK {
                bishop_moves.push(9 * i); // right diagonal
                bishop_moves.push(-9 * i);
            } else {
                bishop_moves.push(7 * i); // left diagonal
                bishop_moves.push(-7 * i);
            }

            rook_moves.push(8 * i);
            rook_moves.push(-8 * i);
            rook_moves.push(i);
            rook_moves.push(-1 * i);
        }

        let mut queen_moves = Vec::new();
        queen_moves.extend_from_slice(&rook_moves);
        queen_moves.extend_from_slice(&bishop_moves);

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
            PieceType::BISHOP => bishop_moves,
            PieceType::ROOK => rook_moves,
            PieceType::QUEEN => queen_moves,
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

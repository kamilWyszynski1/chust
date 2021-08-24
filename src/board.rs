use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq)]
enum Color {
    NONE,
    BLACK,
    WHITE,
}

#[derive(Clone, Copy, PartialEq)]
enum PawnType {
    NONE,
    KING,
    PAWN,
    KNIGHT,
    BISHOP,
    ROOK,
    QUEEN,
}

#[derive(Clone, Copy)]
struct Pawn {
    p_type: PawnType,
    color: Color,
    has_moved: bool,
}

impl Pawn {
    pub fn is_none(&self) -> bool {
        self.p_type == PawnType::NONE
    }

    pub fn is_sliding(&self) -> bool {
        return match self.p_type {
            PawnType::BISHOP | PawnType::ROOK | PawnType::QUEEN => true,
            _ => false,
        };
    }

    pub fn get_moves(&self) -> Vec<i32> {
        if self.p_type == PawnType::NONE {
            return Vec::new();
        }
        // TODO store as static
        let mut bishop_moves = Vec::new();
        let mut rook_moves = Vec::new();

        for i in 1..8 {
            bishop_moves.push(9 * i); // right diagonal
            bishop_moves.push(7 * i); // left diagonal
            bishop_moves.push(-7 * i);
            bishop_moves.push(-9 * i);

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
        let mut pawn_moves = vec![8 * modifier];
        if !self.has_moved {
            pawn_moves.push(16 * modifier);
        }

        return match self.p_type {
            PawnType::NONE => Vec::new(),
            PawnType::KING => vec![-1, 7, 8, 9, 1, -7, -8, -9],
            PawnType::PAWN => pawn_moves,
            PawnType::KNIGHT => vec![6, 15, 17, 10, -6, -15, -17, -10],
            PawnType::BISHOP => bishop_moves,
            PawnType::ROOK => rook_moves,
            PawnType::QUEEN => queen_moves,
        };
    }

    pub fn get_sliding_moves(&self) -> Vec<i32> {
        return match self.p_type {
            PawnType::BISHOP => vec![9, 7, -9, -7],
            PawnType::ROOK => vec![8, 1, -8, -1],
            PawnType::QUEEN => vec![9, 7, -9, -7, 8, 1, -8, -1],
            _ => Vec::new(),
        };
    }
}

impl Pawn {
    fn default() -> Self {
        Pawn {
            p_type: PawnType::NONE,
            color: Color::NONE,
            has_moved: false,
        }
    }

    fn visualize(&self) -> String {
        let ch = match self.p_type {
            PawnType::NONE => "",
            PawnType::KING => "k",
            PawnType::PAWN => "p",
            PawnType::KNIGHT => "n",
            PawnType::BISHOP => "b",
            PawnType::ROOK => "r",
            PawnType::QUEEN => "q",
        };

        return match self.color {
            Color::NONE => "x".to_string(),
            Color::BLACK => ch.to_string(),
            Color::WHITE => ch.to_uppercase(),
        };
    }
}

pub struct Board {
    squares: [Pawn; 64], // 0 is left lower corner
    color_to_move: Color,
}

impl Board {
    pub fn default() -> Board {
        Board {
            squares: [Pawn::default(); 64],
            color_to_move: Color::WHITE,
        }
    }

    pub fn read_fen(&mut self, fen: &str) {
        self.squares = [Pawn::default(); 64]; // reset board
        let piece_from_char: HashMap<char, PawnType> = [
            ('r', PawnType::ROOK),
            ('k', PawnType::KING),
            ('p', PawnType::PAWN),
            ('q', PawnType::QUEEN),
            ('b', PawnType::BISHOP),
            ('n', PawnType::KNIGHT),
        ]
        .iter()
        .cloned()
        .collect();

        let mut rank: i32 = 7;
        let mut file: i32 = 0;

        for (i, c) in fen.chars().enumerate() {
            match c {
                '/' => {
                    file = 0;
                    rank -= 1;
                }
                _ => {
                    if c.is_digit(10) {
                        file += c.to_digit(10).unwrap() as i32;
                    } else {
                        self.squares[(rank * 8 + file) as usize] = Pawn {
                            p_type: piece_from_char
                                .get(&char::to_ascii_lowercase(&c))
                                .unwrap()
                                .clone(),
                            color: match c.is_lowercase() {
                                true => Color::BLACK,
                                false => Color::WHITE,
                            },
                            has_moved: false,
                        };
                        file += 1;
                    }
                }
            }
        }
    }

    pub fn visualize(&self) -> String {
        let mut rank = 7;
        let mut file = 0;
        let mut board = String::new();

        for i in 0..8 {
            board.push_str(format!("{}|", 8 - i).as_str());
            for _ in 0..8 {
                board.push_str(self.squares[8 * rank + file].visualize().as_str());
                file += 1;
            }
            if rank == 0 {
                board.push_str("\n");
                board.push_str("  --------");
                board.push_str("\n");
                board.push_str("  abcdefgh");
                break;
            }
            board.push_str("\n");
            rank -= 1;
            file = 0;
        }
        board
    }

    // make_move validates move and make it
    // m will be always like this: a2a4 meaning that piece from a2 moves to a4
    pub fn make_move(&self, m: &str) -> Result<(), &'static str> {
        let (first, second) = m.split_at(2);
        let first_pos = self.translate_position(first);
        let second_pos = self.translate_position(second);

        self.validate_move(first_pos, second_pos)
    }

    fn validate_move(&self, from: i32, to: i32) -> Result<(), &'static str> {
        let piece = self.squares[from as usize];
        let position_to = self.squares[to as usize];

        println!("from: {}, to: {}", from, to);

        // TODO: check if there won't be check on us
        if piece.is_none()
            || (!position_to.is_none() && piece.color == position_to.color)
            || self.color_to_move != piece.color
        {
            return Err("piece is none, position_to is occupied by the same color piece or it is not your move");
        }

        let available_moves = piece.get_moves();
        if !available_moves.contains(&(to - from)) {
            return Err("that piece cannot make moves like that!");
        }

        // check if there's no other piece on your way
        if piece.is_sliding() {
            let sliding_moves = piece.get_sliding_moves();
            let mut blocked = false;
            let mut from_temp = from.clone();
            let mut is_valid = false;
            for m in &sliding_moves {
                loop {
                    from_temp += m;
                    if from_temp > 63 || from_temp < 0 {
                        break
                    }
                    if from_temp == to {
                        if blocked {
                            return Err("your move is blocked");
                        }
                        is_valid = true;
                        break;
                    }
                    if !self.squares[from_temp as usize].is_none() {
                        blocked = true;
                    }
                }
                if is_valid {
                    break;
                }
                blocked = false;
            }
        }

        println!("{:?}", piece.get_moves());

        Ok(())
    }

    fn translate_position(&self, pos: &str) -> i32 {
        let mut inx: i32 = 0;
        let (col, row) = pos.split_at(1);
        col.chars().for_each(|c| inx += (c as i32 - 'a' as i32));
        row.chars()
            .for_each(|c| inx += (c.to_digit(10).unwrap() as i32 - 1) * 8);
        inx
    }
}

#[cfg(test)]
mod tests {
    use crate::board;
    use crate::board::Color;

    #[test]
    fn block_detection() {
        let mut b = board::Board::default();
        b.read_fen("rnbqkbnr/pppppppp/8/8/8/8/PPP1PPPP/RNBQKBNR");
        assert_eq!(b.make_move("c1g5").unwrap(), ());

        b.read_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");
        assert_eq!(b.make_move("c1g5").err().unwrap(), "your move is blocked");

        b.read_fen("q7/pppppppp/8/8/8/8/8/8");
        b.color_to_move = Color::BLACK;
        assert_eq!(b.make_move("a8a1").err().unwrap(), "your move is blocked");
    }
}
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Hash, Eq)]
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
    kings_positions: HashMap<Color, i32>,
}

const FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPP1PPPP/RNBQKBNR";

impl Board {
    pub fn default() -> Board {
        let mut b = Board {
            squares: [Pawn::default(); 64],
            color_to_move: Color::WHITE,
            kings_positions: HashMap::new(),
        };
        b.read_fen(FEN);
        b
    }

    pub fn read_fen(&mut self, fen: &str) {
        self.squares = [Pawn::default(); 64]; // reset board
        self.kings_positions = HashMap::new();
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

        for (_i, c) in fen.chars().enumerate() {
            match c {
                '/' => {
                    file = 0;
                    rank -= 1;
                }
                _ => {
                    if c.is_digit(10) {
                        file += c.to_digit(10).unwrap() as i32;
                    } else {
                        let color = match c.is_lowercase() {
                            true => Color::BLACK,
                            false => Color::WHITE,
                        };
                        let p = Pawn {
                            p_type: piece_from_char
                                .get(&char::to_ascii_lowercase(&c))
                                .unwrap()
                                .clone(),
                            color,
                            has_moved: false,
                        };
                        let inx = rank * 8 + file;
                        self.squares[inx as usize] = p;
                        if p.p_type == PawnType::KING {
                            self.kings_positions.insert(color, inx);
                        }

                        file += 1;
                    }
                }
            }
        }
    }

    // 1.e4 e5 2.Nf3 f6 3.Nxe5 fxe5 4.Qh5+ Ke7 5.Qxe5+ Kf7 6.Bc4+ d5 7.Bxd5+
    // Kg6 8.h4 h5 9.Bxb7 Bxb7 10.Qf5+ Kh6 11.d4+ g5 12.Qf7 Qe7 13.hxg5+ Qxg5
    // 14.Rxh5#"
    pub fn read_pgn(&mut self, pgn: &str, vis_flag: bool) {
        let mut counter = 1;
        loop {
            let mut pgn = pgn.replace(format!("{}.", counter).as_str(), "");
            let (m, pgn) = pgn.split_once(" ").unwrap();
        }
    }

    #[warn(dead_code)]
    pub fn visualize(&self) {
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
        println!("{}", board)
    }

    // translate_move gets algebraic notation and translates it to move
    // e.g. Nxe5, Qh5+, g5, hxg5+
    fn translate_move(&self, m: &str) -> Result<(i32, i32), &'static str> {
        let (first, second) = m.split_at(2);
        match first {
            "h" => return Err("not implemented"),
        }
    }

    // make_move validates move and make it
    // m will be always like this: a2a4 meaning that piece from a2 moves to a4
    pub fn make_move(&mut self, m: &str) -> Result<(), &'static str> {
        let (first, second) = m.split_at(2);
        let first_pos = self.translate_position(first);
        let second_pos = self.translate_position(second);

        self.validate_move(first_pos, second_pos)
    }

    fn validate_move(&mut self, from: i32, to: i32) -> Result<(), &'static str> {
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

        match self.is_move_possible(&piece, from, to) {
            Ok(_) => {}
            Err(e) => return Err(e),
        };

        self.squares[from as usize] = Pawn::default();
        self.squares[to as usize] = piece;

        // check for check
        let king_pos = self.kings_positions.get(&self.color_to_move).unwrap();
        for (inx, p) in self.squares.iter().enumerate() {
            if piece.color != p.color && !p.is_none() {
                if self.is_move_possible(p, inx as i32, *king_pos).is_ok() {
                    // rollback changes
                    self.squares[from as usize] = piece;
                    self.squares[to as usize] = Pawn::default();

                    return Err("there will be check after a move");
                }
            }
        }
        Ok(())
    }

    fn is_move_possible(&self, piece: &Pawn, from: i32, to: i32) -> Result<(), &'static str> {
        let available_moves = piece.get_moves();
        if !available_moves.contains(&(to - from)) {
            return Err("that piece cannot make moves like that!");
        }

        // check if there's no other piece on your way
        if piece.is_sliding() {
            let sliding_moves = piece.get_sliding_moves();
            let mut blocked = false;
            let mut is_valid = false;
            for m in &sliding_moves {
                let mut from_temp = from.clone();
                loop {
                    from_temp += m;
                    if from_temp > 63 || from_temp < 0 {
                        break;
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
        Ok(())
    }

    fn translate_position(&self, pos: &str) -> i32 {
        let mut inx: i32 = 0;
        let (col, row) = pos.split_at(1);
        col.chars().for_each(|c| inx += c as i32 - 'a' as i32);
        row.chars()
            .for_each(|c| inx += (c.to_digit(10).unwrap() as i32 - 1) * 8);
        inx
    }
}

#[cfg(test)]
mod tests {
    use crate::board;
    use crate::board::{Board, Color};

    #[test]
    fn block_detection() {
        let mut b = board::Board::default();
        assert_eq!(b.make_move("c1g5").unwrap(), ());

        b.read_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");
        assert_eq!(b.make_move("c1g5").err().unwrap(), "your move is blocked");

        b.read_fen("q7/pppppppp/8/8/8/8/8/8");
        b.color_to_move = Color::BLACK;
        assert_eq!(b.make_move("a8a1").err().unwrap(), "your move is blocked");
    }

    #[test]
    fn invalid_move() {
        let mut b = board::Board::default();
        b.read_fen("r7/8/8/8/8/8/8/8");
        b.color_to_move = Color::BLACK;
        assert_eq!(
            b.make_move("a8b1").err().unwrap(),
            "that piece cannot make moves like that!"
        );
    }

    #[test]
    fn king_position() {
        let b = board::Board::default();
        assert_eq!(*b.kings_positions.get(&Color::BLACK).unwrap(), 60);
        assert_eq!(*b.kings_positions.get(&Color::WHITE).unwrap(), 4);
    }

    #[test]
    fn blocked_move() {
        let mut b = board::Board::default();
        b.color_to_move = Color::BLACK;

        b.read_fen("r7/p7/8/8/8/8/8/8");
        assert_eq!(b.make_move("a8a1").err().unwrap(), "your move is blocked");
    }

    #[test]
    fn check_after_move() {
        let mut b = board::Board::default();
        b.color_to_move = Color::BLACK;
        b.read_fen("k7/q7/8/8/8/8/R7/K7");
        assert_eq!(
            b.make_move("a7b7").err().unwrap(),
            "there will be check after a move"
        );

        b.read_fen("k7/q7/p7/8/8/8/R7/K7");
        assert_eq!(b.make_move("a7b7").is_ok(), true);
    }

    #[test]
    fn read_pgn() {
        let pgn = "1.e4 e5 2.Nf3 f6 3.Nxe5 fxe5 4.Qh5+ Ke7 5.Qxe5+ Kf7 6.Bc4+ d5 7.Bxd5+
Kg6 8.h4 h5 9.Bxb7 Bxb7 10.Qf5+ Kh6 11.d4+ g5 12.Qf7 Qe7 13.hxg5+ Qxg5
14.Rxh5#";
        let mut b = Board::default();
        b.read_pgn(pgn, true);
    }
}

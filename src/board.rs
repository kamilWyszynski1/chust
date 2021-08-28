#![allow(warnings, unused)]

use std::borrow::Borrow;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Hash, Eq)]
enum Color {
    NONE,
    BLACK,
    WHITE,
}

impl Color {
    fn opposite(&self) -> Self {
        match self {
            Color::NONE => Color::NONE,
            Color::BLACK => Color::WHITE,
            Color::WHITE => Color::BLACK,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum PieceType {
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
    p_type: PieceType,
    color: Color,
    has_moved: bool,
}

impl Pawn {
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

impl Pawn {
    fn default() -> Self {
        Pawn {
            p_type: PieceType::NONE,
            color: Color::NONE,
            has_moved: false,
        }
    }

    fn visualize(&self) -> String {
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
}

#[derive(Clone)]
pub struct Board {
    squares: [Pawn; 64], // 0 is left lower corner
    color_to_move: Color,
    kings_positions: HashMap<Color, i32>,
    debug: bool,
}

const FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";

impl Board {
    pub fn default() -> Board {
        let mut b = Board {
            squares: [Pawn::default(); 64],
            color_to_move: Color::WHITE,
            kings_positions: HashMap::new(),
            debug: false,
        };
        b.read_fen(FEN);
        b
    }

    pub fn allow_debug(&mut self) {
        self.debug = true
    }

    pub fn read_fen(&mut self, fen: &str) {
        self.squares = [Pawn::default(); 64]; // reset board
        self.kings_positions = HashMap::new();
        let piece_from_char: HashMap<char, PieceType> = [
            ('r', PieceType::ROOK),
            ('k', PieceType::KING),
            ('p', PieceType::PAWN),
            ('q', PieceType::QUEEN),
            ('b', PieceType::BISHOP),
            ('n', PieceType::KNIGHT),
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
                        if p.p_type == PieceType::KING {
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
    pub fn read_pgn(&mut self, pgn: &str, vis_flag: bool) -> Result<(), &'static str> {
        let mut game = String::from(pgn.replace("\n", "").replace("  ", " "));
        let mut general_counter = 1;
        let mut color_counter = 0;
        loop {
            if game.len() == 0 {
                break;
            }
            if color_counter == 0 {
                game = game.replacen(format!("{}.", general_counter).as_str(), "", 1);
            }
            let mut temp_game = game.to_owned();
            while temp_game.starts_with(" ") {
                temp_game = temp_game.replacen(" ", "", 1)
            }

            let (chess_move, trimmed) = match temp_game.split_once(" ") {
                Some((chess_move, trimmed)) => (chess_move, trimmed),
                None => (temp_game.as_str(), ""), // last move
            };
            if trimmed != "" {
                game = String::from(trimmed);
            } else {
                game = String::new();
            }

            if self.debug {
                println!("making {} move", chess_move);
            }

            match self.make_pgn_move(chess_move) {
                Err(e) => return Err(e),
                _ => {}
            }
            if color_counter == 1 {
                color_counter = 0;
                general_counter += 1;
            } else {
                color_counter += 1;
            }
        }
        Ok(())
    }

    fn make_pgn_move(&mut self, m: &str) -> Result<(), &'static str> {
        let (from, to) = match self.translate_pgn_move(m) {
            Ok((from, to)) => (from, to),
            Err(err) => return Err(err),
        };
        let from = from as usize;
        let to = to as usize;
        self.make_move(from, to);
        Ok(())
    }

    fn make_move(&mut self, from: usize, to: usize) {
        self.squares[to] = self.squares[from];
        self.squares[to].has_moved = true;
        self.squares[from] = Pawn::default();
        self.color_to_move = self.color_to_move.opposite();
        if self.squares[to].p_type == PieceType::KING {
            self.kings_positions
                .insert(self.squares[to].color, to as i32);
        }
    }

    // translate_move gets algebraic notation and translates it to move
    // e.g. Nxe5, Qh5+, g5, hxg5+
    fn translate_pgn_move(&mut self, m: &str) -> Result<(i32, i32), &'static str> {
        if m == "O-O" {
            // short castle
            unimplemented!("short castle")
        } else if m == "O-O-O" {
            // long castle
            unimplemented!("long castle")
        }

        let mut pawn_move = false;
        let pawn_letters = vec!["a", "b", "c", "d", "e", "f", "g", "h"];
        let m = m.replace("x", "").replace("+", "").replace("#", "");

        for l in &pawn_letters {
            if m.starts_with(l) {
                pawn_move = true;
                break;
            }
        }

        let piece_to_find;
        let places;
        let direction;
        if pawn_move {
            piece_to_find = PieceType::PAWN;
            if m.len() == 3 {
                // pawn takes
                let (first, second) = m.split_at(1);
                places = self.find_pawn_places(first);
                direction = self.translate_position(second);
            } else {
                // basic move
                direction = self.translate_position(m.as_str());
                let (first, _) = m.split_at(1);
                places = self.find_pawn_places(first);
            }
        } else {
            let (first, mut second) = m.split_at(1);
            let mut additional_info = String::new();
            let piece_to_find = match first {
                "N" => {
                    // both knights can jump into the same square
                    // we need to check if that is happening
                    //
                    // basically check len of move and check for given row/column of a knight
                    if second.len() != 2 {
                        let mut chars = second.chars();
                        additional_info = chars.next().unwrap().to_string();
                        second = chars.as_str();
                    }
                    PieceType::KNIGHT
                }
                "Q" => PieceType::QUEEN,
                "B" => PieceType::BISHOP,
                "R" => PieceType::ROOK,
                "K" => PieceType::KING,
                _ => return Err("invalid piece"),
            };
            places = self.find_piece_places(piece_to_find, self.color_to_move, additional_info);
            direction = self.translate_position(second);
        }
        for place in &places {
            if self.validate_move(*place, direction).is_ok() {
                return Ok((*place, direction));
            }
        }
        return Err("invalid move");
    }

    fn find_piece_places(
        &self,
        piece_type: PieceType,
        color: Color,
        additional_info: String,
    ) -> Vec<i32> {
        let mut places = Vec::new();

        self.squares.iter().enumerate().for_each(|(i, p)| {
            if p.p_type == piece_type && p.color == color {
                if additional_info.len() == 1 {
                    let i = i as i32;
                    // there's additional info
                    let info = additional_info.chars().next().unwrap();
                    if info.is_digit(10) {
                        // check for row
                        let row = info.to_digit(10).unwrap() as i32;
                        if (row - 1) * 8 >= i && row * 8 < i {
                            places.push(i);
                        }
                    } else {
                        // check for column
                        let column = letter_to_i32(&info);
                        let possible_indexes: Vec<i32> =
                            (1..9).map(|x| column + 8 * (x - 1)).collect();
                        if possible_indexes.contains(&i) {
                            places.push(i as i32)
                        }
                    }
                } else {
                    places.push(i as i32)
                }
            }
        });
        places
    }

    // find_pawn_places takes e.g. 'e' and returns all pawn position that is on 'e' line
    fn find_pawn_places(&self, line: &str) -> Vec<i32> {
        let mut places = Vec::new();
        if line.len() != 1 {
            panic!("line len must be 1")
        }
        let mut inx = 0;
        line.chars().for_each(|c| inx = c as i32 - 'a' as i32); // only 1 iteration

        for i in 0..7 {
            let index = inx + 8 * i;
            let p = self.squares[index as usize];
            if p.p_type == PieceType::PAWN && p.color == self.color_to_move {
                places.push(index);
            }
        }

        places
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

    // make_move validates move and make it
    // m will be always like this: a2a4 meaning that piece from a2 moves to a4
    pub fn make_move_internal_notation(&mut self, m: &str) -> Result<(), &'static str> {
        let (first, second) = m.split_at(2);
        let first_pos = self.translate_position(first);
        let second_pos = self.translate_position(second);

        self.validate_move(first_pos, second_pos)
    }

    // validate_move validates if move is legit. It checks every aspect of a game.
    fn validate_move(&mut self, from: i32, to: i32) -> Result<(), &'static str> {
        let piece = self.squares[from as usize];
        let position_to = self.squares[to as usize];

        // TODO: check if there won't be check on us
        if piece.is_none()
            || (!position_to.is_none() && piece.color == position_to.color)
            || self.color_to_move != piece.color
        {
            return Err("piece is none, position_to is occupied by the same color piece or it is not your move");
        }

        match self.is_move_possible(&piece, from, to, self.squares) {
            Ok(_) => {}
            Err(e) => return Err(e),
        };

        let mut squares_copy = self.squares.clone();
        squares_copy[from as usize] = Pawn::default();
        squares_copy[to as usize] = piece;
        let mut kings_positions = self.kings_positions.clone();
        if piece.p_type == PieceType::KING {
            kings_positions.insert(piece.color, to);
        }

        if self.is_check(piece.color, squares_copy, &kings_positions) {
            return Err("there will be check after a move");
        }

        // if self.debug {
        //     println!(
        //         "check detected: {}",
        //         self.is_check(piece.color.opposite(), squares_copy, &kings_positions)
        //     )
        // }
        Ok(())
    }

    fn is_check(
        &self,
        color: Color,
        squares_copy: [Pawn; 64],
        kings_positions: &HashMap<Color, i32>,
    ) -> bool {
        // check for check
        let king_pos = kings_positions.get(&color).unwrap();
        for (inx, p) in squares_copy.iter().enumerate() {
            if color != p.color && !p.is_none() {
                if self
                    .is_move_possible(p, inx as i32, *king_pos, squares_copy)
                    .is_ok()
                {
                    return true;
                }
            }
        }
        return false;
    }

    // is_move_possible checks is move is 'physically' legit.
    fn is_move_possible(
        &self,
        piece: &Pawn,
        from: i32,
        to: i32,
        squares: [Pawn; 64],
    ) -> Result<(), &'static str> {
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
                    if !squares[from_temp as usize].is_none() {
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
        col.chars().for_each(|c| inx += letter_to_i32(&c));
        row.chars()
            .for_each(|c| inx += (c.to_digit(10).unwrap() as i32 - 1) * 8);
        inx
    }
}

fn letter_to_i32(l: &char) -> i32 {
    *l as i32 - 'a' as i32
}

#[cfg(test)]
mod tests {
    use crate::board;
    use crate::board::{Board, Color};

    #[test]
    fn block_detection() {
        let mut b = board::Board::default();
        assert_eq!(b.make_move_internal_notation("c1g5").unwrap(), ());

        b.read_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");
        assert_eq!(
            b.make_move_internal_notation("c1g5").err().unwrap(),
            "your move is blocked"
        );

        b.read_fen("q7/pppppppp/8/8/8/8/8/8");
        b.color_to_move = Color::BLACK;
        assert_eq!(
            b.make_move_internal_notation("a8a1").err().unwrap(),
            "your move is blocked"
        );
    }

    #[test]
    fn invalid_move() {
        let mut b = board::Board::default();
        b.read_fen("r7/8/8/8/8/8/8/8");
        b.color_to_move = Color::BLACK;
        assert_eq!(
            b.make_move_internal_notation("a8b1").err().unwrap(),
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
        assert_eq!(
            b.make_move_internal_notation("a8a1").err().unwrap(),
            "your move is blocked"
        );
    }

    #[test]
    fn check_after_move() {
        let mut b = board::Board::default();
        b.color_to_move = Color::BLACK;
        b.read_fen("k7/q7/8/8/8/8/R7/K7");
        assert_eq!(
            b.make_move_internal_notation("a7b7").err().unwrap(),
            "there will be check after a move"
        );

        b.read_fen("k7/q7/p7/8/8/8/R7/K7");
        assert_eq!(b.make_move_internal_notation("a7b7").is_ok(), true);
    }

    #[test]
    fn read_pgn() {
        let pgn = "1.e4 e5 2.Nf3 f6 3.Nxe5 fxe5 4.Qh5+ Ke7 5.Qxe5+ Kf7 6.Bc4+ d5 7.Bxd5+
    Kg6 8.h4 h5 9.Bxb7 Bxb7 10.Qf5+ Kh6 11.d4+ g5 12.Qf7 Qe7 13.hxg5+ Qxg5
    14.Rxh5#";
        let mut b = Board::default();
        b.allow_debug();
        assert_eq!(b.read_pgn(pgn, true).is_ok(), true);
    }

    #[test]
    fn read_pgn_kasparov_topolov() {
        let pgn = "1. e4 d6 2. d4 Nf6 3. Nc3 g6 4. Be3 Bg7 5. Qd2 c6 6. f3 b5 7. Nge2 Nbd7 8. Bh6
Bxh6 9. Qxh6 Bb7 10. a3 e5 11. O-O-O Qe7 12. Kb1 a6 13. Nc1 O-O-O 14. Nb3 exd4
15. Rxd4 c5 16. Rd1 Nb6 17. g3 Kb8 18. Na5 Ba8 19. Bh3 d5 20. Qf4+ Ka7 21. Rhe1
d4 22. Nd5 Nbxd5 23. exd5 Qd6 24. Rxd4 cxd4 25. Re7+ Kb6 26. Qxd4+ Kxa5 27. b4+
Ka4 28. Qc3 Qxd5 29. Ra7 Bb7 30. Rxb7 Qc4 31. Qxf6 Kxa3 32. Qxa6+ Kxb4 33. c3+
Kxc3 34. Qa1+ Kd2 35. Qb2+ Kd1 36. Bf1 Rd2 37. Rd7 Rxd7 38. Bxc4 bxc4 39. Qxh8
Rd3 40. Qa8 c3 41. Qa4+ Ke1 42. f4 f5 43. Kc1 Rd2 44. Qa7";
        let mut b = Board::default();
        b.allow_debug();
        assert_eq!(b.read_pgn(pgn, true).is_ok(), true);
    }

    #[test]
    fn translate_pgn_move() {
        let mut b = Board::default();
        assert_eq!(b.translate_pgn_move("Nxe5").err().unwrap(), "invalid move");
        assert_eq!(b.translate_pgn_move("Nc3").unwrap(), (1, 18));
        assert_eq!(b.translate_pgn_move("Nf3").unwrap(), (6, 21));
        assert_eq!(b.translate_pgn_move("Nc3").unwrap(), (1, 18));
        assert_eq!(b.translate_pgn_move("Na3").unwrap(), (1, 16));
        assert_eq!(b.translate_pgn_move("Nh3").unwrap(), (6, 23));

        b.read_fen("rnbqkbnr/pppppppp/8/8/8/8/8/RNBQKBNR");
        // white square bishop
        assert_eq!(b.translate_pgn_move("Be2").unwrap(), (5, 12));
        assert_eq!(b.translate_pgn_move("Bd3").unwrap(), (5, 19));
        assert_eq!(b.translate_pgn_move("Bc4").unwrap(), (5, 26));
        assert_eq!(b.translate_pgn_move("Bb5").unwrap(), (5, 33));
        assert_eq!(b.translate_pgn_move("Ba6").unwrap(), (5, 40));
    }

    #[test]
    fn translate_pgn_move_pawns() {
        let mut b = Board::default();
        assert_eq!(b.translate_pgn_move("e4").unwrap(), (12, 28));
        assert_eq!(b.translate_pgn_move("e3").unwrap(), (12, 20));

        assert_eq!(b.translate_pgn_move("a4").unwrap(), (8, 24));
        assert_eq!(b.translate_pgn_move("a3").unwrap(), (8, 16));

        assert_eq!(b.translate_pgn_move("h4").unwrap(), (15, 31));
        assert_eq!(b.translate_pgn_move("h3").unwrap(), (15, 23));

        // takes
        b.read_fen("k7/8/8/8/8/p7/PPPPPPPP/K7");
        assert_eq!(b.translate_pgn_move("bxa3").unwrap(), (9, 16));

        b.read_fen("8/8/8/8/1k6/p7/PPPPPPPP/K7");
        b.allow_debug();
        assert_eq!(b.translate_pgn_move("bxa3").unwrap(), (9, 16));
    }
}

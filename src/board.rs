#![allow(warnings, unused)]

use crate::piece::{Color, Piece, PieceType};
use std::borrow::Borrow;
use std::cmp::{max, min};
use std::collections::hash_map::RandomState;
use std::collections::HashMap;

#[derive(Clone, PartialEq)]
// Transition represents: from, to, promotion(if necessary).
struct Transition(usize, usize, PieceType);

const OUT_OF_BOARD: usize = 64;
const DEFAULT_PROMOTION: PieceType = PieceType::NONE;

impl Transition {
    fn default() -> Self {
        Transition(OUT_OF_BOARD, OUT_OF_BOARD, DEFAULT_PROMOTION) // invalid transition
    }

    fn remove_piece(from: usize) -> Self {
        Transition(from, OUT_OF_BOARD, DEFAULT_PROMOTION)
    }

    fn new(from: usize, to: usize) -> Self {
        Transition(from, to, DEFAULT_PROMOTION)
    }

    fn new_with_promotion(from: usize, to: usize, promotion: PieceType) -> Self {
        Transition(from, to, promotion)
    }

    fn is_default(&self) -> bool {
        self.0 == OUT_OF_BOARD && self.1 == OUT_OF_BOARD
    }
}

#[derive(Clone)]
pub struct Board {
    squares: [Piece; 64], // 0 is left lower corner
    color_to_move: Color,
    kings_positions: HashMap<Color, usize>,
    debug: bool,
    last_transition: Transition,
}

const FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";

impl Board {
    pub fn default() -> Board {
        let mut b = Board {
            squares: [Piece::default(); 64],
            color_to_move: Color::WHITE,
            kings_positions: HashMap::new(),
            debug: false,
            last_transition: Transition::default(),
        };
        b.read_fen(FEN);
        b
    }

    pub fn allow_debug(&mut self) {
        self.debug = true
    }

    pub fn read_fen(&mut self, fen: &str) {
        self.squares = [Piece::default(); 64]; // reset board
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
                        let inx = (rank * 8 + file) as usize;
                        let p = Piece::new(
                            piece_from_char
                                .get(&char::to_ascii_lowercase(&c))
                                .unwrap()
                                .clone(),
                            color,
                        );
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
        let mut game = String::from(pgn.replace("\n", " ").replace("  ", " "));
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

            match self.make_pgn_move(chess_move) {
                Err(e) => return Err(e),
                _ => {}
            }

            if self.debug {
                println!("making {} move, eval: {}", chess_move, self.calc_eval());
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
        let transitions = match self.translate_pgn_move(m) {
            Ok(transitions) => transitions,
            Err(err) => return Err(err),
        };

        // check if castle
        if transitions.len() == 2 {
            // king transition will be always first index
            if self.squares[transitions.get(0).unwrap().0].p_type == PieceType::KING
                && self.squares[transitions.get(1).unwrap().0].p_type == PieceType::ROOK
            {
                return if self
                    .validate_castle(transitions.get(0).unwrap().0, transitions.get(1).unwrap().0)
                {
                    for t in transitions {
                        self.make_move(t, false);
                    }
                    self.swap_color_to_move();
                    Ok(())
                } else {
                    Err("invalid castle")
                };
            }
        }

        for t in transitions {
            match self.validate_move(t.0, t.1) {
                Ok(r) => {
                    match r {
                        Some(additional_transition) => {
                            self.make_move(additional_transition, false);
                        }
                        None => {}
                    }
                    self.make_move(t, true);
                    return Ok(());
                }
                _ => {}
            };
        }
        Err("invalid move")
    }

    fn validate_castle(&self, king_pos: usize, rook_pos: usize) -> bool {
        if !self.squares[king_pos].has_moved && !self.squares[rook_pos].has_moved {
            for inx in min(king_pos, rook_pos) + 1..max(king_pos, rook_pos) {
                if !self.squares[inx].is_none() {
                    return false;
                }
            }
            return true;
        }
        return false;
    }

    fn make_move(&mut self, tr: Transition, swap_color: bool) {
        let from = tr.0;
        let to = tr.1;

        if to == OUT_OF_BOARD {
            self.squares[from] = Piece::default();
        } else {
            self.squares[to] = self.squares[from];
            self.squares[to].has_moved = true;
            if tr.2 != DEFAULT_PROMOTION {
                self.squares[to].p_type = tr.2;
            }
            self.squares[from] = Piece::default();
            if swap_color {
                self.swap_color_to_move();
            }
            if self.squares[to].p_type == PieceType::KING {
                self.kings_positions.insert(self.squares[to].color, to);
            }
            self.last_transition = Transition::new(from, to);
        }
    }

    fn swap_color_to_move(&mut self) {
        self.color_to_move = self.color_to_move.opposite();
    }

    // translate_move gets algebraic notation and parses it to vec of possible 'from' -> 'to' move
    // e.g. Nxe5, Qh5+, g5, hxg5+
    fn translate_pgn_move(&mut self, m: &str) -> Result<Vec<Transition>, &'static str> {
        if m == "O-O" {
            return if self.color_to_move == Color::BLACK {
                Ok(vec![Transition::new(60, 62), Transition::new(63, 61)])
            } else {
                Ok(vec![Transition::new(4, 6), Transition::new(7, 5)])
            };
        } else if m == "O-O-O" {
            return if self.color_to_move == Color::BLACK {
                Ok(vec![Transition::new(60, 58), Transition::new(56, 59)])
            } else {
                Ok(vec![Transition::new(4, 2), Transition::new(0, 3)])
            };
        }

        let mut pawn_move = false; // is pawn move?
        let mut promotion = PieceType::NONE; // is pawn promotion?
        let pawn_letters = vec!["a", "b", "c", "d", "e", "f", "g", "h"];
        let mut m = m.replace("x", "").replace("+", "").replace("#", "");

        for l in &pawn_letters {
            if m.starts_with(l) {
                let temp_m = m.to_owned();
                // handle promotion e.g. hxg8=Q
                if temp_m.contains("=") {
                    let (f, p) = temp_m.split_once("=").unwrap();
                    m = String::from(f);
                    promotion = PieceType::from_sign(p);
                }
                pawn_move = true;
                break;
            }
        }

        let mut transitions = Vec::new();

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
                "R" => {
                    if second.len() != 2 {
                        let mut chars = second.chars();
                        additional_info = chars.next().unwrap().to_string();
                        second = chars.as_str();
                    }
                    PieceType::ROOK
                }
                "K" => PieceType::KING,
                _ => return Err("invalid piece"),
            };
            places = self.find_piece_places(piece_to_find, self.color_to_move, additional_info);
            direction = self.translate_position(second);
        }
        for p in &places {
            transitions.push(Transition::new_with_promotion(*p, direction, promotion));
        }
        return Ok(transitions);
    }

    fn find_piece_places(
        &self,
        piece_type: PieceType,
        color: Color,
        additional_info: String,
    ) -> Vec<usize> {
        let mut places: Vec<usize> = Vec::new();

        self.squares.iter().enumerate().for_each(|(i, p)| {
            if p.p_type == piece_type && p.color == color {
                if additional_info.len() == 1 {
                    // there's additional info
                    let info = additional_info.chars().next().unwrap();
                    if info.is_digit(10) {
                        // check for row
                        let row = info.to_digit(10).unwrap() as usize;
                        if (row - 1) * 8 >= i && row * 8 < i {
                            places.push(i);
                        }
                    } else {
                        // check for column
                        let column = letter_to_i32(&info);
                        let possible_indexes: Vec<usize> =
                            (1..9).map(|x| (column + 8 * (x - 1)) as usize).collect();
                        if possible_indexes.contains(&i) {
                            places.push(i)
                        }
                    }
                } else {
                    places.push(i)
                }
            }
        });
        places
    }

    // find_pawn_places takes e.g. 'e' and returns all pawn position that is on 'e' line
    fn find_pawn_places(&self, line: &str) -> Vec<usize> {
        let mut places: Vec<usize> = Vec::new();
        if line.len() != 1 {
            panic!("line len must be 1")
        }
        let mut inx = 0;
        line.chars().for_each(|c| inx = c as i32 - 'a' as i32); // only 1 iteration

        for i in 0..7 {
            let index = (inx + 8 * i) as usize;
            let p = self.squares[index];
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
    // pub fn make_move_internal_notation(&mut self, m: &str) -> Result<(), &'static str> {
    //     let (first, second) = m.split_at(2);
    //     let first_pos = self.translate_position(first);
    //     let second_pos = self.translate_position(second);
    //
    //     self.validate_move(first_pos, second_pos)
    // }

    // validate_move validates if move is legit. It checks every aspect of a game.
    fn validate_move(
        &mut self,
        from: usize,
        to: usize,
    ) -> Result<Option<Transition>, &'static str> {
        let piece = self.squares[from];
        let position_to = self.squares[to];

        // TODO: check if there won't be check on us
        if piece.is_none()
            || (!position_to.is_none() && piece.color == position_to.color)
            || self.color_to_move != piece.color
        {
            return Err("piece is none, position_to is occupied by the same color piece or it is not your move");
        }

        let mut additional_transition = Transition::default(); // possible additional transition
        match self.is_move_possible(&piece, from, to, self.squares) {
            Ok(r) => match r {
                Some(t) => additional_transition = t,
                None => {}
            },
            Err(e) => return Err(e),
        };

        let mut squares_copy = self.squares.clone();
        let to = to as usize;
        squares_copy[from as usize] = Piece::default();
        squares_copy[to] = piece;
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
        if additional_transition.is_default() {
            Ok(None)
        } else {
            Ok(Some(additional_transition))
        }
    }

    fn is_check(
        &self,
        color: Color,
        squares_copy: [Piece; 64],
        kings_positions: &HashMap<Color, usize>,
    ) -> bool {
        // check for check
        let king_pos = kings_positions.get(&color).unwrap();
        for (inx, p) in squares_copy.iter().enumerate() {
            if color != p.color && !p.is_none() {
                if self
                    .is_move_possible(p, inx, *king_pos, squares_copy)
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
        piece: &Piece,
        from: usize,
        to: usize,
        squares: [Piece; 64],
    ) -> Result<Option<Transition>, &'static str> {
        let available_moves = piece.get_moves(from);
        let transition = to as i32 - from as i32;
        if !available_moves.contains(&transition) {
            return Err("that piece cannot make moves like that!");
        }

        if piece.p_type == PieceType::PAWN {
            if (transition == 8 || transition == -8) && !squares[to].is_none() {
                return Err("pawn cannot move to occupied place");
            }
            return match self.check_en_passant(piece, from, to, transition, squares) {
                Ok(r) => Ok(r),
                Err(err) => Err(err),
            };
        }

        // check if there's no other piece on your way
        if piece.is_sliding() {
            let to = to as i32;
            let from = from as i32;

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
        Ok(None)
    }

    fn check_en_passant(
        &self,
        piece: &Piece,
        from: usize,
        to: usize,
        transition: i32,
        squares: [Piece; 64],
    ) -> Result<Option<Transition>, &'static str> {
        if (transition == 7 || transition == -7 || transition == -9 || transition == 9)
            && squares[to].is_none()
        {
            let mut check_opposite_pawn_position = 0;
            let mut check_opposite_pawn_position_from = 0;
            // check en passant
            if transition > 0 {
                // check if below 'to' is pawn with opposite color
                check_opposite_pawn_position = to - 8;
                check_opposite_pawn_position_from = to + 8;
            } else {
                // check if above 'to' is pawn with opposite color
                check_opposite_pawn_position = to + 8;
                check_opposite_pawn_position_from = to - 8;
            }
            let c_piece = squares[check_opposite_pawn_position];
            if c_piece.p_type != PieceType::PAWN {
                return Ok(None);
            }
            if c_piece.color != piece.color.opposite() {
                return Err("invalid en passant");
            }
            // check if that pawn made 2 moves before
            if self.last_transition
                == Transition::new(
                    check_opposite_pawn_position_from,
                    check_opposite_pawn_position,
                )
            {
                return Ok(Some(Transition::remove_piece(check_opposite_pawn_position)));
            }
        }
        Ok(None)
    }

    fn translate_position(&self, pos: &str) -> usize {
        let mut inx: i32 = 0;
        let (col, row) = pos.split_at(1);
        col.chars().for_each(|c| inx += letter_to_i32(&c));
        row.chars()
            .for_each(|c| inx += (c.to_digit(10).unwrap() as i32 - 1) * 8);
        inx as usize
    }

    // calc_eval calculates value of pieces
    fn calc_eval(&self) -> i32 {
        return self
            .squares
            .iter()
            .filter(|x| x.p_type != PieceType::NONE)
            .map(|x| {
                if x.color == Color::WHITE {
                    x.p_type.points()
                } else {
                    x.p_type.points() * -1
                }
            })
            .sum();
    }
}

fn letter_to_i32(l: &char) -> i32 {
    *l as i32 - 'a' as i32
}

#[cfg(test)]
mod tests {
    use crate::board;
    use crate::board::{Board, Color};

    // #[test]
    // fn block_detection() {
    //     let mut b = board::Board::default();
    //     b.read_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");
    //     assert_eq!(
    //         b.make_move_internal_notation("c1g5").err().unwrap(),
    //         "your move is blocked"
    //     );
    //
    //     b.read_fen("q7/pppppppp/8/8/8/8/8/8");
    //     b.color_to_move = Color::BLACK;
    //     assert_eq!(
    //         b.make_move_internal_notation("a8a1").err().unwrap(),
    //         "your move is blocked"
    //     );
    // }

    // #[test]
    // fn invalid_move() {
    //     let mut b = board::Board::default();
    //     b.read_fen("r7/8/8/8/8/8/8/8");
    //     b.color_to_move = Color::BLACK;
    //     assert_eq!(
    //         b.make_move_internal_notation("a8b1").err().unwrap(),
    //         "that piece cannot make moves like that!"
    //     );
    // }

    #[test]
    fn king_position() {
        let b = board::Board::default();
        assert_eq!(*b.kings_positions.get(&Color::BLACK).unwrap(), 60);
        assert_eq!(*b.kings_positions.get(&Color::WHITE).unwrap(), 4);
    }

    // #[test]
    // fn blocked_move() {
    //     let mut b = board::Board::default();
    //     b.color_to_move = Color::BLACK;
    //
    //     b.read_fen("r7/p7/8/8/8/8/8/8");
    //     assert_eq!(
    //         b.make_move_internal_notation("a8a1").err().unwrap(),
    //         "your move is blocked"
    //     );
    // }

    // #[test]
    // fn check_after_move() {
    //     let mut b = board::Board::default();
    //     b.color_to_move = Color::BLACK;
    //     b.read_fen("k7/q7/8/8/8/8/R7/K7");
    //     assert_eq!(
    //         b.make_move_internal_notation("a7b7").err().unwrap(),
    //         "there will be check after a move"
    //     );
    //
    //     b.read_fen("k7/q7/p7/8/8/8/R7/K7");
    //     assert_eq!(b.make_move_internal_notation("a7b7").is_ok(), true);
    // }

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
    fn test_pgn_with_en_passant() {
        let pgn = "1. e4 d5 2. exd5 Qxd5 3. Nc3 Qa5 4. d3 c6 5. Bd2 Qc7 6. Qe2 Bd7 7. O-O-O Na6 8.
Nf3 O-O-O 9. h4 Nf6 10. h5 e6 11. Ne5 g5 12. hxg6 hxg6 13. Rxh8 Bg7 14. Rxd8+
Kxd8 15. Nxf7+ Kc8 16. Qxe6 Bxe6 17. Ne4 Nxe4 18. dxe4 Bxf7 19. Bxa6 bxa6 20.
Bf4 Qxf4+ 21. Kb1";
        let mut b = Board::default();
        b.allow_debug();
        assert_eq!(b.read_pgn(pgn, true).is_ok(), true);
    }

    #[test]
    fn test_pgn_with_promotion() {
        let pgn = "1. e4 f5 2. exf5 g6 3. fxg6 Nc6 4. gxh7 d6 5. hxg8=Q Be6 6. Qh5+ Kd7 7. Qxe6+
Kxe6 8. Qg4+ Kd5 9. Nc3+ Kc5 10. Qc4+ Kb6 11. Qb5#";
        let mut b = Board::default();
        b.allow_debug();
        assert_eq!(b.read_pgn(pgn, true).is_ok(), true);
    }

    // #[test]
    // fn translate_pgn_move() {
    //     let mut b = Board::default();
    //     assert_eq!(b.translate_pgn_move("Nxe5").unwrap(), (vec![1, 6], 36));
    //     assert_eq!(b.translate_pgn_move("Nc3").unwrap(), (vec![1, 6], 18));
    //     assert_eq!(b.translate_pgn_move("Nf3").unwrap(), (vec![1, 6], 21));
    //     assert_eq!(b.translate_pgn_move("Nc3").unwrap(), (vec![1, 6], 18));
    //     assert_eq!(b.translate_pgn_move("Na3").unwrap(), (vec![1, 6], 16));
    //     assert_eq!(b.translate_pgn_move("Nh3").unwrap(), (vec![1, 6], 23));
    //     //
    //     b.read_fen("rnbqkbnr/pppppppp/8/8/8/8/8/RNBQKBNR");
    //     // white square bishop
    //     assert_eq!(b.translate_pgn_move("Be2").unwrap(), (vec![2, 5], 12));
    //     assert_eq!(b.translate_pgn_move("Bd3").unwrap(), (vec![2, 5], 19));
    //     assert_eq!(b.translate_pgn_move("Bc4").unwrap(), (vec![2, 5], 26));
    //     assert_eq!(b.translate_pgn_move("Bb5").unwrap(), (vec![2, 5], 33));
    //     assert_eq!(b.translate_pgn_move("Ba6").unwrap(), (vec![2, 5], 40));
    // }
    //
    // #[test]
    // fn translate_pgn_move_pawns() {
    //     let mut b = Board::default();
    //     assert_eq!(b.translate_pgn_move("e4").unwrap(), (vec![12], 28));
    //     assert_eq!(b.translate_pgn_move("e3").unwrap(), (vec![12], 20));
    //
    //     assert_eq!(b.translate_pgn_move("a4").unwrap(), (vec![8], 24));
    //     assert_eq!(b.translate_pgn_move("a3").unwrap(), (vec![8], 16));
    //
    //     assert_eq!(b.translate_pgn_move("h4").unwrap(), (vec![15], 31));
    //     assert_eq!(b.translate_pgn_move("h3").unwrap(), (vec![15], 23));
    //
    //     // takes
    //     b.read_fen("k7/8/8/8/8/p7/PPPPPPPP/K7");
    //     assert_eq!(b.translate_pgn_move("bxa3").unwrap(), (vec![9], 16));
    //
    //     b.read_fen("8/8/8/8/1k6/p7/PPPPPPPP/K7");
    //     b.allow_debug();
    //     assert_eq!(b.translate_pgn_move("bxa3").unwrap(), (vec![9], 16));
    // }

    #[test]
    fn test_validate_castle() {
        let mut b = Board::default();
        b.read_fen("8/8/8/8/8/8/8/R3K3");
        assert_eq!(b.validate_castle(4, 0), true);
        b.read_fen("8/8/8/8/8/8/8/4K2R");
        assert_eq!(b.validate_castle(4, 7), true);
        b.read_fen("r3k3/8/8/8/8/8/8");
        assert_eq!(b.validate_castle(60, 56), true);
        b.read_fen("4k2r/8/8/8/8/8/8/8");
        assert_eq!(b.validate_castle(60, 63), true);

        b.read_fen("8/8/8/8/8/8/8/R2PK3");
        assert_eq!(b.validate_castle(4, 0), false);
        b.read_fen("8/8/8/8/8/8/8/4K1PR");
        assert_eq!(b.validate_castle(4, 7), false);
        b.read_fen("r2pk3/8/8/8/8/8/8");
        assert_eq!(b.validate_castle(60, 56), false);
        b.read_fen("4kp1r/8/8/8/8/8/8/8");
        assert_eq!(b.validate_castle(60, 63), false);
    }
}

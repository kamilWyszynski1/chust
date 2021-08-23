use std::collections::HashMap;
use std::ops::Add;
use std::borrow::Borrow;

fn main() {
    let mut b = Board::default();
    b.read_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");
    println!("{}", b.visualize())
}

#[derive(Clone, Copy)]
enum Color {
    NONE,
    BLACK,
    WHITE,
}

#[derive(Clone, Copy)]
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
struct Pawn(PawnType, Color);

impl Pawn {
    fn default() -> Self {
        Pawn(PawnType::NONE, Color::NONE)
    }

    fn visualize(&self) -> String {
        let ch = match self.0 {
            PawnType::NONE => "",
            PawnType::KING => "k",
            PawnType::PAWN => "p",
            PawnType::KNIGHT => "n",
            PawnType::BISHOP => "b",
            PawnType::ROOK => "r",
            PawnType::QUEEN => "q",
        };

        return match self.1 {
            Color::NONE => "x".to_string(),
            Color::BLACK => ch.to_string(),
            Color::WHITE => ch.to_uppercase(),
        };
    }
}

struct Board {
    squares: [Pawn; 64], // 0 is left lower corner
}

impl Board {
    fn default() -> Board {
        Board {
            squares: [Pawn::default(); 64],
        }
    }

    fn read_fen(&mut self, fen: &str) {
        let piece_from_char: HashMap<char, PawnType> = [
            ('r', PawnType::ROOK),
            ('k', PawnType::KING),
            ('p', PawnType::PAWN),
            ('q', PawnType::QUEEN),
            ('b', PawnType::BISHOP),
            ('n', PawnType::KNIGHT),
        ].iter()
        .cloned()
        .collect();

        let mut rank: usize = 7;
        let mut file: usize = 0;

        for (i, c) in fen.chars().enumerate() {
            match c {
                '/' => {
                    file = 0;
                    rank -= 1;
                }
                _ => {
                    if c.is_digit(10) {
                        file += c.to_digit(10).unwrap() as usize;
                    } else {
                        self.squares[rank * 8 + file] = Pawn(
                            piece_from_char.get(&char::to_ascii_lowercase(&c)).unwrap().clone(),
                            match c.is_lowercase() {
                                true => Color::BLACK,
                                false => Color::WHITE,
                            },
                        );
                        file += 1;
                    }
                }
            }
        }
    }

    fn visualize(&self) -> String {
        let mut rank = 7;
        let mut file = 0;
        let mut board = String::new();

        for i in 0..8 {
            for j in 0..8 {
                board.push_str(self.squares[8 * rank + file].visualize().as_str());
                file += 1;
            }
            if rank == 0 {
                break
            }
            board.push_str("\n");
            rank -= 1;
            file = 0;
        }
        board
    }
}

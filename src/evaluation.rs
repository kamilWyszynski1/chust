use crate::board::Board;
use crate::piece::{Color, Piece, PieceType};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::ops::Add;

fn simple_eval(game: [Piece; 64]) -> f32 {
    return game
        .iter()
        .filter(|x| x.p_type != PieceType::NONE)
        .map(|x| {
            if x.color == Color::WHITE {
                x.p_type.points() as f32
            } else {
                (x.p_type.points() * -1) as f32
            }
        })
        .sum();
}

pub trait Evaluator {
    // evaluate returns evaluation of game board. Positive value is advantage of white color.
    fn evaluate(&self, board: &Board) -> f32;
}

// SimpleEvaluator evaluates game based on only material.
pub struct SimpleEvaluator {}

impl Evaluator for SimpleEvaluator {
    fn evaluate(&self, board: &Board) -> f32 {
        return simple_eval(board.squares);
    }
}

// MaterialMobilityEvaluator evaluates game based on material and mobility.
//
// f(p) = 200(K-K')
//        + 9(Q-Q')
//        + 5(R-R')
//        + 3(B-B' + N-N')
//        + 1(P-P')
//        - 0.5(D-D' + S-S' + I-I')
//        + 0.1(M-M') + ...
//
// KQRBNP = number of kings, queens, rooks, bishops, knights and pawns
// D,S,I = doubled, blocked and isolated pawns
// M = Mobility (the number of legal moves)
pub struct MaterialMobilityEvaluator {}

impl Evaluator for MaterialMobilityEvaluator {
    fn evaluate(&self, board: &Board) -> f32 {
        let se = simple_eval(board.squares);
        let ebp = self.eval_bad_pawns(board.squares);
        let mob = self.eval_mobility(board);

        return se - ebp + mob;
    }
}

const PAWN_EVAL_MODIFIER: f32 = 0.5;
const MOBILITY_EVAL_MODIFIER: f32 = 0.1;

impl MaterialMobilityEvaluator {
    // get_pawn_negative_eval sums negative pawns locations and returns evaluation.
    fn eval_bad_pawns(&self, game: [Piece; 64]) -> f32 {
        let d = self.count_doubled_pawns(game);
        let b = self.count_blocked_pawns(game);
        let i = self.count_isolated_pawns(game);

        return (d.0 + b.0 + i.0 - d.1 + b.1 + i.1) as f32 * PAWN_EVAL_MODIFIER;
    }

    // get_pawns_map maps pawns location to its columns.
    fn get_pawns_map(&self, game: [Piece; 64]) -> HashMap<Color, HashMap<usize, i32>> {
        let mut wm = HashMap::new();
        let mut bm = HashMap::new();

        game.iter()
            .enumerate()
            .map(|(inx, p)| (inx, p))
            .filter(|(_, p)| p.p_type == PieceType::PAWN)
            .for_each(|(inx, p)| {
                if p.color == Color::WHITE {
                    *wm.entry(inx % 8).or_insert(0) += 1;
                } else {
                    *bm.entry(inx % 8).or_insert(0) += 1;
                }
            });
        let mut col_map: HashMap<Color, HashMap<usize, i32>> = HashMap::new();
        col_map.insert(Color::WHITE, wm);
        col_map.insert(Color::BLACK, bm);
        return col_map;
    }
    // count_doubled_pawns calculates how many pawns are based on the same column for both colors.
    // value for white color is returned first.
    //
    // e.g. 3 pawn on b, 1 on c, 1 on d, 2 on e -> 5
    fn count_doubled_pawns(&self, game: [Piece; 64]) -> (i32, i32) {
        let col_map = self.get_pawns_map(game);
        return (
            col_map
                .get(&Color::WHITE)
                .unwrap()
                .values()
                .into_iter()
                .filter(|x| x > &&1)
                .sum(),
            col_map
                .get(&Color::BLACK)
                .unwrap()
                .values()
                .into_iter()
                .filter(|x| x > &&1)
                .sum(),
        );
    }

    // count_isolated_pawns counts isolated pawns for each color.
    fn count_isolated_pawns(&self, game: [Piece; 64]) -> (i32, i32) {
        fn count_per_color(m: &HashMap<usize, i32>) -> i32 {
            let mut w = 0;

            for i in 1..7 {
                let v_before = m.get(&(i - 1 as usize));
                let v = m.get(&(i as usize));
                let v_after = m.get(&(i + 1 as usize));

                if (v.is_some() && *v.unwrap() != 0)
                    && (v_before.is_none() || (v_before.is_some() && *v_before.unwrap() == 0))
                    && (v_after.is_none() || (v_after.is_some() && *v_after.unwrap() == 0))
                {
                    w += *v.unwrap();
                }
            }
            return w;
        }
        let col_map = self.get_pawns_map(game);

        return (
            count_per_color(col_map.get(&Color::WHITE).unwrap().borrow()),
            count_per_color(col_map.get(&Color::BLACK).unwrap().borrow()),
        );
    }

    // count_blocked_pawns counts blocked pawns for each color.
    // pawn is blocked when it cannot move forward.
    fn count_blocked_pawns(&self, game: [Piece; 64]) -> (i32, i32) {
        let mut w = 0;
        let mut b = 0;

        game.iter()
            .enumerate()
            .map(|(inx, p)| (inx, p))
            .filter(|(_, p)| p.p_type == PieceType::PAWN)
            .for_each(|(inx, p)| {
                if p.color == Color::WHITE {
                    if !game[inx + 8].is_none() {
                        w += 1;
                    }
                } else {
                    if !game[inx - 8].is_none() {
                        b += 1;
                    }
                }
            });

        return (w, b);
    }

    fn eval_mobility(&self, board: &Board) -> f32 {
        fn eval_mobility_for_color(board: &mut Board, color: Color) -> f32 {
            let mut eval: f32 = 0.0;
            board.color_to_move = color;
            board
                .squares
                .iter()
                .enumerate()
                .map(|(inx, p)| (inx, p))
                .filter(|(_, p)| p.color == color)
                .for_each(|(inx, p)| {
                    let possible_moves = p.get_moves(inx);
                    for m in &possible_moves {
                        match board.validate_move(inx, (inx as i32 + m) as usize) {
                            Ok(_) => {
                                eval += 1.0;
                            }
                            Err(_) => continue,
                        }
                    }
                });
            return eval;
        }
        let mut b_clone = board.clone();
        return (eval_mobility_for_color(&mut b_clone, Color::WHITE)
            - eval_mobility_for_color(&mut b_clone, Color::BLACK))
            * MOBILITY_EVAL_MODIFIER;
    }
}

#[cfg(test)]
mod tests {
    use crate::board::Board;
    use crate::evaluation::{Evaluator, MaterialMobilityEvaluator};
    use crate::piece::{Color, Piece, PieceType};

    #[test]
    fn test_isolated_pawns() {
        let m = MaterialMobilityEvaluator {};
        let mut game = [Piece::default(); 64];
        game[1] = Piece::new(PieceType::PAWN, Color::WHITE);
        game[13] = Piece::new(PieceType::PAWN, Color::WHITE);
        game[5] = Piece::new(PieceType::PAWN, Color::WHITE);
        game[6] = Piece::new(PieceType::PAWN, Color::WHITE);

        assert_eq!(m.count_isolated_pawns(game), (1, 0));

        let mut game = [Piece::default(); 64];
        game[1] = Piece::new(PieceType::PAWN, Color::WHITE);
        game[17] = Piece::new(PieceType::PAWN, Color::WHITE);
        game[14] = Piece::new(PieceType::PAWN, Color::WHITE);
        game[6] = Piece::new(PieceType::PAWN, Color::WHITE);
        game[3] = Piece::new(PieceType::PAWN, Color::WHITE);
        game[4] = Piece::new(PieceType::PAWN, Color::WHITE);

        assert_eq!(m.count_isolated_pawns(game), (4, 0));
    }

    #[test]
    fn test_count_double_pawns() {
        let m = MaterialMobilityEvaluator {};
        let mut game = [Piece::default(); 64];
        game[1] = Piece::new(PieceType::PAWN, Color::WHITE);
        game[17] = Piece::new(PieceType::PAWN, Color::WHITE);
        game[14] = Piece::new(PieceType::PAWN, Color::WHITE);
        game[6] = Piece::new(PieceType::PAWN, Color::WHITE);
        game[3] = Piece::new(PieceType::PAWN, Color::WHITE);
        game[4] = Piece::new(PieceType::PAWN, Color::WHITE);
        assert_eq!(m.count_doubled_pawns(game), (4, 0));
    }

    #[test]
    fn test_count_blocked_pawns() {
        let m = MaterialMobilityEvaluator {};
        let mut game = [Piece::default(); 64];
        game[1] = Piece::new(PieceType::PAWN, Color::WHITE);
        game[17] = Piece::new(PieceType::PAWN, Color::WHITE);

        game[9] = Piece::new(PieceType::PAWN, Color::BLACK);
        game[25] = Piece::new(PieceType::PAWN, Color::BLACK);
        assert_eq!(m.count_blocked_pawns(game), (2, 2));
    }

    #[test]
    fn test_material_mobility_eval() {
        let pgn = "1. e4 d5 2. exd5 Qxd5 3. Nc3 Qa5 4. d3 c6 5. Bd2 Qc7 6. Qe2 Bd7 7. O-O-O Na6 8.
Nf3 O-O-O 9. h4 Nf6 10. h5 e6 11. Ne5 g5 12. hxg6 hxg6 13. Rxh8 Bg7 14. Rxd8+
Kxd8 15. Nxf7+ Kc8 16. Qxe6 Bxe6 17. Ne4 Nxe4 18. dxe4 Bxf7 19. Bxa6 bxa6 20.
Bf4 Qxf4+ 21. Kb1";
        let mut b = Board::default();
        b.read_pgn(pgn, true);
        let m = MaterialMobilityEvaluator {};
        let mut e: f32 = 0.0;
        for i in 0..1000 {
            e = m.evaluate(&b);
        }
        println!("{}", e)
    }
}

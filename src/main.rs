mod board;

fn main() {
    let mut b = board::Board::default();
    b.read_fen("rnbqkbnr/pppppppp/8/8/8/8/PPP1PPPP/RNBQKBNR");
    match b.make_move("c1g5") {
        Ok(_) => {}
        Err(e) => {println!("{}", e)}
    }
}


mod board;
mod piece;

fn main() {
    let a: Vec<i32> = (1..9).into_iter().map(|x| x * 2).collect();
    println!("{:?}", a)
}

mod board;
mod piece;

struct A {
    a: i32,
    b: bool,
}
impl A {
    fn new(a: i32, b: bool) -> Self {
        A { a, b }
    }
}

fn main() {
    let v = vec![A::new(10, false), A::new(20, true), A::new(25, true)];
    let val: i32 = v.iter().map(|x| if x.b { x.a } else { x.a * -1 }).sum();
    println!("{}", val)
}

use std::fs;

mod parse;

fn main() {
    let map = fs::read("maps/Cat_Isle.bin").unwrap();
    println!("{:?}", parse::parse(&map));
}

mod parse;
mod statistics;
mod viewer;

use std::fs;

use viewer::view_map;

fn main() {
    let map = fs::read("maps/Cat_Isle.bin").unwrap();
    let map = parse::parse(&map).unwrap();

    view_map(&map);
}

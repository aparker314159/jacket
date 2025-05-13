extern crate chumsky;
extern crate dynasmrt;

use chumsky::Parser;
mod parser;
mod jit;


fn main() {

    let src_file = match std::env::args().nth(1) {
        Some(s) => s,
        None => panic!("No source file provided!"),
    };

    let src = std::fs::read_to_string(src_file).unwrap();

    // For now don't worry about #lang racket
    let ast = parser::parse_expr().padded().parse(&src);
    let binding = jit::JIT::compile(&ast.unwrap());
    let ret = binding.run();

    match ret {
        Ok(_) => print!("done yippee"),
        Err(_) => print!("whoops :("),
    }
    // println!("{:?}", ast);
}

mod parser;





fn main() {

    let src_file = match std::env::args().nth(1) {
        Some(s) => s,
        None => panic!("No source file provided!"),
    };

    let src = std::fs::read_to_string(src_file).unwrap();



}

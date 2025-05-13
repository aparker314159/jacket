use chumsky::prelude::*;


#[derive(Clone, Debug)]
pub enum Value {
    BoolV(bool),
    IntV(i64),
    CharV(char),
}

#[derive(Clone, Debug)]
pub enum Primitive {
    /* Prim 0 */
    Void,
    ReadByte,
    PeekByte,
    /* Prim 1 */
    Add1, Sub1,
    IsZero,
    IsChar,
    IntToChar, CharToInt,
    WriteByte,
    /* Prim 2 */
}

type Iden = String;


#[derive(Debug)]
pub enum Expr {
    Lit{v: Value},
    Identifier{id: Iden},
    PrimN{prim: Primitive, args: Vec<Expr>},
    If{if_: Box<Expr>, then_: Box<Expr>, else_: Box<Expr>},
    App{func: Box<Expr>, args: Vec<Expr>},
}



fn parse_bool<'src>() -> impl Parser<'src, &'src str, Value, extra::Err<Simple<'src, char>>> + Clone {
    (just("true").or(just("false"))).map(|b: &str| match b {
        "true" => Value::BoolV(true),
        "false" => Value::BoolV(false),
        _ => unreachable!(),
    })
}

fn parse_int<'src>() -> impl Parser<'src, &'src str, Value, extra::Err<Simple<'src, char>>> + Clone {
    text::int(10)
        .map(|i: &str| Value::IntV(i.parse().unwrap()))
} 

fn parse_char<'src>() -> impl Parser<'src, &'src str, Value, extra::Err<Simple<'src, char>>> + Clone {
    let parse_unum = just("u").ignore_then(
        text::int(10)
            .map(|i: &str| Value::CharV(char::from_u32(i.parse().unwrap()).expect("Not a valid char code!")))
    );
    let parse_c = any().map(|c: char| Value::CharV(c));
    just("#\\").ignore_then(parse_unum.or(parse_c))
}

fn parse_value<'src>() -> impl Parser<'src, &'src str, Value, extra::Err<Simple<'src, char>>> + Clone {
    parse_bool().or(parse_int()).or(parse_char())
}

fn parse_prim_name<'src>() -> impl Parser<'src, &'src str, Primitive, extra::Err<Simple<'src, char>>> + Clone {
    choice((
        just("void").to(Primitive::Void),
        just("read-byte").to(Primitive::ReadByte),
        just("peek-byte").to(Primitive::PeekByte),
        just("add1").to(Primitive::Add1),
        just("sub1").to(Primitive::Sub1),
        just("zero?").to(Primitive::IsZero),
        just("char?").to(Primitive::IsChar),
        just("integer->char").to(Primitive::IntToChar),
        just("char->int").to(Primitive::CharToInt),
        just("write-byte").to(Primitive::WriteByte),
    ))
}

fn is_identifier_char(c: &char) -> bool {
    !("()[]{}\",'`;#|\\ \t\n\r\x0B\x0C".contains(*c))
}

fn parse_iden_expr<'src>() -> impl Parser<'src, &'src str, Expr, extra::Err<Simple<'src, char>>> + Clone {
    (any().filter(is_identifier_char)).repeated().at_least(1).collect::<String>()
        .map(|s| {Expr::Identifier{id: s.to_string()}})
}

fn parse_open<'src>() -> impl Parser<'src, &'src str, char, extra::Err<Simple<'src, char>>> + Clone {
    one_of("([{") 
}

fn parse_close<'src>() -> impl Parser<'src, &'src str, char, extra::Err<Simple<'src, char>>> + Clone {
    one_of(")]}") 
}

pub fn parse_expr<'src>() -> impl Parser<'src, &'src str, Expr, extra::Err<Simple<'src, char>>> + Clone {
    recursive(|expr| {
        let parse_prim = (parse_prim_name()
            .then_ignore(text::whitespace())
            .then(expr.clone().separated_by(text::whitespace()).collect::<Vec<_>>()))
            .delimited_by(parse_open(), parse_close())
            .map(|(prim, args)| { Expr::PrimN{prim, args} });

        let parse_app = (expr.clone()
            .then_ignore(text::whitespace())
            .then(expr.clone().separated_by(text::whitespace()).collect::<Vec<_>>()))
            .delimited_by(parse_open(), parse_close())
            .map(|(e, args)| { Expr::App{func: Box::new(e), args} });

        let parse_if = (
            just("if").then_ignore(text::whitespace())
            .then(expr.clone()).then_ignore(text::whitespace())
            .then(expr.clone()).then_ignore(text::whitespace())
            .then(expr.clone()).then_ignore(text::whitespace())
            ).delimited_by(parse_open(), parse_close())
            .map(|(((_, if_), then_), else_)| {Expr::If { if_: Box::new(if_), then_: Box::new(then_), else_: Box::new(else_)}} );

        (parse_value().map(|v| { Expr::Lit{v} }))
            .or(parse_iden_expr())
            .or(parse_if)
            .or(parse_prim)
            .or(parse_app)
    })
}

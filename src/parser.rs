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

#[derive(Debug)]
pub enum Expr {
    Lit(Value),
    PrimN(Primitive, Vec<Expr>),
    App(Box<Expr>, Vec<Expr>),
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
        text::ascii::keyword("void").to(Primitive::Void),
        text::ascii::keyword("read-byte").to(Primitive::ReadByte),
        text::ascii::keyword("peek-byte").to(Primitive::PeekByte),
        text::ascii::keyword("add1").to(Primitive::Add1),
        text::ascii::keyword("sub1").to(Primitive::Sub1),
        text::ascii::keyword("zero?").to(Primitive::IsZero),
        text::ascii::keyword("char?").to(Primitive::IsChar),
        text::ascii::keyword("integer->char").to(Primitive::IntToChar),
        text::ascii::keyword("char->int").to(Primitive::CharToInt),
        text::ascii::keyword("write-byte").to(Primitive::WriteByte),
    ))
}

pub fn parse_expr<'src>() -> impl Parser<'src, &'src str, Expr, extra::Err<Simple<'src, char>>> + Clone {
    recursive(|expr| {
        let parse_prim = (parse_prim_name()
            .then_ignore(text::whitespace())
            .then(expr.clone().separated_by(text::whitespace()).collect::<Vec<_>>()))
            .delimited_by(just('('), just(')'))
            .map(|(prim, args)| { Expr::PrimN(prim, args) });

        let parse_app = (expr.clone()
            .then_ignore(text::whitespace())
            .then(expr.separated_by(text::whitespace()).collect::<Vec<_>>()))
            .delimited_by(just('('), just(')'))
            .map(|(e, args)| { Expr::App(Box::new(e), args) });

        (parse_value().map(|v| {Expr::Lit(v)})).or(parse_prim).or(parse_app)
    })
}

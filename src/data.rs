#[derive(Debug, PartialEq, Clone)]
pub enum LispDatum {
    Cons(Box<LispDatum>, Box<LispDatum>),  // These refer to (a . b), which resolves to (cons a b), and can probably be removed entirely.
    Complex(f64, f64),
    Real(f64),
    Rational(i32, i32),
    Integer(i32),
    Symbol(String),
    Nil,
}

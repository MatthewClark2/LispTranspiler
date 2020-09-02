pub enum LispDatum {
    Cons(Box<LispDatum>, Box<LispDatum>),
    Complex(f64, f64),
    Real(f64),
    Rational(i32, i32),
    Integer(i32),
    Symbol(String),
    Nil,
}

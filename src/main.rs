mod lex;
mod parse;
mod data;
mod ast;

#[link(name = "lisp", kind = "static")]
extern "C" {
    fn _rust_demo(v: f32);
}

fn main() {
    println!("Hello, world!");
    unsafe { _rust_demo(2. * std::f32::consts::PI) }
    parse::parse(lex::start("").unwrap());
}

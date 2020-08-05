#[link(name="lisp", kind="static")]
extern {
    fn _rust_demo(v: f32);
}

fn main() {
    println!("Hello, world!");
    unsafe { _rust_demo(16.2) }
}

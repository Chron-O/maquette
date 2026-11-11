// Turns ui/app.slint into Rust code, before the program is compiled.

fn main() {
    slint_build::compile("ui/app.slint").expect("ui/app.slint does not compile");
}

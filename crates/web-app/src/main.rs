use paper_immersive_reader_web_app::app::AppShell;

fn main() {
    let shell = AppShell::default();

    println!("{}", shell.title);
    for route in shell.routes {
        println!("{} {}", route.path, route.label);
    }
}

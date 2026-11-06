mod graph;
mod inspector;
mod look;
mod ui;

fn main() {
    match ui::start() {
        Ok(ui) => std::process::exit(opsis_qt::run(Box::new(ui))),
        Err(error) => {
            eprintln!("Opsis couldn't start: {error}");
            std::process::exit(1);
        }
    }
}

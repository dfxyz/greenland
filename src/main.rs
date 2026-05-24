#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> anyhow::Result<()> {
    let _guard = naive_logger::init();
    if let Err(e) = greenland::app::run() {
        eprintln!("{e:?}");
        return Err(e);
    }
    Ok(())
}

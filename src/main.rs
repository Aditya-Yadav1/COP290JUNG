// src/main.rs
mod ui;
mod sheet_functions;
mod parser;
mod calculate_functions;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Rusty Spreadsheet GUI",
        options,
        Box::new(|_cc| {
            // initialize your Sheet with desired size
            let sheet = sheet_functions::Sheet::new(100, 100);
            Box::new(ui::SpreadsheetApp::new(sheet))
        }),
    )
}
// src/main.rs
mod ui;
mod sheet_functions;
mod parser;
mod calculate_functions;
mod utils;
mod themes;
use crate::sheet_functions::Sheet;
use crate::ui::Sheets;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Rusty Spreadsheet GUI",
        options,
        Box::new(|_cc| {
            let sheet = Sheet::new(20, 10);
            let sheets = vec![Sheets{sheet:sheet.clone(), name:String::from("Sheet 1")}];
            Box::new(ui::SpreadsheetApp::new(sheets))
        }),
    )
}
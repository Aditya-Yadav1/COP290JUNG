// src/main.rs
mod app;
mod sheet_functions;
mod parser;
mod calculate_functions;
mod ui;
use ui::app_impl::{SpreadsheetApp,Sheets};
use crate::sheet_functions::Sheet;
use std::env;
fn main() -> Result<(), eframe::Error> {
    let args : Vec<String> = env::args().collect();

    if args.len()==1{
        println!("To run sheet in terminal, enter row and column , for gui enter cargo run gui");
        return Ok(());
    }

    let mode = &args[1];
    if mode == "gui"{
        let options = eframe::NativeOptions::default();
        eframe::run_native(
            "Rusty Spreadsheet GUI",
            options,
            Box::new(|_cc| {
                let sheet = Sheet::new(20, 10);
                let sheets = vec![Sheets{sheet:sheet.clone(), name:String::from("Sheet 1")}];
                Box::new(SpreadsheetApp::new(sheets))
            }),
        )
    }
    else{
        if args.len() != 3 {
            println!("Enter the row and column number");
            return Ok(());
        }

        let row = args[1].parse::<i32>().unwrap();
        let col = args[2].parse::<i32>().unwrap();
        
        let mut row_start = 0;
        let mut col_start = 0;
        let mut time = 0.0;
        let mut status = String::new();
        let mut print_enabled = true;
        status.push_str("ok");

        let mut sheet = Sheet::new(row, col);

        loop{
            if print_enabled{
                sheet_functions::print_sheet(row_start, col_start, row, col, &mut sheet);
            }
                print!("[{:.1}] ({}) > ", time, status);
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            
            let mut command = String::new();
            std::io::stdin().read_line(&mut command).unwrap();
            parser::parse_command(&command, &mut row_start, &mut col_start, &mut time, &mut status, &row, &col, &mut sheet , &mut print_enabled);
        }
    }
}
// src/main.rs
mod app;
mod sheet_functions;
mod parser;
mod calculate_functions;
mod ui;
use ui::app_impl::{SpreadsheetApp,Sheets};
use crate::sheet_functions::Sheet;
use std::env;
use egui::{FontData, FontDefinitions, FontFamily};

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    // 1) Embed your main Unicode font first:
    fonts.font_data.insert(
        "noto_sans".to_owned(),
        FontData::from_static(include_bytes!("../assets/fonts/NotoSans-VariableFont_wdth,wght.ttf")),
    );

    // 2) Embed your emoji font second:
    fonts.font_data.insert(
        "emoji".to_owned(),
        FontData::from_static(include_bytes!("../assets/fonts/Symbola-Emoji.ttf")),
    );

    // 3) For proportional text: use NOTO_SANS first, then EMOJI as fallback
    {
        let prop = fonts.families.get_mut(&FontFamily::Proportional).unwrap();
        prop.clear();
        prop.push("noto_sans".to_owned());
        prop.push("emoji".to_owned());
        // then you can let the rest (default egui fonts) follow automatically
    }

    // 4) Likewise for monospace if you need it:
    {
        let mono = fonts.families.get_mut(&FontFamily::Monospace).unwrap();
        mono.clear();
        mono.push("noto_sans".to_owned());
        mono.push("emoji".to_owned());
    }

    ctx.set_fonts(fonts);

    // bump the body‐text size so color emojis look right 
}


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
            Box::new(|cc| {
                setup_custom_fonts(&cc.egui_ctx);
                let sheet = Sheet::new(20, 20);
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


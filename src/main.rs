mod sheet_functions;
mod parser;
mod calculate_functions;
mod ui_sheet_functions;

// #[cfg(feature = "gui")]
// mod ui;
// #[cfg(feature = "gui")]
// mod app;

use std::env;
use crate::sheet_functions::Sheet;

// #[cfg(feature = "gui")]
// use ui::app_impl::{SpreadsheetApp, Sheets};
// #[cfg(feature = "gui")]
// use egui::{FontData, FontDefinitions, FontFamily};

// #[cfg(feature = "gui")]
// fn setup_custom_fonts(ctx: &egui::Context) {
//     let mut fonts = FontDefinitions::default();
//     fonts.font_data.insert(
//         "noto_sans".to_owned(),
//         FontData::from_static(include_bytes!("../assets/fonts/NotoSans-VariableFont_wdth,wght.ttf")),
//     );
//     fonts.font_data.insert(
//         "emoji".to_owned(),
//         FontData::from_static(include_bytes!("../assets/fonts/Symbola-Emoji.ttf")),
//     );

//     {
//         let prop = fonts.families.get_mut(&FontFamily::Proportional).unwrap();
//         prop.clear();
//         prop.push("noto_sans".to_owned());
//         prop.push("emoji".to_owned());
//     }

//     {
//         let mono = fonts.families.get_mut(&FontFamily::Monospace).unwrap();
//         mono.clear();
//         mono.push("noto_sans".to_owned());
//         mono.push("emoji".to_owned());
//     }

//     ctx.set_fonts(fonts);
// }

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    #[cfg(feature = "gui")]
    {
            let options = eframe::NativeOptions::default();
            return eframe::run_native(
                "Rusty Spreadsheet GUI",
                options,
                Box::new(|cc| {
                    setup_custom_fonts(&cc.egui_ctx);
                    egui_extras::install_image_loaders(&cc.egui_ctx);
                    let sheet = Sheet::new(20, 20);
                    let sheets = vec![Sheets {
                        sheet: sheet.clone(),
                        name: String::from("Sheet 1"),
                    }];
                    Box::new(SpreadsheetApp::new(sheets))
                }),
            ).map_err(|e| Box::new(e) as Box<dyn std::error::Error>);
    }

    #[cfg(feature = "no_gui")]
    {
        if args.len() != 3 {
            println!("Usage: cargo run <rows> <cols>");
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

        loop {
            if print_enabled {
                sheet_functions::print_sheet(row_start, col_start, row, col, &mut sheet);
            }

            print!("[{:.1}] ({}) > ", time, status);
            std::io::Write::flush(&mut std::io::stdout()).unwrap();

            let mut command = String::new();
            std::io::stdin().read_line(&mut command).unwrap();

            parser::parse_command(
                &command,
                &mut row_start,
                &mut col_start,
                &mut time,
                &mut status,
                &row,
                &col,
                &mut sheet,
                &mut print_enabled,
            );
        }
    }

    #[cfg(all(not(feature = "gui"), not(feature = "no_gui")))]
    {
        compile_error!("Either feature 'gui' or 'no_gui' must be enabled.");
    }

    // Ok(())
}

use std::fs::File;
use std::io::Write;
use crate::sheet_functions::Sheet;
use std::io::{BufReader, BufRead};
use crate::sheet_functions::Cell;
use crate::sheet_functions::CellInfo;
use std::collections::HashSet;
use std::fs;
use crate::app_impl::Sheets;
use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use crate::sheet_functions::OpCode;
use crate::sheet_functions::OpCode::*;
use std::string::String;

pub fn convert_to_csv(sheet: &Sheet, filename: &str) {
    let save_file_name = format!("{}.csv", filename);
    let mut file = File::create(save_file_name).unwrap();
    for row in &sheet.data {
        let row_values : Vec<String>= row.iter()
            .map(|cell| {
                if cell.is_error {
                    "Err".to_string()
                } else {
                    cell.value.to_string()
                }
            })
            .collect();
        
        let line = row_values.join(",") + "\n";
        file.write_all(line.as_bytes()).unwrap();
    }
    // file.flush().unwrap();
    // file.close().unwrap();
}


pub fn open_csv(filename: &str,sheet: &mut Sheet)-> String {
    let mut status = String::from("CSV loaded!");
    let file = match File::open(filename) {
        Ok(f) => f,
        Err(e) => {
            status = format!("Failed to open file: {}", e);
            return status; 
        }
    };
    let reader = BufReader::new(file);
    sheet.data.clear();
    let mut has_error = false;
    for (line_num, line) in reader.lines().enumerate() {
        let line = match line {
            Ok(l) => l,
            Err(_) => {
                has_error = true;
                break; 
            }
        };
        let row: Vec<Cell> = line
            .split(',')
            .map(|value| {
                let trimmed = value.trim();
                match trimmed.parse::<i32>() {
                    Ok(num) => Cell {
                        value: num,
                        string: None,
                        is_error: false,
                        op_code: NoConstraint,
                        cell1: CellInfo { row: -1, col: -1 },
                        cell2: CellInfo { row: -1, col: -1 },
                        dependencies: HashSet::new()
                    },
                    Err(_) => {
                        Cell {
                            value: 0,
                            string : None,
                            is_error: true,
                            op_code: NoConstraint,
                            cell1: CellInfo { row: -1, col: -1 },
                            cell2: CellInfo { row: -1, col: -1 },
                            dependencies: HashSet::new()
                        }
                    }
                }
            })
            .collect();
        sheet.data.push(row);
    }

    if has_error {
        status = "error loading csv".to_string();
    } else {
        status = "CSV loaded successfully".to_string();
    }

    status
}


pub fn save_sheet(sheet: &Sheet, filename: &str) {
    let json = serde_json::to_string_pretty(sheet).unwrap();
    let mut file = File::create(filename).unwrap();
    file.write_all(json.as_bytes()).unwrap();
}

pub fn load_sheet(filename: &str) -> Sheet {
    let data = fs::read_to_string(filename).unwrap();
    serde_json::from_str(&data).unwrap()
}

pub fn save_all_sheets(sheets: &Vec<Sheets>, filename: &str) {
    let mut file = File::create(filename).unwrap();
    let encoder = GzEncoder::new(file, Compression::default());
    serde_json::to_writer_pretty(encoder, sheets).unwrap();   
}

pub fn load_all_sheets(filename: &str) -> Vec<Sheets> {
    let file = File::open(filename).unwrap();
    let decoder = GzDecoder::new(file);
    serde_json::from_reader(decoder).unwrap()
}
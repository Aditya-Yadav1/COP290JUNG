use std::fs::File;
use std::io::Write;
use crate::sheet_functions::Sheet;
use std::io::{BufReader, BufRead};
use crate::sheet_functions::Cell;
use crate::sheet_functions::CellInfo;
use std::collections::HashSet;

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
    let open_file_name = format!("{}.csv", filename);
    let mut status = String::from("CSV loaded!");
    let file = match File::open(open_file_name) {
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
                        op_code: 'X',
                        cell1: CellInfo { row: -1, col: -1 },
                        cell2: CellInfo { row: -1, col: -1 },
                        dependencies: HashSet::new()
                    },
                    Err(_) => {
                        Cell {
                            value: 0,
                            string : None,
                            is_error: true,
                            op_code: 'X',
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
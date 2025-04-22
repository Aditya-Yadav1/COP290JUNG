use std::fs::File;
use std::io::Write;
use crate::sheet_functions::Sheet;
use std::io::{BufReader, BufRead};
use crate::sheet_functions::Cell;
use crate::sheet_functions::CellInfo;
use std::collections::HashSet;
use crate::ui::app_impl::Sheets;
use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use crate::sheet_functions::{self};
use crate::sheet_functions::OpCode;
use crate::sheet_functions::OpCode::*;
use crate::ui::app_impl::{Action,SpreadsheetApp};
use std::string::String;
use crate::sheet_functions::col_num_to_col_name;

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


pub fn open_csv(filename: &str,app: &mut SpreadsheetApp)-> String {
    let row= 10;
    let col= 10;
    app.new_sheet_name = filename.to_string();
    app.create_new_sheet(row,col);
    let sheet = &mut app.sheets[app.current_sheet_index].sheet;

    let status;
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
    for (_, line) in reader.lines().enumerate() {
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
        sheet.rows = sheet.data.len() as i32;
        sheet.cols = sheet.data[0].len() as i32;
    }

    if has_error {
        status = "error loading csv".to_string();
    } else {
        status = "CSV loaded successfully".to_string();
    }

    status
}



pub fn save_all_sheets(sheets: &Vec<Sheets>, filename: &str) {
    let file = File::create(filename).unwrap();
    let encoder = GzEncoder::new(file, Compression::default());
    serde_json::to_writer_pretty(encoder, sheets).unwrap();   
}

pub fn load_all_sheets(filename: &str) -> Vec<Sheets> {
    let file = File::open(filename).unwrap();
    let decoder = GzDecoder::new(file);
    serde_json::from_reader(decoder).unwrap()
}

pub fn insert_undo(sheet_index: usize, row: i16, col: i16, previous_cell: Cell, app: &mut SpreadsheetApp) {
    sheet_functions::remove_dependency(
        &CellInfo {row: row as i16,col: col as i16},
        &mut app.sheets[sheet_index].sheet,
    );
    let curr_cell = app.sheets[sheet_index].sheet.data[row as usize][col as usize].clone();
    let old_cell = previous_cell.clone();
    if old_cell.cell1.row != -1 && old_cell.cell1.col != -1 {
        app.sheets[app.current_sheet_index].sheet.data[old_cell.cell1.row as usize][old_cell.cell1.col as usize].dependencies.insert(col as i32 * 1000 + row as i32);
    }
    if old_cell.cell2.row != -1 && old_cell.cell2.col != -1 {
        app.sheets[app.current_sheet_index].sheet.data[old_cell.cell2.row as usize][old_cell.cell2.col as usize].dependencies.insert(col as i32 * 1000 + row as i32);
    }
    app.sheets[sheet_index].sheet.data[row as usize][col as usize] = old_cell;

    app.redo_stack.push(Action::Inserted {
        sheet_index,
        row,
        col,
        previous_cell:curr_cell,
    });

    sheet_functions::recalculate_dependecy(CellInfo {row: row as i16,col: col as i16}, &mut app.sheets[sheet_index].sheet);
}

pub fn cut_undo(sheet_index: usize, row1: i16, col1: i16, previous_cell1: Cell, row2: i16, col2: i16, previous_cell2: Cell, app: &mut SpreadsheetApp) {
    sheet_functions::remove_dependency(
        &CellInfo {row: row1 as i16,col: col1 as i16},
        &mut app.sheets[sheet_index].sheet,
    );
    sheet_functions::remove_dependency(
        &CellInfo {row: row2 as i16,col: col2 as i16},
        &mut app.sheets[sheet_index].sheet,
    );
    let curr_cell1 = app.sheets[sheet_index].sheet.data[row1 as usize][col1 as usize].clone();
    let curr_cell2 = app.sheets[sheet_index].sheet.data[row2 as usize][col2 as usize].clone();
    app.sheets[sheet_index].sheet.data[row1 as usize][col1 as usize] = previous_cell1;
    app.sheets[sheet_index].sheet.data[row2 as usize][col2 as usize] = previous_cell2;
    let cell1_1 = app.sheets[sheet_index].sheet.data[row1 as usize][col1 as usize].cell1.clone();
    let cell1_2 = app.sheets[sheet_index].sheet.data[row1 as usize][col1 as usize].cell2.clone();
    let cell2_1 = app.sheets[sheet_index].sheet.data[row2 as usize][col2 as usize].cell1.clone();
    let cell2_2 = app.sheets[sheet_index].sheet.data[row2 as usize][col2 as usize].cell2.clone();
    if cell1_1.row != -1 && cell1_1.col != -1 {
       app.sheets[sheet_index].sheet.data[cell1_1.row as usize][cell1_1.col as usize].dependencies.insert(col1 as i32 * 1000 + row1 as i32);
    }
    if cell1_2.row != -1 && cell1_2.col != -1 {
        app.sheets[sheet_index].sheet.data[cell1_2.row as usize][cell1_2.col as usize].dependencies.insert(col1 as i32 * 1000 + row1 as i32);
    }
    if cell2_1.row != -1 && cell2_1.col != -1 {
        app.sheets[sheet_index].sheet.data[cell2_1.row as usize][cell2_1.col as usize].dependencies.insert(col2 as i32 * 1000 + row2 as i32);
    }
    if cell2_2.row != -1 && cell2_2.col != -1 {
        app.sheets[sheet_index].sheet.data[cell2_2.row as usize][cell2_2.col as usize].dependencies.insert(col2 as i32 * 1000 + row2 as i32);
    }
    sheet_functions::recalculate_dependecy(CellInfo {row: row1 as i16,col: col1 as i16}, &mut app.sheets[sheet_index].sheet);
    sheet_functions::recalculate_dependecy(CellInfo {row: row2 as i16,col: col2 as i16}, &mut app.sheets[sheet_index].sheet);
    app.redo_stack.push(Action::CutAction {
        sheet_index : sheet_index,
        row1 : row1,
        col1 : col1,
        previous_cell1 : curr_cell1,
        row2 : row2,
        col2 : col2,
        previous_cell2 : curr_cell2,
    });
}

pub fn get_cell_formula(row: i16, col: i16, cell: &Cell) -> String {
    match cell.op_code {
        OpCode::NoConstraint => format!("{}{}={}", col_num_to_col_name(col as i32), row + 1, cell.value),
        OpCode::CellEqualsCell => format!(
            "{}{}={}{}",
            col_num_to_col_name(col as i32),
            row + 1,
            col_num_to_col_name(cell.cell1.col as i32),
            cell.cell1.row + 1
        ),
        OpCode::CellPlusCell | OpCode::CellMinusCell | OpCode::CellTimesCell | OpCode::CellDivideCell => {
            let operator = match cell.op_code {
                OpCode::CellPlusCell => "+",
                OpCode::CellMinusCell => "-",
                OpCode::CellTimesCell => "*",
                OpCode::CellDivideCell => "/",
                _ => unreachable!(),
            };
            format!(
                "{}{}={}{}{}{}{}",
                col_num_to_col_name(col as i32),
                row + 1,
                col_num_to_col_name(cell.cell1.col as i32),
                cell.cell1.row + 1,
                operator,
                col_num_to_col_name(cell.cell2.col as i32),
                cell.cell2.row + 1
            )
        }
        OpCode::CellPlusConstant | OpCode::CellMinusConstant | OpCode::CellTimesConstant | OpCode::CellDivideConstant => {
            let operator = match cell.op_code {
                OpCode::CellPlusConstant => "+",
                OpCode::CellMinusConstant => "-",
                OpCode::CellTimesConstant => "*",
                OpCode::CellDivideConstant => "/",
                _ => unreachable!(),
            };
            let constant = ((cell.cell2.row as i32) << 16) | (cell.cell2.col as i32 & 0xFFFF); // Recover 32-bit constant
            format!(
                "{}{}={}{}{}{}",
                col_num_to_col_name(col as i32),
                row + 1,
                col_num_to_col_name(cell.cell1.col as i32),
                cell.cell1.row + 1,
                operator,
                constant
            )
        }
        OpCode::ConstantDividesCell => {
            let constant = ((cell.cell2.row as i32) << 16) | (cell.cell2.col as i32 & 0xFFFF); // Recover 32-bit constant
            format!(
                "{}{}={}/{}{}",
                col_num_to_col_name(col as i32),
                row + 1,
                constant,
                col_num_to_col_name(cell.cell1.col as i32),
                cell.cell1.row + 1
            )
        }
        OpCode::Sum | OpCode::Min | OpCode::Max | OpCode::Avg | OpCode::Stdev => {
            let func_name = match cell.op_code {
                OpCode::Sum => "SUM",
                OpCode::Min => "MIN",
                OpCode::Max => "MAX",
                OpCode::Avg => "AVG",
                OpCode::Stdev => "STDEV",
                _ => unreachable!(),
            };
            format!(
                "{}{}={}({}{}:{}{})",
                col_num_to_col_name(col as i32),
                row + 1,
                func_name,
                col_num_to_col_name(cell.cell1.col as i32),
                cell.cell1.row + 1,
                col_num_to_col_name(cell.cell2.col as i32),
                cell.cell2.row + 1
            )
        }
        OpCode::Sleep => format!(
            "{}{}=SLEEP({}{})",
            col_num_to_col_name(col as i32),
            row + 1,
            col_num_to_col_name(cell.cell1.col as i32),
            cell.cell1.row + 1,
        ),
        OpCode::String => format!(
            "{}{}=\"string\"",
            col_num_to_col_name(col as i32),
            row + 1, 
        ),
    }
}

use std::thread; 
use std::time::Duration;
use regex::Regex;
use crate::sheet_functions;
use crate::sheet_functions::col_name_to_col_num; 
use crate::sheet_functions::CellInfo;
use crate::calculate_functions::compute_cell;   
use crate::sheet_functions::is_valid_cell;
use crate::sheet_functions::Sheet; 
use std::time::Instant;
use crate::sheet_functions::OpCode;
use crate::sheet_functions::OpCode::*;
use std::string::String;

pub fn get_op_code(op_code: char, constopcell: bool) -> OpCode {
    // function to get opcode for the case of int op cell or cell op int
    match op_code {
        '+' => CellPlusConstant,
        '-' => CellMinusConstant,
        '*' => CellTimesConstant,
        '/' => if constopcell { ConstantDividesCell } else { CellDivideConstant },
        _ => NoConstraint
    }
}

pub fn get_op_code2(op_code: char) -> OpCode {
    match op_code {
        '+' => CellPlusCell,
        '-' => CellMinusCell,
        '*' => CellTimesCell,
        '/' => CellDivideCell,
        _ => NoConstraint
    }
}


pub fn func_to_op_code(func: &str) -> OpCode {
    // function for getting opcode for the case of func(cell:cell)
    match func {
        "SUM" => Sum,
        "MIN" => Min,
        "MAX" => Max,
        "AVG" => Avg,
        "STDEV" => Stdev,
        _ => NoConstraint
    }
}


fn remove_space(command: &mut String) {
    *command = command.chars().filter(|&c| c != ' ').collect();
}

pub fn parse_command(command:&str, row_start:&mut i32, col_start:&mut i32, time:&mut f32, status:&mut String, total_rows:&i32, total_cols:&i32, sheet: &mut Sheet,print_enabled : &mut bool){
    let mut command = command.trim().to_string();
    let mut sleep_timer = 0;
    let start_time = Instant::now();
    let re_scroll_to = Regex::new(r"^scroll_to([A-Z]+)(\d+)$").unwrap();
    let re_cell_eq_int_op_int = Regex::new(r"^([A-Z]+)(\d+)=(\d+)([+\-*/])(\d+)$").unwrap();
    let re_cell_eq_cell_op_cell = Regex::new(r"^([A-Z]+)(\d+)=([A-Z]+)(\d+)([+\-*/])([A-Z]+)(\d+)$").unwrap();
    let re_cell_eq_int_op_cell = Regex::new(r"^([A-Z]+)(\d+)=(\d+)([+\-*/])([A-Z]+)(\d+)$").unwrap();
    let re_cell_eq_cell_op_int = Regex::new(r"^([A-Z]+)(\d+)=([A-Z]+)(\d+)([+\-*/])(\d+)$").unwrap();
    let re_cell_eq_func = Regex::new(r"^([A-Z]+)(\d+)=([A-Z]+)\(([A-Z]+)(\d+):([A-Z]+)(\d+)\)$").unwrap();
    let re_cell_eq_int = Regex::new(r"^([A-Z]+)(\d+)=(-?\d+)$").unwrap();
    let re_cell_eq_cell = Regex::new(r"^([A-Z]+)(\d+)=([A-Z]+)(\d+)$").unwrap();
    let re_sleep_int = Regex::new(r"^([A-Z]+)(\d+)=SLEEP\((\d+)\)$").unwrap();
    let re_sleep_cell = Regex::new(r"^([A-Z]+)(\d+)=SLEEP\(([A-Z]+)(\d+)\)$").unwrap();
    let re_string_cell = Regex::new(r"^([A-Z]+)([0-9]+)=([a-z_.,:;\-/@#$%^&!?()\[\]{}<>\s]*)$").unwrap();
    let re_sort = Regex::new(r"^SORT\(\s*([A-Z]+)(\d+)\s*:\s*([A-Z]+)(\d+)\s*;\s*([A-Z]+)(\d+)\s*:\s*([A-Z]+)(\d+)\s*;\s*(asc|desc)\s*\)$").unwrap();
    remove_space(&mut command);

   
    if command == "w" {
            *row_start = std::cmp::max(0, *row_start - 10);
            *status = String::from("ok");
        }
    else if command == "s" {
            *row_start = std::cmp::min(*row_start + 10, total_rows - 10);   
            *status = String::from("ok");
        }
    else if command == "a" {
            *col_start = std::cmp::max(0, *col_start - 10);
            *status = String::from("ok");
        }
    else if command == "d" {
            *col_start = std::cmp::min(*col_start + 10, total_cols - 10);
            *status = String::from("ok");
        }
    else if command == "q" {
            std::process::exit(0);
        }
    else if command == "enable_output" {
            *print_enabled = true;
            *status = String::from("ok");
        }
    else if command == "disable_output" {
            *print_enabled = false;
            *status = String::from("ok");
        }
    // scroll_to
    else if let Some(caps) = re_scroll_to.captures(&command) {
        let col_str = caps.get(1).unwrap().as_str(); // Extract column letters
        let row_num: i32 = caps.get(2).unwrap().as_str().parse().unwrap(); // Extract row number

        let c = col_name_to_col_num(col_str); // Convert column letters to number
        let r = row_num - 1; // Convert to 0-based index
        if is_valid_cell(r, c, *total_rows, *total_cols) {
            *col_start = c;
            *row_start = r;
            *status = String::from("ok");
        } else {
            *status = String::from("err");
        }
        return; // Exit early after handling scroll_to
    }
    // Cell = String
    else if let Some(caps) = re_string_cell.captures(&command) {
        *time = 0.0;
        let ref_col = caps.get(1).unwrap().as_str();
        let ref_row: i32 = caps.get(2).unwrap().as_str().parse().unwrap();
        let value = caps.get(3).unwrap().as_str();
        let col = col_name_to_col_num(ref_col);
        let row = ref_row - 1; 
        if is_valid_cell(row, col, *total_rows, *total_cols) {
            sheet.data[row as usize][col as usize].string = Some(value.to_string());
            sheet.data[row as usize][col as usize].value = 0;
            sheet.data[row as usize][col as usize].is_error = false;
            *status = String::from("ok");
            let cell = CellInfo { row: row as i16, col: col as i16 };
            let cell1 = CellInfo { row: -1, col: -1 };
            let cell2 = CellInfo { row: -1, col: -1 };
            sheet_functions::add_constraints(cell, cell1, cell2, String, sheet, status, &mut sleep_timer);
             
        } else {
            *status = String::from("Invalid cmd");
        }
    }

    // Cell = int op int
    else if let Some(caps) = re_cell_eq_int_op_int.captures(&command) {
        *time = 0.0;
        let ref_col = caps.get(1).unwrap().as_str();
        let ref_row: i32 = caps.get(2).unwrap().as_str().parse().unwrap();
        let val1: i32 = caps.get(3).unwrap().as_str().parse().unwrap();
        let op = caps.get(4).unwrap().as_str().chars().next().unwrap();
        let val2: i32 = caps.get(5).unwrap().as_str().parse().unwrap();

        
        let col = col_name_to_col_num(ref_col);
        let row = ref_row - 1;
        
        if is_valid_cell(row, col, *total_rows, *total_cols) && (op == '+' || op == '-' || op == '*' || op == '/') {
            let op_code = get_op_code(op, false);
            let (ans, err) = compute_cell(op_code, val1, val2, status);
            sheet.data[row as usize][col as usize].value = ans;
            sheet.data[row as usize][col as usize].is_error = err;
            sheet.data[row as usize][col as usize].string = None;

            let cell = CellInfo { row: row as i16, col: col as i16 };
            let cell1 = CellInfo { row: -1, col: -1 };
            let cell2 = CellInfo { row: -1, col: -1 };
            let op_code = NoConstraint;
            sheet_functions::add_constraints(cell, cell1, cell2, op_code, sheet, status, &mut sleep_timer);
            *status = String::from("ok");
        } else {
            *status = String::from("Invalid cmd");
        }
    }
    // Cell = cell op cell
    else if let Some(caps) = re_cell_eq_cell_op_cell.captures(&command) {
        *time = 0.0;
        let ref_col = caps.get(1).unwrap().as_str();
        let ref_row: i32 = caps.get(2).unwrap().as_str().parse().unwrap();
        let val_col1 = caps.get(3).unwrap().as_str();
        let val_row1: i32 = caps.get(4).unwrap().as_str().parse().unwrap();
        let op = caps.get(5).unwrap().as_str().chars().next().unwrap();
        let val_col2 = caps.get(6).unwrap().as_str();
        let val_row2: i32 = caps.get(7).unwrap().as_str().parse().unwrap();
        
        let col = col_name_to_col_num(ref_col);
        let row = ref_row - 1;
        let col1 = col_name_to_col_num(val_col1);
        let col2 = col_name_to_col_num(val_col2);
        
        if is_valid_cell(row, col, *total_rows, *total_cols) && 
           is_valid_cell(val_row1 - 1, col1, *total_rows, *total_cols) && 
           is_valid_cell(val_row2 - 1, col2, *total_rows, *total_cols) && 
           (op == '+' || op == '-' || op == '*' || op == '/') {
            let op_code = get_op_code2(op);
            let cell = CellInfo { row: row as i16, col: col as i16 };
            let cell1 = CellInfo { row: val_row1 as i16 - 1, col: col1 as i16 };
            let cell2 = CellInfo { row: val_row2 as i16 - 1, col: col2 as i16 }; 
            sheet_functions::add_constraints(cell, cell1, cell2, op_code, sheet, status, &mut sleep_timer);
             
        } else {
            *status = String::from("Invalid cmd");
        }
    }
    // Cell = int op cell
    else if let Some(caps) = re_cell_eq_int_op_cell.captures(&command) {
        *time = 0.0;
        let ref_col = caps.get(1).unwrap().as_str();
        let ref_row  = caps.get(2).unwrap().as_str().parse::<i32>().unwrap();
        let val1  = caps.get(3).unwrap().as_str().parse::<i32>().unwrap();
        let op = caps.get(4).unwrap().as_str().chars().next().unwrap();
        let val_col1 = caps.get(5).unwrap().as_str();
        let val_row1  = caps.get(6).unwrap().as_str().parse::<i32>().unwrap();
        
        let col  = col_name_to_col_num(ref_col) ;
        let row  = ref_row as i32 - 1;
        let col1   = col_name_to_col_num(val_col1)  ;
        
        if is_valid_cell(row as i32, col as i32, *total_rows, *total_cols) && 
           is_valid_cell(val_row1 - 1, col1 as i32, *total_rows, *total_cols) && 
           (op == '+' || op == '-' || op == '*' || op == '/') { 
            // Splitting the constant into two 16 bit variables
            let const1 = val1 & 0xFFFF;
            let const2 = (val1 >> 16) & 0xFFFF;
            
            let op_code = get_op_code(op, true);
            let cell = CellInfo { row: row as i16, col: col as i16 };
            let cell1 = CellInfo { row: val_row1 as i16 - 1, col: col1 as i16 };
            let cell2 = CellInfo { row: const2 as i16, col: const1 as i16 };
            
            sheet_functions::add_constraints(cell, cell1, cell2, op_code, sheet, status, &mut sleep_timer);
            
        } else {
            *status = String::from("Invalid cmd");
        }
    }
    // Cell = cell op int
    else if let Some(caps) = re_cell_eq_cell_op_int.captures(&command) {
        *time = 0.0;
        let ref_col = caps.get(1).unwrap().as_str();
        let ref_row: i32 = caps.get(2).unwrap().as_str().parse().unwrap();
        let val_col1 = caps.get(3).unwrap().as_str();
        let val_row1: i32 = caps.get(4).unwrap().as_str().parse().unwrap();
        let op = caps.get(5).unwrap().as_str().chars().next().unwrap();
        let val1: i32 = caps.get(6).unwrap().as_str().parse().unwrap();
        
        let col = col_name_to_col_num(ref_col);
        let row = ref_row - 1;
        let col1 = col_name_to_col_num(val_col1);
        
        if is_valid_cell(row, col, *total_rows, *total_cols) && 
           is_valid_cell(val_row1 - 1, col1, *total_rows, *total_cols) && 
           (op == '+' || op == '-' || op == '*' || op == '/') { 
            // Splitting the constant into two 16 bit variables
            let const1 = val1 & 0xFFFF;
            let const2 = (val1 >> 16) & 0xFFFF; 
            let op_code = get_op_code(op, false);
            let cell = CellInfo { row: row as i16, col: col as i16 };
            let cell1 = CellInfo { row: val_row1 as i16 - 1, col: col1 as i16 };
            let cell2 = CellInfo { row: const2 as i16, col: const1 as i16 };
            
            sheet_functions::add_constraints(cell, cell1, cell2, op_code, sheet, status, &mut sleep_timer);
          
        } else {
            *status = String::from("Invalid cmd");
        }
    }
// Cell = func(cell:cell)
else if let Some(caps) = re_cell_eq_func.captures(&command) {
    *time = 0.0;
    let ref_col = caps.get(1).unwrap().as_str();
    let ref_row: i32 = caps.get(2).unwrap().as_str().parse().unwrap();
    let func_name = caps.get(3).unwrap().as_str();
    let val_col1 = caps.get(4).unwrap().as_str();
    let val_row1: i32 = caps.get(5).unwrap().as_str().parse().unwrap();
    let val_col2 = caps.get(6).unwrap().as_str();
    let val_row2: i32 = caps.get(7).unwrap().as_str().parse().unwrap();

    let col = col_name_to_col_num(ref_col);
    let row = ref_row - 1;
    let col1 = col_name_to_col_num(val_col1);
    let col2 = col_name_to_col_num(val_col2);

    let row1 = val_row1 - 1;
    let row2 = val_row2 - 1;

    if is_valid_cell(row, col, *total_rows, *total_cols) &&
       is_valid_cell(row1, col1, *total_rows, *total_cols) &&
       is_valid_cell(row2, col2, *total_rows, *total_cols) &&
       val_row1 <= val_row2 && col1 <= col2 {
 
            let op_code = func_to_op_code(func_name);
            if op_code == NoConstraint {
                *status = String::from("Invalid cmd");
                return;
            }
            let cell = CellInfo { row: row as i16, col: col as i16 };
            let cell1 = CellInfo { row: row1 as i16, col: col1 as i16 };
            let cell2 = CellInfo { row: row2 as i16, col: col2 as i16 };

            sheet_functions::add_constraints(cell, cell1, cell2, op_code, sheet, status, &mut sleep_timer);
        // }
    } else {
        *status = String::from("Invalid cmd");
    }
}

    // Cell = int
    else if let Some(caps) = re_cell_eq_int.captures(&command) {
        *time = 0.0;
        let ref_col = caps.get(1).unwrap().as_str();
        let ref_row: i32 = caps.get(2).unwrap().as_str().parse().unwrap();
        let val1: i32 = caps.get(3).unwrap().as_str().parse().unwrap_or_else(|_| {
            *status = String::from("Invalid cmd");
            return 0;
        });
        
        let col = col_name_to_col_num(ref_col);
        let row = ref_row - 1; 
        if is_valid_cell(row, col, *total_rows, *total_cols) { 
            sheet.data[row as usize][col as usize].string = None;
            sheet.data[row as usize][col as usize].value = val1;
            sheet.data[row as usize][col as usize].is_error = false;
            *status = String::from("ok");
            let cell = CellInfo { row: row as i16, col: col as i16 };
            let cell1 = CellInfo { row: -1, col: -1 };
            let cell2 = CellInfo { row: -1, col: -1 };
            let op_code = NoConstraint;
            sheet_functions::add_constraints(cell, cell1, cell2, op_code, sheet, status, &mut sleep_timer);
 
        } else {
            *status = String::from("Invalid cmd");
        }
    }
    // Cell = cell
    else if let Some(caps) = re_cell_eq_cell.captures(&command) {
        *time = 0.0;
        let ref_col = caps.get(1).unwrap().as_str();
        let ref_row: i32 = caps.get(2).unwrap().as_str().parse().unwrap();
        let val_col1 = caps.get(3).unwrap().as_str();
        let val_row1: i32 = caps.get(4).unwrap().as_str().parse().unwrap();
        
        let col = col_name_to_col_num(ref_col);
        let row = ref_row - 1;
        let col1 = col_name_to_col_num(val_col1);
        let row1 = val_row1 - 1;
        
        if is_valid_cell(row, col, *total_rows, *total_cols) && is_valid_cell(row1, col1, *total_rows, *total_cols) {
            
                let cell = CellInfo { row: row as i16, col: col as i16 };
                let cell1 = CellInfo { row: row1 as i16, col: col1 as i16 };
                let cell2 = CellInfo { row: -1, col: -1 };
                let op_code = CellEqualsCell;
    
                sheet_functions::add_constraints(cell, cell1, cell2, op_code, sheet, status, &mut sleep_timer);
        } else {
            *status = String::from("Invalid cmd");
        }
    }
    // Cell = SLEEP(int)
    else if let Some(caps) = re_sleep_int.captures(&command) {
        let ref_col = caps.get(1).unwrap().as_str();
        let ref_row: i32 = caps.get(2).unwrap().as_str().parse().unwrap();
        let val1: i32 = caps.get(3).unwrap().as_str().parse().unwrap();

        let col = col_name_to_col_num(ref_col);
        let row = ref_row - 1;

        if is_valid_cell(row, col, *total_rows, *total_cols) {
            *status = String::from("ok");
            let cell = CellInfo { row: row as i16, col: col as i16 };
            let cell1 = CellInfo { row: -1, col: -1 };
            let cell2 = CellInfo { row: -1, col: -1 };
            sheet.data[row as usize][col as usize].value = val1;
            sheet.data[row as usize][col as usize].is_error = false;
            sheet.data[row as usize][col as usize].string = None; 
            let op_code = NoConstraint;
            sheet_functions::add_constraints(cell, cell1, cell2, op_code, sheet, status, &mut sleep_timer);
            if val1 >= 0 {
                *time = val1 as f32;
                sleep_timer += val1;
                // thread::sleep(Duration::from_secs(val1 as u64));
            }
        } else {
            *status = String::from("err");
        }
    }
    // Cell = SLEEP(cell)
    else if let Some(caps) = re_sleep_cell.captures(&command) {
        let ref_col = caps.get(1).unwrap().as_str();
        let ref_row: i32 = caps.get(2).unwrap().as_str().parse().unwrap();
        let val_col1 = caps.get(3).unwrap().as_str();
        let val_row1: i32 = caps.get(4).unwrap().as_str().parse().unwrap();

        let col = col_name_to_col_num(ref_col);
        let row = ref_row - 1;
        let col1 = col_name_to_col_num(val_col1);
        let row1 = val_row1 - 1;

        if is_valid_cell(row, col, *total_rows, *total_cols) && is_valid_cell(row1, col1, *total_rows, *total_cols) {
            let c1_value;
            let c1 = &sheet.data[row1 as usize][col1 as usize];
            if c1.string.is_some() {
                c1_value = 0;
            } else {
                c1_value = c1.value;
            }

            let cell = CellInfo { row: row as i16, col: col as i16 };
            let cell1 = CellInfo { row: row1 as i16, col: col1 as i16 };
            let cell2 = CellInfo { row: -1, col: -1 };
            let op_code = Sleep;
            sheet_functions::add_constraints(cell, cell1, cell2, op_code, sheet, status, &mut sleep_timer);
            if c1_value >= 0 {
                sleep_timer += c1_value;
                *status = String::from("ok");
                // thread::sleep(Duration::from_secs(c1_value as u64));
            } else {
                *status = String::from("err");
            }
        } else {
            *status = String::from("err");
        }
    } 
    else if let Some(caps) = re_sort.captures(&command) {
        let ref_col1 = caps.get(1).unwrap().as_str();
        let ref_row1: i32 = caps.get(2).unwrap().as_str().parse().unwrap();
        let ref_col2 = caps.get(3).unwrap().as_str();
        let ref_row2: i32 = caps.get(4).unwrap().as_str().parse().unwrap();
        let ref_col3 = caps.get(5).unwrap().as_str();
        let ref_row3: i32 = caps.get(6).unwrap().as_str().parse().unwrap();
        let ref_col4 = caps.get(7).unwrap().as_str();
        let ref_row4: i32 = caps.get(8).unwrap().as_str().parse().unwrap();

        let col1 = col_name_to_col_num(ref_col1);
        let row1 = ref_row1 - 1;
        let col2 = col_name_to_col_num(ref_col2);
        let row2 = ref_row2 - 1;
        let col3 = col_name_to_col_num(ref_col3);
        let row3 = ref_row3 - 1;
        let col4 = col_name_to_col_num(ref_col4);
        let row4 = ref_row4 - 1;

        println!("{} {} {} {} {} {} {} {}", col1, row1, col2, row2, col3, row3, col4, row4);
           
    }
    else {
        *status = String::from("err");
    }

    let elapsed = start_time.elapsed();
    if elapsed.as_secs_f32() > sleep_timer as f32 {
        *time = elapsed.as_secs_f32();
    }
    else{
        *time = sleep_timer as f32;
        thread::sleep(Duration::from_secs(sleep_timer as u64 - elapsed.as_secs()));
    }
}

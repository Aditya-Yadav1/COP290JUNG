use std::thread;
use std::env;
use std::time::Duration;
use regex::Regex;
use crate::sheet_functions;
use crate::sheet_functions::col_name_to_col_num;
use crate::sheet_functions::CellInfo;
use crate::calculate_functions::compute_cell;   
use crate::sheet_functions::is_valid_cell;
use crate::sheet_functions::Sheet;
/*
    = ->cell
    + -> cell+cell
    - ->cell-cell
    * -> cell*cell
    / -> cell/cell
    S -> SUM
    m -> MIN
    M -> MAX
    A -> AVG
    D -> STDEV
    Z -> sleep(cell)
    X -> const op const,const , sleep(const)
    p -> const+cell or cell+const
    s -> const-cell or cell-const
    u -> const*cell or cell*const
    d -> cell/const
    b -> const/cell
*/

pub fn get_op_code(op_code: char, constopcell: bool) -> char {
    // function to get opcode for the case of int op cell or cell op int
    match op_code {
        '+' => 'p',
        '-' => 's',
        '*' => 'u',
        '/' => if constopcell { 'b' } else { 'd' },
        _ => '\0'
    }
}

pub fn get_op_code_rev(op_code: char) -> char {
    // function to get operation from opcode for the case of int op cell or cell op int
    match op_code {
        'p' => '+',
        's' => '-',
        'u' => '*',
        'd' => '/',
        'b' => '/',
        _ => '\0'
    }
}

pub fn func_to_op_code(func: &str) -> char {
    // function for getting opcode for the case of func(cell:cell)
    match func {
        "SUM" => 'S',
        "MIN" => 'm',
        "MAX" => 'M',
        "AVG" => 'A',
        "STDEV" => 'D',
        _ => 'X'
    }
}
 

fn remove_space(command: &mut String) {
    *command = command.chars().filter(|&c| c != ' ').collect();
}

pub fn parse_command(command:&str, row_start:&mut i32, col_start:&mut i32, time:&mut f32, status:&mut String, total_rows:&i32, total_cols:&i32, sheet: &mut Sheet){
    let mut command = command.trim().to_string();
    // remove_space(&mut command);

    match command.as_str() {
        "w" =>{
            *row_start = std::cmp::max(0, *row_start - 10);
            *status = String::from("ok");
            return;
        }
        "s" =>{
            *row_start = std::cmp::min(*row_start + 10, total_rows - 1);   
            *status = String::from("ok");
            return;
        }
        "a" =>{
            *col_start = std::cmp::max(0, *col_start - 10);
            *status = String::from("ok");
            return;
        }
        "d" =>{
            *col_start = std::cmp::min(*col_start + 10, total_cols - 1);
            *status = String::from("ok");
            return;
        }
        "q" =>{
            std::process::exit(0);
        }
        "enable_output" => {
            // TODO: Implement this
            *status = String::from("ok");
            return;
        },
        "disable_output" => {
            // TODO: Implement this
            *status = String::from("ok");
            return;
        },
        _ => {} // Continue with complex patterns
    }
    let re_scroll_to = Regex::new(r"^scroll_to\(([A-Z]+)(\d+)\)$").unwrap();
    let re_cell_eq_int_op_int = Regex::new(r"^([A-Z]+)(\d+)=(\d+)([+\-*/])(\d+)$").unwrap();
    let re_cell_eq_cell_op_cell = Regex::new(r"^([A-Z]+)(\d+)=([A-Z]+)(\d+)([+\-*/])([A-Z]+)(\d+)$").unwrap();
    let re_cell_eq_int_op_cell = Regex::new(r"^([A-Z]+)(\d+)=(\d+)([+\-*/])([A-Z]+)(\d+)$").unwrap();
    let re_cell_eq_cell_op_int = Regex::new(r"^([A-Z]+)(\d+)=([A-Z]+)(\d+)([+\-*/])(\d+)$").unwrap();
    let re_cell_eq_func = Regex::new(r"^([A-Z]+)(\d+)=([A-Z]+)\(([A-Z]+)(\d+):([A-Z]+)(\d+)\)$").unwrap();
    let re_cell_eq_int = Regex::new(r"^([A-Z]+)(\d+)=(\d+)$").unwrap();
    let re_cell_eq_cell = Regex::new(r"^([A-Z]+)(\d+)=([A-Z]+)(\d+)$").unwrap();
    let re_sleep_int = Regex::new(r"^([A-Z]+)(\d+)=SLEEP\((\d+)\)$").unwrap();
    let re_sleep_cell = Regex::new(r"^([A-Z]+)(\d+)=SLEEP\(([A-Z]+)(\d+)\)$").unwrap();

    // scroll_to
    if let Some(caps) = re_scroll_to.captures(&command) {
        let ref_col = caps.get(1).unwrap().as_str();
        let ref_row: i32 = caps.get(2).unwrap().as_str().parse().unwrap();
        
        let c = col_name_to_col_num(ref_col);
        let r = ref_row - 1;
        
        if is_valid_cell(r, c, *total_rows, *total_cols) {
            *col_start = c;
            *row_start = r;
            *status = String::from("ok");
        } else {
            *status = String::from("Invalid cmd");
        }
    }
    // Cell = int op int
    else if let Some(caps) = re_cell_eq_int_op_int.captures(&command) {
        let ref_col = caps.get(1).unwrap().as_str();
        let ref_row: i32 = caps.get(2).unwrap().as_str().parse().unwrap();
        let val1: i32 = caps.get(3).unwrap().as_str().parse().unwrap();
        let op = caps.get(4).unwrap().as_str().chars().next().unwrap();
        let val2: i32 = caps.get(5).unwrap().as_str().parse().unwrap();
        
        let col = col_name_to_col_num(ref_col);
        let row = ref_row - 1;
        
        if is_valid_cell(row, col, *total_rows, *total_cols) && (op == '+' || op == '-' || op == '*' || op == '/') {
            let ans = compute_cell(op, val1, val2, status);
            sheet.data[row as usize][col as usize].value = ans;
            if ans == -1 {
                sheet.data[row as usize][col as usize].is_error = true;
            }
            let cell = CellInfo { row: row as i32, col: col as i32 };
            let cell1 = CellInfo { row: -1, col: -1 };
            let cell2 = CellInfo { row: -1, col: -1 };
            let op_code = 'X';
            sheet_functions::add_constraints(cell, cell1, cell2, op_code, sheet, status, &mut 0);
            *status = String::from("ok");
        } else {
            *status = String::from("Invalid cmd");
        }
    }
    // Cell = cell op cell
    else if let Some(caps) = re_cell_eq_cell_op_cell.captures(&command) {
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
            let cell = CellInfo { row: row as i32, col: col as i32 };
            let cell1 = CellInfo { row: val_row1 as i32 - 1, col: col1 as i32 };
            let cell2 = CellInfo { row: val_row2 as i32 - 1, col: col2 as i32 };
            
            sheet_functions::add_constraints(cell, cell1, cell2, op, sheet, status, &mut 0); 
        } else {
            *status = String::from("Invalid cmd");
        }
    }
    // Cell = int op cell
    else if let Some(caps) = re_cell_eq_int_op_cell.captures(&command) {
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
            let cell = CellInfo { row: row as i32, col: col as i32 };
            let cell1 = CellInfo { row: val_row1 as i32 - 1, col: col1 as i32 };
            let cell2 = CellInfo { row: const2 as i32, col: const1 as i32 };
            
            sheet_functions::add_constraints(cell, cell1, cell2, op_code, sheet, status, &mut 0); 
        } else {
            *status = String::from("Invalid cmd");
        }
    }
    // Cell = cell op int
    else if let Some(caps) = re_cell_eq_cell_op_int.captures(&command) {
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
            println!("HAHA");
            println!("{} {} {} {}", const1, const2, val1, op);
            let op_code = get_op_code(op, false);
                            let cell = CellInfo { row: row as i32, col: col as i32 };
            let cell1 = CellInfo { row: val_row1 as i32 - 1, col: col1 as i32 };
            let cell2 = CellInfo { row: const2 as i32, col: const1 as i32 };
            
            sheet_functions::add_constraints(cell, cell1, cell2, op_code, sheet, status, &mut 0); 
        } else {
            *status = String::from("Invalid cmd");
        }
    }
    // Cell = func(cell:cell)
    else if let Some(caps) = re_cell_eq_func.captures(&command) {
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
        
        if is_valid_cell(row, col, *total_rows, *total_cols) && 
           is_valid_cell(val_row1 - 1, col1, *total_rows, *total_cols) && 
           is_valid_cell(val_row2 - 1, col2, *total_rows, *total_cols) && 
           val_row1 <= val_row2 && col1 <= col2 {
            
            let op_code = func_to_op_code(func_name);
            let cell = CellInfo { row: row as i32, col: col as i32 };
            let cell1 = CellInfo { row: val_row1 as i32 - 1, col: col1 as i32 };
            let cell2 = CellInfo { row: val_row2 as i32 - 1, col: col2 as i32 };
            
            sheet_functions::add_constraints(cell, cell1, cell2, op_code, sheet, status, &mut 0);

        } else {
            *status = String::from("Invalid cmd");
        }
    }
    // Cell = int
    else if let Some(caps) = re_cell_eq_int.captures(&command) {
        let ref_col = caps.get(1).unwrap().as_str();
        let ref_row: i32 = caps.get(2).unwrap().as_str().parse().unwrap();
        let val1: i32 = caps.get(3).unwrap().as_str().parse().unwrap();
        
        let col = col_name_to_col_num(ref_col);
        let row = ref_row - 1; 
        if is_valid_cell(row, col, *total_rows, *total_cols) { 
            sheet.data[row as usize][col as usize].value = val1;
            sheet.data[row as usize][col as usize].is_error = false;
            *status = String::from("ok");
            let cell = CellInfo { row: row as i32, col: col as i32 };
            let cell1 = CellInfo { row: -1, col: -1 };
            let cell2 = CellInfo { row: -1, col: -1 };
            let op_code = 'X';
            sheet_functions::add_constraints(cell, cell1, cell2, op_code, sheet, status, &mut 0);
 
        } else {
            *status = String::from("Invalid cmd");
        }
    }
    // Cell = cell
    else if let Some(caps) = re_cell_eq_cell.captures(&command) {
        let ref_col = caps.get(1).unwrap().as_str();
        let ref_row: i32 = caps.get(2).unwrap().as_str().parse().unwrap();
        let val_col1 = caps.get(3).unwrap().as_str();
        let val_row1: i32 = caps.get(4).unwrap().as_str().parse().unwrap();
        
        let col = col_name_to_col_num(ref_col);
        let row = ref_row - 1;
        let col1 = col_name_to_col_num(val_col1);
        let row1 = val_row1 - 1;
        
        if is_valid_cell(row, col, *total_rows, *total_cols) && is_valid_cell(row1, col1, *total_rows, *total_cols) {
            let cell = CellInfo { row: row as i32, col: col as i32 };
            let cell1 = CellInfo { row: row1 as i32, col: col1 as i32 };
            let cell2 = CellInfo { row: -1, col: -1 };
            let op_code = '=';
            sheet_functions::add_constraints(cell, cell1, cell2, op_code, sheet, status, &mut 0); 
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
            let cell = CellInfo { row: row as i32, col: col as i32 };
            let cell1 = CellInfo { row: -1, col: -1 };
            let cell2 = CellInfo { row: -1, col: -1 };
            sheet.data[row as usize][col as usize].value = val1;
            sheet.data[row as usize][col as usize].is_error = false;
            
            let op_code = 'X';
            sheet_functions::add_constraints(cell, cell1, cell2, op_code, sheet, status, &mut 0);
            if val1 >= 0 {
                thread::sleep(Duration::from_secs(val1 as u64));
            }
             
        } else {
            *status = String::from("Invalid cmd");
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
            let cell = CellInfo { row: row as i32, col: col as i32 };
            let cell1 = CellInfo { row: row1 as i32, col: col1 as i32 };
            let cell2 = CellInfo { row: -1, col: -1 };
            let op_code = 'Z';
            sheet_functions::add_constraints(cell, cell1, cell2, op_code, sheet, status, &mut 0); 
        } else {
            *status = String::from("Invalid cmd");
        }
    }
    else {
        *status = String::from("Invalid cmd");
    }
}




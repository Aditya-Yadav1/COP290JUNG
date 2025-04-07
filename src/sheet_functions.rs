use crate::calculate_functions::compute_cell;
use crate::calculate_functions::compute_range_func;
use crate::calculate_functions;
use std::collections::{HashMap, HashSet, VecDeque};


#[derive(Clone)]
pub struct CellInfo {
    pub row: i32,
    pub col: i32,
}

#[derive(Clone)]
pub struct DependencyNode {
    pub data: i32,
    pub next: Option<Box<DependencyNode>>,
}
#[derive(Clone)]
pub struct Cell {
    pub value: i32,
    pub is_error: bool,
    pub op_code: char,
    pub cell1: CellInfo,
    pub cell2: CellInfo,
    pub dependencies: Option<Box<DependencyNode>>,
}

#[derive(Clone)]
pub struct Sheet {
    pub rows: i32,
    pub cols: i32,
    pub data: Vec<Vec<Cell>>,
}


// is_valid_cell
pub fn is_valid_cell(row: i32, col: i32, total_rows: i32, total_cols: i32) -> bool {
    row >= 0 && row < total_rows && col >= 0 && col < total_cols
}           


impl Sheet {
    pub fn new(m: i32, n: i32) -> Self {
        let mut data = Vec::with_capacity(m as usize);
        for _ in 0..m {
            let mut row = Vec::with_capacity(n as usize);
            for _ in 0..n {
                row.push(Cell {
                    value: 0,
                    is_error: false,
                    op_code: 'X',
                    cell1: CellInfo { row: -1, col: -1 },
                    cell2: CellInfo { row: -1, col: -1 },
                    dependencies: None,
                });
            }
            data.push(row);
        }
        Sheet {
            rows: m,
            cols: n,
            data,
        }
    }
}



pub fn col_num_to_col_name(col_num: i32) -> String {
    let mut col_name = String::new();
    let mut col_num = col_num;
    while col_num >= 0 {
        let x = ('A' as u8 + (col_num % 26) as  u8) as char;
        col_name.push(x);
        col_num /= 26;
        col_num -= 1;
    }
    col_name = col_name.chars().rev().collect();
    return col_name;
}

pub fn col_name_to_col_num(col_name: &str) -> i32 {
    let mut col  = -1;
    for c in col_name.chars() {
        col = (col + 1) * 26;
        col += (c as u8 - 'A' as u8) as i32;
    }
    return col;
}
  
pub fn print_sheet(start_row: i32, start_col: i32, total_rows: i32, total_cols: i32, sheet: &mut Sheet) {
    let max_col_display : i32 = start_col + std::cmp::min(total_cols - start_col, 10);
    let max_row_display : i32 = start_row + std::cmp::min(total_rows - start_row, 10);
    let space : usize = 10;

    print!("{:>space$}", "");
    for i in start_col..max_col_display {
        print!("{:>space$}", col_num_to_col_name(i));
    }
    println!("");
    for i in start_row..max_row_display {
        print!("{:>space$}", i + 1);
        for j in start_col..max_col_display {
            print!("{:>space$}", sheet.data[i as usize][j as usize].value);
        }
        println!("");
    }
}



pub fn recalculate(cell: &mut Cell, sheet: &mut Sheet, sleep_timer: &mut i32) {
    if cell.op_code == '\0' {
        return;
    }

    let mut ans: i32 = 0;
    let mut calc_error = false;

    match cell.op_code {
        '=' => {
            if sheet.data[cell.cell1.row as usize][cell.cell1.col as usize].is_error {
                cell.is_error = true;
                return;
            }
            cell.is_error = false;
            ans = sheet.data[cell.cell1.row as usize][cell.cell1.col as usize].value;
        },
        '+' | '-' | '*' | '/' => {
            if sheet.data[cell.cell1.row as usize][cell.cell1.col as usize].is_error ||
               sheet.data[cell.cell2.row as usize][cell.cell2.col as usize].is_error {
                cell.is_error = true;
                return;
            }
            let val1 = sheet.data[cell.cell1.row as usize][cell.cell1.col as usize].value;
            let val2 = sheet.data[cell.cell2.row as usize][cell.cell2.col as usize].value;
            ans = compute_cell(cell.op_code, val1, val2, &mut String::new());
            cell.is_error = calc_error;
        },
        'p' | 's' | 'u' | 'd' | 'b' => {
            if sheet.data[cell.cell1.row as usize][cell.cell1.col as usize].is_error {
                cell.is_error = true;
                return;
            }
            
            let val = (cell.cell2.row as i32) << 16 | (cell.cell2.col as i32);
            ans = compute_cell(cell.op_code, 
                             sheet.data[cell.cell1.row as usize][cell.cell1.col as usize].value, 
                             val,
                             &mut String::new());
            cell.is_error = calc_error;
        },
        'Z' => {
            if sheet.data[cell.cell1.row as usize][cell.cell1.col as usize].is_error {
                cell.is_error = true;
                return;
            }
            ans = sheet.data[cell.cell1.row as usize][cell.cell1.col as usize].value;
            *sleep_timer += ans;
        },
        'S' | 'm' | 'M' | 'A' | 'D' => {
            for i in cell.cell1.row..=cell.cell2.row {
                for j in cell.cell1.col..=cell.cell2.col {
                    if sheet.data[i as usize][j as usize].is_error {
                        cell.is_error = true;
                        return;
                    }
                }
            }
            ans = compute_range_func(sheet, 
                                   cell.op_code, 
                                   cell.cell1.row as i32, 
                                   cell.cell1.col as i32,
                                   cell.cell2.row as i32, 
                                   cell.cell2.col as i32,
                                   &mut String::new());
            cell.is_error = calc_error;
        },
        _ => return,
    }
    cell.value = ans;
    calc_error = false;

}
  
pub fn recalculate_all(sheet: &mut Sheet, changed_cell: CellInfo) {
    // Build a map of cells and their dependents
    let mut dependents_map: HashMap<(i32, i32), Vec<(i32, i32)>> = HashMap::new();
    
    // First pass: build the dependency graph
    for i in 0..sheet.rows {
        for j in 0..sheet.cols {
            // Skip cells with no operations
            if sheet.data[i as usize][j as usize].op_code == '\0' || 
               sheet.data[i as usize][j as usize].op_code == 'X' {
                continue;
            }
            
            let cell = &sheet.data[i as usize][j as usize];
            
            // Add first dependency
            if cell.cell1.row >= 0 && cell.cell1.col >= 0 {
                let dep_key = (cell.cell1.row, cell.cell1.col);
                dependents_map.entry(dep_key).or_insert(Vec::new()).push((i, j));
            }
            
            // Add second dependency if it exists
            if cell.cell2.row >= 0 && cell.cell2.col >= 0 && 
               (cell.op_code == '+' || cell.op_code == '-' || 
                cell.op_code == '*' || cell.op_code == '/' || 
                cell.op_code == 'S' || cell.op_code == 'm' || 
                cell.op_code == 'M' || cell.op_code == 'A' || 
                cell.op_code == 'D') {
                
                // For range functions, add all cells in the range as dependencies
                if cell.op_code == 'S' || cell.op_code == 'm' || 
                   cell.op_code == 'M' || cell.op_code == 'A' || 
                   cell.op_code == 'D' {
                    for r in cell.cell1.row..=cell.cell2.row {
                        for c in cell.cell1.col..=cell.cell2.col {
                            let dep_key = (r, c);
                            dependents_map.entry(dep_key).or_insert(Vec::new()).push((i, j));
                        }
                    }
                } else {
                    // For regular binary operations
                    let dep_key = (cell.cell2.row, cell.cell2.col);
                    dependents_map.entry(dep_key).or_insert(Vec::new()).push((i, j));
                }
            }
        }
    }
    
    // Queue for BFS, starting from the changed cell
    let mut queue = VecDeque::new();
    queue.push_back((changed_cell.row, changed_cell.col));
    
    // Set to keep track of cells already queued
    let mut visited = HashSet::new();
    visited.insert((changed_cell.row, changed_cell.col));
    
    // Set to detect circular dependencies
    let mut circular_detected = HashSet::new();
    
    // Sleep timer for Z operations
    let mut sleep_timer = 0;
    
    // Perform BFS to update all dependent cells
    while let Some((row, col)) = queue.pop_front() {
        // Skip if this is not a changed cell
        if row != changed_cell.row || col != changed_cell.col {
            // Update the cell value
            let mut cell = sheet.data[row as usize][col as usize].clone();
            recalculate(&mut cell, sheet, &mut sleep_timer);
            sheet.data[row as usize][col as usize] = cell;
        }
        
        // Add dependents to the queue
        if let Some(deps) = dependents_map.get(&(row, col)) {
            for (dep_row, dep_col) in deps {
                // Check for circular dependency
                if circular_detected.contains(&(*dep_row, *dep_col)) {
                    // Mark the cell as having an error
                    sheet.data[*dep_row as usize][*dep_col as usize].is_error = true;
                    continue;
                }
                
                circular_detected.insert((*dep_row, *dep_col));
                
                // Only add to queue if not already visited
                if !visited.contains(&(*dep_row, *dep_col)) {
                    visited.insert((*dep_row, *dep_col));
                    queue.push_back((*dep_row, *dep_col));
                }
            }
        }
    }
}


pub fn add_constraints(curr_cell: CellInfo, cell1: CellInfo, cell2: CellInfo, op_code: char, sheet: &mut Sheet, status: &mut String, sleep_timer: &mut i32) {
    let curr_cell_row_col = curr_cell.col * 1000 + curr_cell.row;
    let mut ans = 0;
    let mut calc_error = false;
    let mut dependencies = HashSet::new();
    dependencies.insert(curr_cell_row_col);

    match op_code {
        'X' => {
            ans = sheet.data[curr_cell.row as usize][curr_cell.col as usize].value;
            // TODO: Implement remove_dependency
            let cell = &mut sheet.data[curr_cell.row as usize][curr_cell.col as usize];
            cell.op_code = op_code;
            cell.cell1 = CellInfo { row: -1, col: -1 };
            cell.cell2 = CellInfo { row: -1, col: -1 };
        },
        '=' => {
            // TODO: Implement cycle checking
            if false {
                *status = String::from("circular error");
                return;
            }
            // TODO: Implement remove_dependency
            let cell = &mut sheet.data[curr_cell.row as usize][curr_cell.col as usize];
            cell.cell1 = cell1.clone();
            cell.cell2 = CellInfo { row: -1, col: -1 };
            cell.op_code = op_code;
            
            if sheet.data[cell1.row as usize][cell1.col as usize].is_error {
                calc_error = true;
            } else {
                ans = sheet.data[cell1.row as usize][cell1.col as usize].value;
            }
            // TODO: Implement insert_into_list
        },
        'p' | 's' | 'u' | 'd' | 'b' | 'Z' => {
            // TODO: Implement cycle checking
            if false {
                *status = String::from("circular error");
                return;
            }
            // TODO: Implement remove_dependency
            let cell = &mut sheet.data[curr_cell.row as usize][curr_cell.col as usize];
            cell.cell1 = cell1.clone();
            cell.cell2 = cell2.clone();
            cell.op_code = op_code;
            
            let value = (cell2.row as i32) << 16 | (cell2.col as i32 & 0xFFFF);
            if sheet.data[cell1.row as usize][cell1.col as usize].is_error {
                calc_error = true;
            } else {
                ans = compute_cell(op_code, 
                                 sheet.data[cell1.row as usize][cell1.col as usize].value, 
                                 value,
                                 status);
            }
            // TODO: Implement insert_into_list
        },
        'S' | 'm' | 'M' | 'A' | 'D' => {
            // TODO: Implement cycle checking for range functions
            if false {
                *status = String::from("circular error");
                return;
            }
            
            for i in cell1.row..=cell2.row {
                for j in cell1.col..=cell2.col {
                    if sheet.data[i as usize][j as usize].is_error {
                        calc_error = true;
                        break;
                    }
                }
                if calc_error {
                    break;
                }
            }
            
            // TODO: Implement remove_dependency
            let cell = &mut sheet.data[curr_cell.row as usize][curr_cell.col as usize];
            cell.cell1 = cell1.clone();
            cell.cell2 = cell2.clone();
            cell.op_code = op_code;
            
            if !calc_error {
                ans = compute_range_func(sheet,
                                       op_code,
                                       cell1.row as i32,
                                       cell1.col as i32,
                                       cell2.row as i32,
                                       cell2.col as i32,
                                       status);
            }
            
            // TODO: Implement insert_into_list for range
        },
        '+' | '-' | '*' | '/' => {
            // TODO: Implement cycle checking
            if false {
                *status = String::from("circular error");
                return;
            }
            // TODO: Implement remove_dependency
            let cell = &mut sheet.data[curr_cell.row as usize][curr_cell.col as usize];
            cell.cell1 = cell1.clone();
            cell.cell2 = cell2.clone();
            cell.op_code = op_code;
            
            if sheet.data[cell1.row as usize][cell1.col as usize].is_error ||
               sheet.data[cell2.row as usize][cell2.col as usize].is_error {
                calc_error = true;
            } else {
                ans = compute_cell(op_code,
                                 sheet.data[cell1.row as usize][cell1.col as usize].value,
                                 sheet.data[cell2.row as usize][cell2.col as usize].value,
                                 status);
            }
            // TODO: Implement insert_into_list for both cells
        },
        _ => {
            *status = String::from("Invalid operation");
            return;
        }
    }

    let cell = &mut sheet.data[curr_cell.row as usize][curr_cell.col as usize];
    cell.value = ans;
    cell.is_error = calc_error;
    
    if cell.op_code == 'Z' {
        *sleep_timer += ans;
    }
    recalculate_all(sheet, curr_cell);
    // TODO: Implement topological 
}

  






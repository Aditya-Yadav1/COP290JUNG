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
pub struct Cell {
    pub value: i32,
    pub string: Option<String>,
    pub is_error: bool,
    pub op_code: char,
    pub cell1: CellInfo,
    pub cell2: CellInfo,
    pub dependencies: HashSet<i32>,  
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
                    string: None,
                    is_error: false,
                    op_code: 'X',
                    cell1: CellInfo { row: -1, col: -1 },
                    cell2: CellInfo { row: -1, col: -1 },
                    dependencies: HashSet::new()
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
            if sheet.data[i as usize][j as usize].is_error == false{
                if sheet.data[i as usize][j as usize].string.is_some() {
                    let s = sheet.data[i as usize][j as usize].string.as_ref().unwrap();
                    print!("{:>space$}", s);
                }
                else {
            print!("{:>space$}", sheet.data[i as usize][j as usize].value);}}
            else {
                print!("{:>space$}", "Err");
            }
        }
        println!("");
    }
}

pub fn check_cycle(avl_tree: &std::collections::HashMap<i32, i32>,cell1: &CellInfo,cell2: &CellInfo,) -> bool {
    let key1 = cell1.col * 1000 + cell1.row;
    if avl_tree.contains_key(&key1) {
        return true;
    }
    if cell2.col == -1 && cell2.row == -1 {
        return false;
    }
    let key2 = cell2.col * 1000 + cell2.row;
    if avl_tree.contains_key(&key2) {
        return true;
    }
    false
}
pub fn check_cycle_range_funcs(avl_tree: &std::collections::HashMap<i32, i32>, cell1: &CellInfo, cell2: &CellInfo) -> bool {
    for (&key, _) in avl_tree.iter() {
        let col = key / 1000;
        let row = key % 1000;

        if col >= cell1.col && col <= cell2.col && row >= cell1.row && row <= cell2.row {
            return true;
        }
    }
    false
}
pub fn recalculate(sheet: &mut Sheet, row: usize, col: usize, sleep_timer: &mut i32) {
    // ── 1) Extract the cell’s own metadata with one tiny mutable borrow ──
    let (op_code, cell1, cell2) = {
        let c = &mut sheet.data[row][col];
        (c.op_code, c.cell1.clone(), c.cell2.clone())
    }; 
    if op_code == 'X' {
        return;
    }

    // ── 2) Compute the new value, error flag, and any sleep delta using *only* immutable borrows ──
    let (new_value, is_error,is_error2, delta) = {
        // Immutable borrow of the whole sheet
        let s = &*sheet;

        let mut err = false;
        let mut val = 0;
        let mut d = 0;
        let mut err1: bool = false;

        match op_code {
            '=' => {
                let ref_cell = &s.data[cell1.row as usize][cell1.col as usize];
                if ref_cell.is_error {
                    err = true;
                } else {
                    val = ref_cell.value;
                }
            }
            '+' | '-' | '*' | '/' => {
                let a = &s.data[cell1.row as usize][cell1.col as usize];
                let b = &s.data[cell2.row as usize][cell2.col as usize];
                if a.is_error || b.is_error || a.string.is_some() || b.string.is_some() {
                    err = true;
                } else {
                    (val, err1) = compute_cell(op_code, a.value, b.value, &mut String::new());
                }
            }
            'p' | 's' | 'u' | 'd' | 'b' => {
                let a = &s.data[cell1.row as usize][cell1.col as usize];
                if a.is_error || a.string.is_some() {
                    err = true;
                } else {
                    (val, err1) = compute_cell(op_code, a.value, cell2.row << 16 | cell2.col, &mut String::new());
                }
            }
            'Z' => {
                let a = &s.data[cell1.row as usize][cell1.col as usize];
                if a.is_error || a.string.is_some() {
                    err = true;
                } else {
                    val = a.value;
                    d = val;
                }
            }
            'S' | 'm' | 'M' | 'A' | 'D' => {
                // range check
                for i in cell1.row..=cell2.row {
                    for j in cell1.col..=cell2.col {
                        if s.data[i as usize][j as usize].is_error || s.data[i as usize][j as usize].string.is_some() {
                            err = true;
                            break;
                        }
                    }
                    if err { break; }
                }
                if !err {
                    val = compute_range_func(
                        s, op_code,
                        cell1.row, cell1.col,
                        cell2.row, cell2.col,
                        &mut String::new()
                    );
                }
            }
            _ => {}
        }
        (val,err, err1, d)
    };

    // ── 3) Finally, write back with one fresh mutable borrow ──
    {
        let c = &mut sheet.data[row][col];
        c.is_error = is_error;
        if is_error2 == true {
            c.is_error = true;
        }
        else {
        c.value    = new_value;}
    }
    *sleep_timer += delta;
}



pub fn topological_sort(avl_tree: &mut std::collections::HashMap<i32, i32>, sheet: &Sheet) -> Vec<i32> {
    let mut queue = std::collections::VecDeque::new();
    let mut result = Vec::new();

    // Find nodes with indegree 0
    for (&key, &value) in avl_tree.iter() {
        if value == 0 {
            queue.push_back(key);
        }
    }

    while let Some(node) = queue.pop_front() {
        result.push(node);

        for &dep in &sheet.data[(node % 1000) as usize][(node / 1000) as usize].dependencies {
            if let Some(indegree) = avl_tree.get_mut(&dep) {
                *indegree -= 1;
                if *indegree == 0 {
                    queue.push_back(dep);
                }
            }
        }
    }

    result
} 

pub fn remove_dependency(cell: &CellInfo, sheet: &mut Sheet) {
    let row = cell.row as usize;
    let col = cell.col as usize;
 
    let (op_code, cell1, cell2) = {
        let curr_cell = &sheet.data[row][col];
        (curr_cell.op_code, curr_cell.cell1.clone(), curr_cell.cell2.clone())
    };

    match op_code {
        'X' => {}
        '=' | 'p' | 's' | 'u' | 'd' | 'b' | 'Z' => {
            sheet.data[cell1.row as usize][cell1.col as usize]
                .dependencies
                .remove(&(col as i32 * 1000 + row as i32));
        }
        'S' | 'm' | 'M' | 'A' | 'D' => {
            for i in cell1.row..=cell2.row {
                for j in cell1.col..=cell2.col {
                    sheet.data[i as usize][j as usize]
                        .dependencies
                        .remove(&(col as i32 * 1000 + row as i32));
                }
            }
        }
        '+' | '-' | '*' | '/' => {
            sheet.data[cell1.row as usize][cell1.col as usize]
                .dependencies
                .remove(&(col as i32 * 1000 + row as i32));
            sheet.data[cell2.row as usize][cell2.col as usize]
                .dependencies
                .remove(&(col as i32 * 1000 + row as i32));
        }
        _ => {}
    } 

    let curr_cell = &mut sheet.data[row][col];
    curr_cell.cell1 = CellInfo { row: -1, col: -1 };
    curr_cell.cell2 = CellInfo { row: -1, col: -1 };
}


 
pub fn add_to_tree(mut avl_tree: &mut std::collections::HashMap<i32, i32>, cell: CellInfo, sheet: &Sheet) {
    for &curr in &sheet.data[cell.row as usize][cell.col as usize].dependencies {
        if !avl_tree.contains_key(&curr) {
            avl_tree.insert(curr, 1);
            let temp = CellInfo { row: curr % 1000, col: curr / 1000 };
            add_to_tree(&mut avl_tree, temp, sheet);
        } else {
            *avl_tree.entry(curr).or_insert(1) += 1;
        }
    }
}

pub fn add_constraints(curr_cell: CellInfo, cell1: CellInfo, cell2: CellInfo, op_code: char, sheet: &mut Sheet, status: &mut String, sleep_timer: &mut i32) {
    let curr_cell_row_col = curr_cell.col * 1000 + curr_cell.row;
    let mut ans = 0;
    let mut calc_error = false; 
    let mut calc_error1 = false; 
    // Create a HashMap named avl_tree where the key is curr_cell_row_col and the value is indegree (0 by default)
    let mut avl_tree: std::collections::HashMap<i32, i32> = std::collections::HashMap::new();
    avl_tree.insert(curr_cell_row_col, 0);
    let temp = CellInfo{ row: -1, col: -1};
    add_to_tree(&mut avl_tree, curr_cell.clone(), sheet);   
    match op_code {
        'X' => {
            ans = sheet.data[curr_cell.row as usize][curr_cell.col as usize].value;
            remove_dependency(&curr_cell, sheet);
            let cell = &mut sheet.data[curr_cell.row as usize][curr_cell.col as usize];
            cell.op_code = op_code;
            cell.cell1 = CellInfo { row: -1, col: -1 };
            cell.cell2 = CellInfo { row: -1, col: -1 };
        },
        '=' => {
            if check_cycle(&avl_tree, &cell1, &temp) {
                *status = String::from("circular error"); 
                return;
            }
            remove_dependency(&curr_cell, sheet);
            let cell = &mut sheet.data[curr_cell.row as usize][curr_cell.col as usize];
            cell.cell1 = cell1.clone();
            cell.cell2 = CellInfo { row: -1, col: -1 };
            cell.op_code = op_code;
            
            if sheet.data[cell1.row as usize][cell1.col as usize].is_error {
                calc_error = true;
            } else {
                ans = sheet.data[cell1.row as usize][cell1.col as usize].value;
            }
            sheet.data[cell1.row as usize][cell1.col as usize].dependencies.insert(curr_cell_row_col);
        },
        'p' | 's' | 'u' | 'd' | 'b' | 'Z' => {
            if check_cycle(&avl_tree, &cell1, &temp) {
                *status = String::from("circular error"); 
                return;
            }
            remove_dependency(&curr_cell, sheet);
            let cell = &mut sheet.data[curr_cell.row as usize][curr_cell.col as usize];
            cell.cell1 = cell1.clone();
            cell.cell2 = cell2.clone();
            cell.op_code = op_code; 
            let value = (cell2.row as i32) << 16 | (cell2.col as i32 & 0xFFFF); 
            if sheet.data[cell1.row as usize][cell1.col as usize].is_error {
                calc_error = true;
            } else {
                let (a,b) = compute_cell(op_code, 
                                 sheet.data[cell1.row as usize][cell1.col as usize].value, 
                                 value,
                                 status);
                ans = a;
                calc_error1 = b;
            } 
            sheet.data[cell1.row as usize][cell1.col as usize].dependencies.insert(curr_cell_row_col);
        },
        'S' | 'm' | 'M' | 'A' | 'D' => {
            if check_cycle_range_funcs(&avl_tree, &cell1, &cell2) {
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
            
            remove_dependency(&curr_cell, sheet);
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
            
            for i in cell1.row..=cell2.row {
                for j in cell1.col..=cell2.col {
                    sheet.data[i as usize][j as usize].dependencies.insert(curr_cell_row_col);
                }
            }
        },
        '+' | '-' | '*' | '/' => {
            if check_cycle(&avl_tree, &cell1, &cell2) {
                *status = String::from("circular error"); 
                return;
            }
            remove_dependency(&curr_cell, sheet);
            let cell = &mut sheet.data[curr_cell.row as usize][curr_cell.col as usize];
            cell.cell1 = cell1.clone();
            cell.cell2 = cell2.clone();
            cell.op_code = op_code;
            
            if sheet.data[cell1.row as usize][cell1.col as usize].is_error ||
               sheet.data[cell2.row as usize][cell2.col as usize].is_error {
                calc_error = true;
            } else {
                let (a,b) = compute_cell(op_code,
                                 sheet.data[cell1.row as usize][cell1.col as usize].value,
                                 sheet.data[cell2.row as usize][cell2.col as usize].value,
                                 status);
                ans = a;
                calc_error1 = b;
            }
            sheet.data[cell1.row as usize][cell1.col as usize].dependencies.insert(curr_cell_row_col);
            sheet.data[cell2.row as usize][cell2.col as usize].dependencies.insert(curr_cell_row_col);
        },
        _ => {
            *status = String::from("Invalid operation");
            return;
        }
    } 
    let cell = &mut sheet.data[curr_cell.row as usize][curr_cell.col as usize];
    if calc_error == true || calc_error1 == true {
        cell.is_error = true;}
    else {
        cell.value  = ans;
    }
    if cell.op_code == 'Z' {
        *sleep_timer += ans;
    }
    let sorted = topological_sort(&mut avl_tree, &sheet);  
    *status = String::from("ok");

    for i in sorted.into_iter(){
        let row = i % 1000;
        let col = i / 1000;
        recalculate(sheet, row as usize, col as usize, sleep_timer);
    }  
} 
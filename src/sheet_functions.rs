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
pub fn insert_into_list(sheet: &mut Sheet, from: &CellInfo, to: &CellInfo) {
    let new_node = DependencyNode {
        data: from.col * 1000 + from.row,
        next: sheet.data[to.row as usize][to.col as usize].dependencies.take(),
    };
    sheet.data[to.row as usize][to.col as usize].dependencies = Some(Box::new(new_node));
}

pub fn remove_dependency(sheet: &mut Sheet, cell: &CellInfo) {
    sheet.data[cell.row as usize][cell.col as usize].dependencies = None;
}
pub fn has_cycle(sheet: &Sheet, start: &CellInfo, target: &CellInfo, visited: &mut HashSet<i32>) -> bool {
    let key = start.col * 1000 + start.row;
    if visited.contains(&key) {
        return false;
    }
    visited.insert(key);

    if let Some(mut node) = &sheet.data[start.row as usize][start.col as usize].dependencies {
        while let Some(n) = node.as_ref() {
            let row = n.data % 1000;
            let col = n.data / 1000;
            if row == target.row && col == target.col {
                return true;
            }
            if has_cycle(sheet, CellInfo { row, col }, target, visited) {
                return true;
            }
            node = &n.next;
        }
    }
    false
}

pub fn topological(sheet: &mut Sheet, cell_info: &CellInfo, sleep_timer: &mut i32) {
    let mut indegree = HashMap::new();
    let mut graph = HashMap::new();
    let mut visited_cells = HashSet::new();
    let mut queue = VecDeque::new();

    // BFS to build dependency graph
    queue.push_back(cell_info);
    visited_cells.insert(cell_info.col * 1000 + cell_info.row);

    while let Some(cell_info) = queue.pop_front() {
        let key = cell_info.col * 1000 + cell_info.row;
        let cell = &sheet.data[cell_info.row as usize][cell_info.col as usize];
        let mut node = &cell.dependencies;

        while let Some(dep_node) = node {
            let dep_row = dep_node.data % 1000;
            let dep_col = dep_node.data / 1000;
            let dep_key = dep_col * 1000 + dep_row;
            *indegree.entry(key).or_insert(0) += 1;
            graph.entry(dep_key).or_insert(Vec::new()).push(cell_info.clone());

            let next_info = CellInfo { row: dep_row, col: dep_col };
            if !visited_cells.contains(&dep_key) {
                visited_cells.insert(dep_key);
                queue.push_back(next_info);
            }

            node = &dep_node.next;
        }
    }

    let mut q = VecDeque::new();

    for &k in visited_cells.iter() {
        if indegree.get(&k).unwrap_or(&0) == &0 {
            let row = k % 1000;
            let col = k / 1000;
            q.push_back(CellInfo { row, col });
        }
    }

    while let Some(ci) = q.pop_front() {
        let key = ci.col * 1000 + ci.row;
        let cell = &mut sheet.data[ci.row as usize][ci.col as usize];
        recalculate(cell, sheet, sleep_timer);

        if let Some(children) = graph.get(&key) {
            for child in children {
                let child_key = child.col * 1000 + child.row;
                let count = indegree.entry(child_key).or_insert(0);
                *count -= 1;
                if *count == 0 {
                    q.push_back(child.clone());
                }
            }
        }
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
  


// pub fn add_constraints(curr_cell: CellInfo, cell1: CellInfo, cell2: CellInfo, op_code: char, sheet: &mut Sheet, status: &mut String, sleep_timer: &mut i32) {
//     let curr_cell_row_col = curr_cell.col * 1000 + curr_cell.row;
//     let mut ans = 0;
//     let mut calc_error = false;
//     let mut dependencies = HashSet::new();
//     dependencies.insert(curr_cell_row_col);

//     match op_code {
//         'X' => {
//             ans = sheet.data[curr_cell.row as usize][curr_cell.col as usize].value;
//             // TODO: Implement remove_dependency
//             let curr_cell = curr_cell.clone();
//             remove_dependency(sheet, curr_cell);
//             let cell = &mut sheet.data[curr_cell.row as usize][curr_cell.col as usize];
//             cell.op_code = op_code;
//             cell.cell1 = CellInfo { row: -1, col: -1 };
//             cell.cell2 = CellInfo { row: -1, col: -1 };
//         },
//         '=' => {
//             // TODO: Implement cycle checking
//             let curr_cell = curr_cell.clone();
//             let mut visited = HashSet::new();
//             if has_cycle(sheet, cell1.clone(), curr_cell, &mut visited) {
//                 *status = String::from("circular error");
//                 return;
//             }

//             // TODO: Implement remove_dependency
//             remove_dependency(sheet, curr_cell);
//             let cell = &mut sheet.data[curr_cell.row as usize][curr_cell.col as usize];
//             cell.cell1 = cell1.clone();
//             cell.cell2 = CellInfo { row: -1, col: -1 };
//             cell.op_code = op_code;
            
//             if sheet.data[cell1.row as usize][cell1.col as usize].is_error {
//                 calc_error = true;
//             } else {
//                 ans = sheet.data[cell1.row as usize][cell1.col as usize].value;
//             }
//             // TODO: Implement insert_into_list
//             insert_into_list(sheet, curr_cell, cell1);
//         },
//         'p' | 's' | 'u' | 'd' | 'b' | 'Z' => {
//             // TODO: Implement cycle checking
//             let curr_cell = curr_cell.clone();
//             let mut visited = HashSet::new();
//             if has_cycle(sheet, cell1.clone(), curr_cell, &mut visited) {
//                 *status = String::from("circular error");
//                 return;
//             }
//             // TODO: Implement remove_dependency
//             remove_dependency(sheet, curr_cell);
//             let cell = &mut sheet.data[curr_cell.row as usize][curr_cell.col as usize];
//             cell.cell1 = cell1.clone();
//             cell.cell2 = cell2.clone();
//             cell.op_code = op_code;
            
//             let value = (cell2.row as i32) << 16 | (cell2.col as i32 & 0xFFFF);
//             if sheet.data[cell1.row as usize][cell1.col as usize].is_error {
//                 calc_error = true;
//             } else {
//                 ans = compute_cell(op_code, 
//                                  sheet.data[cell1.row as usize][cell1.col as usize].value, 
//                                  value,
//                                  status);
//             }
//             // TODO: Implement insert_into_list
//             insert_into_list(sheet, curr_cell, cell1);
//             insert_into_list(sheet, curr_cell, cell2);
//         },
//         'S' | 'm' | 'M' | 'A' | 'D' => {
//             // TODO: Implement cycle checking for range functions
//             let curr_cell = curr_cell.clone();
//             let mut visited = HashSet::new();
//             if has_cycle(sheet, cell1.clone(), curr_cell, &mut visited) {
//                 *status = String::from("circular error");
//                 return;
//             }
            
//             for i in cell1.row..=cell2.row {
//                 for j in cell1.col..=cell2.col {
//                     if sheet.data[i as usize][j as usize].is_error {
//                         calc_error = true;
//                         break;
//                     }
//                 }
//                 if calc_error {
//                     break;
//                 }
//             }
            
//             // TODO: Implement remove_dependency
//             remove_dependency(sheet, curr_cell);
//             let cell = &mut sheet.data[curr_cell.row as usize][curr_cell.col as usize];
//             cell.cell1 = cell1.clone();
//             cell.cell2 = cell2.clone();
//             cell.op_code = op_code;
            
//             if !calc_error {
//                 ans = compute_range_func(sheet,
//                                        op_code,
//                                        cell1.row as i32,
//                                        cell1.col as i32,
//                                        cell2.row as i32,
//                                        cell2.col as i32,
//                                        status);
//             }
            
//             // TODO: Implement insert_into_list for range
//             insert_into_list(sheet, curr_cell, cell1);
//             insert_into_list(sheet, curr_cell, cell2);
//         },
//         '+' | '-' | '*' | '/' => {
//             let curr_cell = curr_cell.clone();
//             // TODO: Implement cycle checking
//             let mut visited = HashSet::new();
//             if has_cycle(sheet, cell1.clone(), curr_cell, &mut visited) {
//                 *status = String::from("circular error");
//                 return;
//             }
//             // TODO: Implement remove_dependency
//             remove_dependency(sheet, curr_cell);
//             let cell = &mut sheet.data[curr_cell.row as usize][curr_cell.col as usize];
//             cell.cell1 = cell1.clone();
//             cell.cell2 = cell2.clone();
//             cell.op_code = op_code;
            
//             if sheet.data[cell1.row as usize][cell1.col as usize].is_error ||
//                sheet.data[cell2.row as usize][cell2.col as usize].is_error {
//                 calc_error = true;
//             } else {
//                 ans = compute_cell(op_code,
//                                  sheet.data[cell1.row as usize][cell1.col as usize].value,
//                                  sheet.data[cell2.row as usize][cell2.col as usize].value,
//                                  status);
//             }
//             // TODO: Implement insert_into_list for both cells
//             insert_into_list(sheet, curr_cell, cell1);
//             insert_into_list(sheet, curr_cell, cell2);
//         },
//         _ => {
//             *status = String::from("Invalid operation");
//             return;
//         }
//     }

//     let cell = &mut sheet.data[curr_cell.row as usize][curr_cell.col as usize];
//     cell.value = ans;
//     cell.is_error = calc_error;
    
//     if cell.op_code == 'Z' {
//         *sleep_timer += ans;
//     }
//     // TODO: Implement topological 
//     topological(sheet, curr_cell, sleep_timer);
// }

  
pub fn add_constraints(
    curr_cell: CellInfo,
    cell1: CellInfo,
    cell2: CellInfo,
    op_code: char,
    sheet: &mut Sheet,
    status: &mut String,
    sleep_timer: &mut i32,
) {
    let curr_cell_key = curr_cell.col * 1000 + curr_cell.row;
    let mut ans = 0;
    let mut calc_error = false;
    let mut dependencies = HashSet::new();
    dependencies.insert(curr_cell_key);

    match op_code {
        'X' => {
            remove_dependency(sheet, &curr_cell);
            let cell = &mut sheet.data[curr_cell.row as usize][curr_cell.col as usize];
            cell.op_code = op_code;
            cell.cell1 = CellInfo { row: -1, col: -1 };
            cell.cell2 = CellInfo { row: -1, col: -1 };
            // No recalculation needed
        }
        '=' => {
            let mut visited = HashSet::new();
            if has_cycle(sheet, &cell1, &curr_cell, &mut visited) {
                *status = String::from("circular error");
                return;
            }

            remove_dependency(sheet, &curr_cell);
            let cell = &mut sheet.data[curr_cell.row as usize][curr_cell.col as usize];
            cell.cell1 = cell1;
            cell.cell2 = CellInfo { row: -1, col: -1 };
            cell.op_code = op_code;

            if sheet.data[cell.cell1.row as usize][cell.cell1.col as usize].is_error {
                calc_error = true;
            } else {
                ans = sheet.data[cell.cell1.row as usize][cell.cell1.col as usize].value;
            }

            insert_into_list(sheet, &curr_cell, &cell.cell1);
        }
        'p' | 's' | 'u' | 'd' | 'b' | 'Z' => {
            let mut visited = HashSet::new();
            if has_cycle(sheet, &cell1, &curr_cell, &mut visited) {
                *status = String::from("circular error");
                return;
            }

            remove_dependency(sheet, &curr_cell);
            let cell = &mut sheet.data[curr_cell.row as usize][curr_cell.col as usize];
            cell.cell1 = cell1;
            cell.cell2 = cell2;
            cell.op_code = op_code;

            let value = (cell.cell2.row as i32) << 16 | (cell.cell2.col as i32 & 0xFFFF);

            if sheet.data[cell.cell1.row as usize][cell.cell1.col as usize].is_error {
                calc_error = true;
            } else {
                ans = compute_cell(
                    op_code,
                    sheet.data[cell.cell1.row as usize][cell.cell1.col as usize].value,
                    value,
                    status,
                );
            }

            insert_into_list(sheet, &curr_cell, &cell.cell1);
            insert_into_list(sheet, &curr_cell, &cell.cell2);
        }
        'S' | 'm' | 'M' | 'A' | 'D' => {
            let mut visited = HashSet::new();
            if has_cycle(sheet, &cell1, &curr_cell, &mut visited) {
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

            remove_dependency(sheet, &curr_cell);
            let cell = &mut sheet.data[curr_cell.row as usize][curr_cell.col as usize];
            cell.cell1 = cell1;
            cell.cell2 = cell2;
            cell.op_code = op_code;

            if !calc_error {
                ans = compute_range_func(
                    sheet,
                    op_code,
                    cell1.row,
                    cell1.col,
                    cell2.row,
                    cell2.col,
                    status,
                );
            }

            insert_into_list(sheet, &curr_cell, &cell1);
            insert_into_list(sheet, &curr_cell, &cell2);
        }
        '+' | '-' | '*' | '/' => {
            let mut visited = HashSet::new();
            if has_cycle(sheet, &cell1, &curr_cell, &mut visited) {
                *status = String::from("circular error");
                return;
            }

            remove_dependency(sheet, &curr_cell);
            let cell = &mut sheet.data[curr_cell.row as usize][curr_cell.col as usize];
            cell.cell1 = cell1;
            cell.cell2 = cell2;
            cell.op_code = op_code;

            if sheet.data[cell.cell1.row as usize][cell.cell1.col as usize].is_error
                || sheet.data[cell.cell2.row as usize][cell.cell2.col as usize].is_error
            {
                calc_error = true;
            } else {
                ans = compute_cell(
                    op_code,
                    sheet.data[cell.cell1.row as usize][cell.cell1.col as usize].value,
                    sheet.data[cell.cell2.row as usize][cell.cell2.col as usize].value,
                    status,
                );
            }

            insert_into_list(sheet, &curr_cell, &cell.cell1);
            insert_into_list(sheet, &curr_cell, &cell.cell2);
        }
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

    topological(sheet, &curr_cell, sleep_timer);
}


pub struct CellInfo {
    pub row: i16,
    pub col: i16,
}
 
pub struct DependencyNode {
    pub data: i32,
    pub next: Option<Box<DependencyNode>>,
}
 
pub struct Cell {
    pub value: i32,
    pub is_error: bool,
    pub op_code: char,
    pub cell1: CellInfo,
    pub cell2: CellInfo,
    pub dependencies: Option<Box<DependencyNode>>,
}
 
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
  
pub fn print_sheet(start_row: i32, start_col: i32, total_rows: i32, total_cols: i32) {
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
            print!("{:>space$}", 0);
        }
        println!("");
    }
}





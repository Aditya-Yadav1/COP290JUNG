// use std::f64::consts::E;
use crate::sheet_functions::Sheet;

pub fn sum(sheet: &Sheet, val_row1: i32, c1: i32, val_row2: i32, c2: i32) -> i32 {
    let mut a = 0;
    for i in val_row1..=val_row2 {
        for j in c1..=c2 {
            a += sheet.data[i as usize][j as usize].value;
        }
    }
    a
}

pub fn min(sheet: &Sheet, val_row1: i32, c1: i32, val_row2: i32, c2: i32) -> i32 {
    let mut a = i32::MAX;
    for i in val_row1..=val_row2 {
        for j in c1..=c2 {
            let b = sheet.data[i as usize][j as usize].value;
            if b < a {
                a = b;
            }
        }
    }
    a
}

pub fn max(sheet: &Sheet, val_row1: i32, c1: i32, val_row2: i32, c2: i32) -> i32 {
    let mut a = i32::MIN;
    for i in val_row1..=val_row2 {
        for j in c1..=c2 {
            let b = sheet.data[i as usize][j as usize].value;
            if b > a {
                a = b;
            }
        }
    }
    a
}

pub fn avg(sheet: &Sheet, val_row1: i32, c1: i32, val_row2: i32, c2: i32) -> i32 {
    let total = sum(sheet, val_row1, c1, val_row2, c2);
    let count = (val_row2 - val_row1 + 1) * (c2 - c1 + 1);
    if count > 0 {
        total / count
    } else {
        -1
    }
}

pub fn stdev(sheet: &Sheet, val_row1: i32, c1: i32, val_row2: i32, c2: i32) -> i32 {
    let avg_val = avg(sheet, val_row1, c1, val_row2, c2);
    let mut total = 0;
    for i in val_row1..=val_row2 {
        for j in c1..=c2 {
            let diff = sheet.data[i as usize][j as usize].value - avg_val;
            total += diff * diff;
        }
    }
    let std = (total as f64 / ((val_row2 - val_row1 + 1) * (c2 - c1 + 1)) as f64).sqrt();
    std.round() as i32
}

pub fn compute_cell(op_code: char, cell_value: i32, cell_value2: i32, status: &mut String) -> (i32, bool){

    
    match op_code {
        '+' | 'p' => (cell_value + cell_value2,false),
        '-' | 's' => (cell_value - cell_value2, false),
        '*' | 'u' => (cell_value * cell_value2, false),
        '/' | 'd' => {
            if cell_value2 == 0 {

                (-1, true)
            } else {
                (cell_value / cell_value2,false)
            }
        }
        'b' => {
            if cell_value == 0 {

                (-1, true)
            } else {
                (cell_value2 / cell_value, false)
            }
        }
        'Z' => (cell_value, false),
        _ => {
            status.push_str("err");
            (-1, true)
        }
    }
}

pub fn compute_range_func(sheet: &Sheet, op_code: char, row1: i32, col1: i32, row2: i32, col2: i32, status: &mut String) -> i32 {
    if col1 > col2 || row1 > row2 {
        status.push_str("err");
        return -1;
    } 
    match op_code {
        'S' => sum(sheet, row1, col1, row2, col2),
        'm' => min(sheet, row1, col1, row2, col2),
        'M' => max(sheet, row1, col1, row2, col2),
        'A' => avg(sheet, row1, col1, row2, col2),
        'D' => stdev(sheet, row1, col1, row2, col2),
        _ => -1
    }

}
// use std::f64::consts::E;
use crate::sheet_functions::Sheet;
use crate::sheet_functions::OpCode;
use crate::sheet_functions::OpCode::*;
use std::string::String;

pub fn sum(sheet: &Sheet, val_row1: i16, c1: i16, val_row2: i16, c2: i16) -> i32 {
    let mut a = 0;
    for i in val_row1..=val_row2 {
        for j in c1..=c2 {
            a += sheet.data[i as usize][j as usize].value;
        }
    }
    a
}

pub fn min(sheet: &Sheet, val_row1: i16, c1: i16, val_row2: i16, c2: i16) -> i32 {
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

pub fn max(sheet: &Sheet, val_row1: i16, c1: i16, val_row2: i16, c2: i16) -> i32 {
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

pub fn avg(sheet: &Sheet, val_row1: i16, c1: i16, val_row2: i16, c2: i16) -> i32 {
    let total = sum(sheet, val_row1, c1, val_row2, c2);
    let row2 = val_row2 as i32;
    let col2 = c2 as i32;
    let row1 = val_row1 as i32;
    let col1 = c1 as i32;

    let count = ((row2 - row1 + 1)  * (col2 - col1 + 1)) as i32;
    if count > 0 {
        total / count
    } else {
        -1
    }
}

pub fn stdev(sheet: &Sheet, val_row1: i16, c1: i16, val_row2: i16, c2: i16) -> i32 {
    let avg_val = avg(sheet, val_row1, c1, val_row2, c2);
    let mut total = 0;
    for i in val_row1..=val_row2 {
        for j in c1..=c2 {
            let diff = sheet.data[i as usize][j as usize].value - avg_val;
            total += diff * diff;
        }
    }
    let row1 = val_row1 as i32;
    let row2 = val_row2 as i32;
    let col1 = c1 as i32;
    let col2 = c2 as i32;
    let count = ((row2 - row1 + 1)  * (col2 - col1 + 1)) as i32;
    let std = (total as f64 / count as f64).sqrt();
    std.round() as i32
}

pub fn compute_cell(op_code: OpCode, cell_value: i32, cell_value2: i32, status: &mut String) -> (i32, bool){

    
    match op_code {
        CellPlusCell | CellPlusConstant => (cell_value + cell_value2,false),
        CellMinusCell | CellMinusConstant => (cell_value - cell_value2, false),
        CellTimesCell | CellTimesConstant => (cell_value * cell_value2, false),
        CellDivideCell | CellDivideConstant => {
            if cell_value2 == 0 {

                (-1, true)
            } else {
                (cell_value / cell_value2,false)
            }
        }
        ConstantDividesCell => {
            if cell_value == 0 {

                (-1, true)
            } else {
                (cell_value2 / cell_value, false)
            }
        }
        String => (cell_value, false),
        _ => {
            status.push_str("err");
            (-1, true)
        }
    }
}

pub fn compute_range_func(sheet: &Sheet, op_code: OpCode, row1: i16, col1: i16, row2: i16, col2: i16, status: &mut String) -> i32 {
    if col1 > col2 || row1 > row2 {
        status.push_str("err");
        return -1;
    } 
    match op_code {
        Sum => sum(sheet, row1, col1, row2, col2),
        Min => min(sheet, row1, col1, row2, col2),
        Max => max(sheet, row1, col1, row2, col2),
        Avg => avg(sheet, row1, col1, row2, col2),
        Stdev => stdev(sheet, row1, col1, row2, col2),
        _ => -1
    }

}
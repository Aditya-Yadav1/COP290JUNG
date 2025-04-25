// use std::f64::consts::E;
use crate::sheet_functions::Sheet;
use crate::sheet_functions::OpCode;
use crate::sheet_functions::OpCode::*;
use core::hash;
use std::string::String;
use std::u8::MIN;

pub fn sum(sheet: &Sheet, val_row1: i16, c1: i16, val_row2: i16, c2: i16) -> i32 {
    let mut a = 0;
    let hash_map_size = sheet.data.len();
    let range_size = ((val_row2 - val_row1 + 1) as usize * (c2 - c1 + 1) as usize) as usize;
    if hash_map_size > range_size{
        for i in val_row1..=val_row2 {
            for j in c1..=c2 {
                let value = sheet
                    .data
                    .get(&(i as i16, j as i16))
                    .map_or(0, |cell| cell.value); // Use 0 if the cell is not in the map
                a += value;
            }
        }
    }
    else{
        for (row,col) in sheet.data.keys(){
            if row >= &val_row1 && row <= &val_row2 && col >= &c1 && col <= &c2{
                let value = sheet
                    .data
                    .get(&(*row, *col))
                    .map_or(0, |cell| cell.value); 
                a += value;
            }
        }
    }
    a
}

pub fn min(sheet: &Sheet, val_row1: i16, c1: i16, val_row2: i16, c2: i16) -> i32 {
    let mut a = i32::MAX;
    let hash_map_size = sheet.data.len();
    let range_size = ((val_row2 - val_row1 + 1) as usize * (c2 - c1 + 1) as usize) as usize;
    if hash_map_size >= range_size{
    for i in val_row1..=val_row2 {
        for j in c1..=c2 {
            let value = sheet
                .data
                .get(&(i as i16, j as i16))
                .map_or(0, |cell| cell.value); // Use 0 if the cell is not in the map
            if value < a {
                a = value;
            }
        }
    }}
    else{
        let mut count=0;
        for (row,col) in sheet.data.keys(){
            if row >= &val_row1 && row <= &val_row2 && col >= &c1 && col <= &c2{
                let value = sheet.data.get(&(*row, *col)).map_or(0, |cell| cell.value); 
                if value < a { a = value;}count = count +1  ;
            }}
        if count != range_size{ a = if a > 0 {0} else {a};}}
    a
}

pub fn max(sheet: &Sheet, val_row1: i16, c1: i16, val_row2: i16, c2: i16) -> i32 {
    let mut a = i32::MIN;
    let hash_map_size = sheet.data.len();
    let range_size = ((val_row2 - val_row1 + 1) as usize * (c2 - c1 + 1) as usize) as usize;
    if hash_map_size >= range_size{
    for i in val_row1..=val_row2 {
        for j in c1..=c2 {
            let value = sheet
                .data
                .get(&(i as i16, j as i16))
                .map_or(0, |cell| cell.value); // Use 0 if the cell is not in the map
            if value > a {
                a = value;
            }
        }
    }}
    else{
        let mut count=0;
        for (row,col) in sheet.data.keys(){
            if row >= &val_row1 && row <= &val_row2 && col >= &c1 && col <= &c2{
                let value = sheet.data.get(&(*row, *col)).map_or(0, |cell| cell.value); 
                if value > a { a = value;}count = count + 1;
            }}
        if count != range_size{ a = if a < 0 {0} else {a};} }
    a
}

pub fn avg(sheet: &Sheet, val_row1: i16, c1: i16, val_row2: i16, c2: i16) -> i32 {
    let total = sum(sheet, val_row1, c1, val_row2, c2);
    let row2 = val_row2 as i32;
    let col2 = c2 as i32;
    let row1 = val_row1 as i32;
    let col1 = c1 as i32;

    let count = ((row2 - row1 + 1) as i32 * (col2 - col1 + 1) as i32) as i32;
    if count > 0 {
        total / count
    } else {
        -1
    }
}
pub fn stdev(sheet: &Sheet, val_row1: i16, c1: i16, val_row2: i16, c2: i16) -> i32 {
    let avg_val = avg(sheet, val_row1, c1, val_row2, c2);
    let mut total : i64 = 0;
    let count = ((val_row2 - val_row1 + 1) as i32 * (c2 - c1 + 1)as i32) as i32;
    let hash_map_size = sheet.data.len();
    if hash_map_size > count as usize{
        for i in val_row1..=val_row2 {
            for j in c1..=c2 {
                let value = sheet
                    .data
                    .get(&(i as i16, j as i16))
                    .map_or(0, |cell| cell.value); // Use 0 if the cell is not in the map
                let diff : i64 =( value - avg_val) as i64;
                total += diff * diff;
            }
        }
    }
    else{
        for (row,col) in sheet.data.keys(){
            if row >= &val_row1 && row <= &val_row2 && col >= &c1 && col <= &c2{
                let value = sheet.data.get(&(*row, *col)).map_or(0, |cell| cell.value); 
                let diff : i64 =( value - avg_val) as i64;total += diff * diff;
            }}
    }
    let std = (total as f64 / count as f64).sqrt();
    std.round() as i32
}

pub fn compute_cell(op_code: OpCode, cell_value: i32, cell_value2: i32, status: &mut String) -> (i32, bool){
    match op_code {
        CellPlusCell | CellPlusConstant => (cell_value + cell_value2,false),
        CellMinusCell | CellMinusConstant => (cell_value - cell_value2, false),
        ConstantMinusCell => (cell_value2 - cell_value, false),
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
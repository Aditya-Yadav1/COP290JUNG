use crate::calculate_functions::compute_cell;
use crate::calculate_functions::compute_range_func;
use crate::calculate_functions;
use std::collections::{HashMap, HashSet, VecDeque};
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::ser::SerializeTuple;
use serde::de::{self, Visitor, SeqAccess};
use std::fmt;
use std::string::String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize  )]
pub enum OpCode {
    NoConstraint,
    CellEqualsCell,
    CellPlusCell,
    CellMinusCell,
    CellTimesCell,
    CellDivideCell,
    CellPlusConstant,
    CellMinusConstant,
    CellTimesConstant,
    CellDivideConstant,
    ConstantDividesCell,
    Sum,
    Min,
    Max,
    Avg,
    Stdev,
    Sleep,
    String,
}


#[derive(Clone,Debug)]
pub struct CellInfo {
    pub row: i16,
    pub col: i16,
}

#[derive(Clone,Debug)]
pub struct Cell {
    pub value: i32,
    pub is_error: bool,
    pub string: Option<String>,
    pub op_code: OpCode,
    pub cell1: CellInfo,
    pub cell2: CellInfo,
    pub dependencies: HashSet<i32>,  
}

#[derive(Clone)]
pub struct extendedCell{
    pub value: i32,
    pub string: Option<String>,
    pub is_error: bool,
    pub op_code: OpCode,
    pub cell1: CellInfo,
    pub cell2: CellInfo,
    pub dependencies: HashSet<i32>,  
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Sheet {
    pub rows: i32,
    pub cols: i32,
    pub data: Vec<Vec<Cell>>,
}

impl Serialize for CellInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tuple = serializer.serialize_tuple(2)?;
        tuple.serialize_element(&self.row)?;
        tuple.serialize_element(&self.col)?;
        tuple.end()
    }
}

impl<'de> Deserialize<'de> for CellInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CellInfoVisitor;

        impl<'de> Visitor<'de> for CellInfoVisitor {
            type Value = CellInfo;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a tuple [row, col]")
            }
            fn visit_seq<V>(self, mut seq: V) -> Result<CellInfo, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let row = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let col = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(CellInfo { row, col })
            }
        }
        deserializer.deserialize_tuple(2, CellInfoVisitor)
    }
}

impl Serialize for Cell {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tuple = serializer.serialize_tuple(7)?;
        tuple.serialize_element(&self.value)?;
        tuple.serialize_element(&self.string)?;
        tuple.serialize_element(&self.is_error)?;
        tuple.serialize_element(&self.op_code)?;
        tuple.serialize_element(&self.cell1)?;
        tuple.serialize_element(&self.cell2)?;
        tuple.serialize_element(&self.dependencies)?;
        tuple.end()
    }
}

impl<'de> Deserialize<'de> for Cell {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CellVisitor;

        impl<'de> Visitor<'de> for CellVisitor {
            type Value = Cell;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a compact Cell as a 7-element tuple")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Cell, V::Error>
            where
                V: SeqAccess<'de>,
            {
                Ok(Cell {
                    value: seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?,
                    string: seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?,
                    is_error: seq.next_element()?.ok_or_else(|| de::Error::invalid_length(2, &self))?,
                    op_code: seq.next_element()?.ok_or_else(|| de::Error::invalid_length(3, &self))?,
                    cell1: seq.next_element()?.ok_or_else(|| de::Error::invalid_length(4, &self))?,
                    cell2: seq.next_element()?.ok_or_else(|| de::Error::invalid_length(5, &self))?,
                    dependencies: seq.next_element()?.ok_or_else(|| de::Error::invalid_length(6, &self))?,
                })
            }
        }

        deserializer.deserialize_tuple(7, CellVisitor)
    }
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
                    string: None,
                    op_code: NoConstraint,
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

impl Cell {
    fn default() -> Self {
        Cell {
            value: 0,
            is_error: false,
            string: None,
            op_code: NoConstraint,
            cell1: CellInfo { row: -1, col: -1 },
            cell2: CellInfo { row: -1, col: -1 },
            dependencies: HashSet::new(),
        }
    }

    fn delete_cell_value(&mut self) {
        self.value = 0;
        self.string = None;
        self.is_error = false;
        self.op_code = NoConstraint;
        self.cell1 = CellInfo { row: -1, col: -1 };
        self.cell2 = CellInfo { row: -1, col: -1 };
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
    let key1 = cell1.col as i32 * 1000 + cell1.row as i32;
    if avl_tree.contains_key(&key1) {
        return true;
    }
    if cell2.col == -1 && cell2.row == -1 {
        return false;
    }
    let key2 = cell2.col as i32 * 1000 + cell2.row as i32;
    if avl_tree.contains_key(&key2) {
        return true;
    }
    false
}
pub fn check_cycle_range_funcs(avl_tree: &std::collections::HashMap<i32, i32>, cell1: &CellInfo, cell2: &CellInfo) -> bool {
    for (&key, _) in avl_tree.iter() {
        let col = key / 1000;
        let row = key % 1000;

        if col >= cell1.col as i32 && col <= cell2.col as i32 && row >= cell1.row as i32 && row <= cell2.row as i32 {
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
    if op_code == NoConstraint || op_code == String {
        return;
    }

    // ── 2) Compute the new value, error flag, and any sleep delta using *only* immutable borrows ──
    let (new_value, string_value, is_error, is_error2) = {
        // Immutable borrow of the whole sheet
        let s = &*sheet;

        let mut err = false;
        let mut val = 0;
        let mut strval = String::new();
        let mut err1: bool = false;

        match op_code {
            CellEqualsCell => {
                let ref_cell = &s.data[cell1.row as usize][cell1.col as usize];
                if ref_cell.is_error {
                    err = true;
                } else {
                    if ref_cell.string.is_some(){
                        strval = ref_cell.string.as_ref().unwrap().clone();
                    }else{
                        val = ref_cell.value; 
                    } 
                }
            }
            CellPlusCell | CellMinusCell | CellTimesCell | CellDivideCell => {
                let a = &s.data[cell1.row as usize][cell1.col as usize];
                let b = &s.data[cell2.row as usize][cell2.col as usize];
                if a.is_error || b.is_error || a.string.is_some() || b.string.is_some() {
                    err = true;
                } else {
                    (val, err1) = compute_cell(op_code, a.value, b.value, &mut String::new());
                }
            }
            CellPlusConstant | CellMinusConstant | CellTimesConstant | CellDivideConstant | ConstantDividesCell => {
                let a = &s.data[cell1.row as usize][cell1.col as usize];
                if a.is_error || a.string.is_some() {
                    err = true;
                } else {
                    (val, err1) = compute_cell(op_code, a.value, (cell2.row as i32) << 16 | cell2.col as i32, &mut String::new());
                }
            }
            Sleep => {
                let a = &s.data[cell1.row as usize][cell1.col as usize];
                if a.is_error || a.string.is_some() {
                    err = true;
                } else {
                    val = a.value;
                    *sleep_timer += val;
                }
            }
            Sum | Min | Max | Avg | Stdev => {
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
        (val,strval,err, err1)
    };

    // ── 3) Finally, write back with one fresh mutable borrow ──
    {
        let c = &mut sheet.data[row][col];
        c.is_error = is_error;
        if is_error2 == true {
            c.is_error = true;
        }
        else {
            if op_code == CellEqualsCell && string_value != "" {
                c.string = Some(string_value.clone());
            } else{
                c.value  = new_value;
                c.string = None;
            }
        }
    }
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

use OpCode::*;

pub fn remove_dependency(cell: &CellInfo, sheet: &mut Sheet) {
    let row = cell.row as usize;
    let col = cell.col as usize;
 
    let (op_code, cell1, cell2) = {
        let curr_cell = &sheet.data[row][col];
        (curr_cell.op_code, curr_cell.cell1.clone(), curr_cell.cell2.clone())
    };

    match op_code {
        NoConstraint => {}
        CellEqualsCell | CellPlusConstant | CellTimesConstant | CellMinusConstant | CellDivideConstant | ConstantDividesCell | Sleep => {
            sheet.data[cell1.row as usize][cell1.col as usize]
                .dependencies
                .remove(&(col as i32 * 1000 + row as i32));
        }
        Sum | Min | Max | Avg | Stdev => {
            for i in cell1.row..=cell2.row {
                for j in cell1.col..=cell2.col {
                    sheet.data[i as usize][j as usize]
                        .dependencies
                        .remove(&(col as i32 * 1000 + row as i32));
                }
            }
        }
        CellPlusCell | CellMinusCell | CellTimesCell | CellDivideCell => {
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
            let temp = CellInfo { row: (curr % 1000) as i16, col: (curr / 1000) as i16 };
            add_to_tree(&mut avl_tree, temp, sheet);
        } else {
            *avl_tree.entry(curr).or_insert(1) += 1;
        }
    }
}

pub fn add_constraints(curr_cell: CellInfo, cell1: CellInfo, cell2: CellInfo, op_code: OpCode, sheet: &mut Sheet, status: &mut String, sleep_timer: &mut i32) {
    let curr_cell_row_col = curr_cell.col as i32 * 1000 + curr_cell.row as i32;
    let mut ans = 0;
    let mut calc_error = false; 
    let mut calc_error1 = false; 
    // Create a HashMap named avl_tree where the key is curr_cell_row_col and the value is indegree (0 by default)
    let mut avl_tree: std::collections::HashMap<i32, i32> = std::collections::HashMap::new();
    avl_tree.insert(curr_cell_row_col, 0);
    let temp = CellInfo{ row: -1, col: -1};
    add_to_tree(&mut avl_tree, curr_cell.clone(), sheet);   
    match op_code {
        NoConstraint => {
            ans = sheet.data[curr_cell.row as usize][curr_cell.col as usize].value;
            remove_dependency(&curr_cell, sheet);
            let cell = &mut sheet.data[curr_cell.row as usize][curr_cell.col as usize];
            cell.op_code = op_code;
            cell.cell1 = CellInfo { row: -1, col: -1 };
            cell.cell2 = CellInfo { row: -1, col: -1 };
        },
        String => {
            // string case
            remove_dependency(&curr_cell, sheet); 
            let cell = &mut sheet.data[curr_cell.row as usize][curr_cell.col as usize];
            cell.op_code = op_code;
            cell.cell1 = CellInfo { row: -1, col: -1 };
            cell.cell2 = CellInfo { row: -1, col: -1 };
        },
        CellEqualsCell | Sleep => {
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
        CellPlusConstant | CellMinusConstant | CellTimesConstant | CellDivideConstant | ConstantDividesCell => {
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
        Sum | Min | Max | Avg | Stdev => {
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
                                       cell1.row as i16,
                                       cell1.col as i16,
                                       cell2.row as i16,
                                       cell2.col as i16,
                                       status);
            }
            
            for i in cell1.row..=cell2.row {
                for j in cell1.col..=cell2.col {
                    sheet.data[i as usize][j as usize].dependencies.insert(curr_cell_row_col);
                }
            }
        },
        CellPlusCell | CellMinusCell | CellTimesCell | CellDivideCell => {
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
            *status = String::from("err");
            return;
        }
    } 
    let cell = &mut sheet.data[curr_cell.row as usize][curr_cell.col as usize];
    cell.is_error = calc_error || calc_error1;
    if !cell.is_error && cell.op_code != String {
        cell.value  = ans;
        cell.string = None;
    }

    let sorted = topological_sort(&mut avl_tree, &sheet);  
    *status = String::from("ok");

    for i in sorted.into_iter(){
        let row = i % 1000;
        let col = i / 1000;
        if row == curr_cell.row as i32 && col == curr_cell.col as i32 {
            continue;
        }
        recalculate(sheet, row as usize, col as usize, sleep_timer);
    }  
} 


pub fn update_dependencies(old_cell: Cell,old_cell_row: i16, old_cell_col: i16, new_cell_row: i16, new_cell_col: i16, sheet: &mut Sheet) {
    //goes to dependency set of cells depending on old cell and removes the refrence to old cell and adds the refrence to new cell
    let mut avl_tree: std::collections::HashMap<i32, i32> = std::collections::HashMap::new();
    avl_tree.insert(new_cell_col as i32 * 1000 + new_cell_row as i32, 0);
    let temp = CellInfo{ row: -1, col: -1};
    add_to_tree(&mut avl_tree, CellInfo{ row: new_cell_row, col: new_cell_col }, sheet);

    let sorted = topological_sort(&mut avl_tree, sheet);

    for i in sorted.into_iter(){
        let row = i % 1000;
        let col = i / 1000;
        if row == new_cell_row as i32 && col == new_cell_col as i32 {
            continue;
        }
        else if sheet.data[row as usize][col as usize].op_code == Sum ||
           sheet.data[row as usize][col as usize].op_code == Min ||
           sheet.data[row as usize][col as usize].op_code == Max ||
           sheet.data[row as usize][col as usize].op_code == Avg ||
           sheet.data[row as usize][col as usize].op_code == Stdev {
           recalculate(sheet, row as usize, col as usize, &mut 0);
        }
        else{
            if sheet.data[row as usize][col as usize].cell1.row == old_cell_row &&
               sheet.data[row as usize][col as usize].cell1.col == old_cell_col {
                sheet.data[row as usize][col as usize].cell1 = CellInfo{ row: new_cell_row, col: new_cell_col };
            }
            if sheet.data[row as usize][col as usize].cell2.row == old_cell_row &&
               sheet.data[row as usize][col as usize].cell2.col == old_cell_col {
                sheet.data[row as usize][col as usize].cell2 = CellInfo{ row: new_cell_row, col: new_cell_col };
            }
        }
    }
}

pub fn change_dependecy_set(new_cell: &mut Cell, sheet: &mut Sheet , del_range_dependencies: bool) {
    //removes the range/ non range dependencies in the given dependency set
    for i in new_cell.dependencies.clone() {
        let row = i%1000;
        let col = i/1000;
        if (sheet.data[row as usize][col as usize].op_code == Sum ||
           sheet.data[row as usize][col as usize].op_code == Min ||
           sheet.data[row as usize][col as usize].op_code == Max ||
           sheet.data[row as usize][col as usize].op_code == Avg ||
           sheet.data[row as usize][col as usize].op_code == Stdev ) {
            if del_range_dependencies{
                new_cell.dependencies.remove(&(col as i32 * 1000 + row as i32));
            }
            else{
                recalculate_dependecy(CellInfo{ row: row as i16, col: col as i16 }, sheet);
            }
        }
        else if !del_range_dependencies{
            new_cell.dependencies.remove(&(col as i32 * 1000 + row as i32));
        }
    }
}


pub fn recalculate_dependecy(curr_cell: CellInfo, sheet: &mut Sheet) {
    let mut dependency_set = std::collections::HashMap::new();
    dependency_set.insert(curr_cell.col as i32 * 1000 + curr_cell.row as i32, 0);
    add_to_tree(&mut dependency_set, curr_cell.clone(), sheet);
    let sorted = topological_sort(&mut dependency_set, sheet);
    for i in sorted.into_iter(){
        let row = i % 1000;
        let col = i / 1000;
        if row == curr_cell.row as i32 && col == curr_cell.col as i32 {
            continue;
        }
        recalculate(sheet, row as usize, col as usize, &mut 0);
    }
}
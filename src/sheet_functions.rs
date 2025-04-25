use crate::calculate_functions::compute_cell;
use crate::calculate_functions::compute_range_func;
use std::collections::{HashSet,HashMap};
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::ser::SerializeTuple;
use serde::de::{self, Visitor, SeqAccess};
use std::fmt;
use std::string::String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
 

#[derive(Clone, Deserialize, Serialize)]
pub struct Sheet {
    pub rows: i32,
    pub cols: i32,
    pub buul: Vec<Vec<bool>>,
    pub data: HashMap<(i16, i16), Cell>,
    pub tuup: HashMap<((i16,i16),(i16,i16)),Vec<(i16,i16)>>,
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
        
pub fn get_or_create_cell(sheet: &mut Sheet, row: i32, col: i32) -> &mut Cell {
    sheet.data.entry((row as i16, col as i16)).or_insert_with(|| Cell {
        value: 0,
        is_error: false,
        string: None,
        op_code: OpCode::NoConstraint,
        cell1: CellInfo { row: -1, col: -1 },
        cell2: CellInfo { row: -1, col: -1 },
        dependencies: std::collections::HashSet::new(),
    })
}
// is_valid_cell
pub fn is_valid_cell(row: i32, col: i32, total_rows: i32, total_cols: i32) -> bool {
    row >= 0 && row < total_rows && col >= 0 && col < total_cols
}           


impl Sheet {
    pub fn new(rows: i32, cols: i32) -> Self {
        let mut buul = Vec::with_capacity(rows as usize);

        // Initialize the `buul` vector
        for _ in 0..rows {
            let mut row = Vec::with_capacity(cols as usize);
            for _ in 0..cols {
                row.push(false); // Initialize `buul` with `false`
            }
            buul.push(row);
        }

        // Initialize the `data` and `tuup` HashMaps as empty
        let data = HashMap::with_capacity(1000);
        let tuup = HashMap::with_capacity(1000);
        
        Sheet {
            rows,
            cols,
            buul,
            data,
            tuup,
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
    let max_col_display: i32 = start_col + std::cmp::min(total_cols - start_col, 10);
    let max_row_display: i32 = start_row + std::cmp::min(total_rows - start_row, 10);
    let space: usize = 10;

    // Print column headers
    print!("{:>space$}", "");
    for i in start_col..max_col_display {
        print!("{:>space$}", col_num_to_col_name(i));
    }
    println!();

    // Print rows
    for i in start_row..max_row_display {
        print!("{:>space$}", i + 1); // Print row header
        for j in start_col..max_col_display {
            if let Some(cell) = sheet.data.get(&(i as i16, j as i16)) {
                // If the cell exists in the map
                if !cell.is_error {
                    if let Some(s) = &cell.string {
                        print!("{:>space$}", s); // Print string value
                    } else {
                        print!("{:>space$}", cell.value); // Print numeric value
                    }
                } else {
                    print!("{:>space$}", "Err"); // Print error
                }
            } else {
                // If the cell does not exist in the map, print 0
                print!("{:>space$}", 0);
            }
        }
        println!();
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
    
    let (op_code, cell1, cell2) = {
        let c = sheet.data.get_mut(&(row as i16, col as i16)).expect("not in map");
        (c.op_code, c.cell1.clone(), c.cell2.clone())
    }; 
    if op_code == NoConstraint || op_code == String {
        return;
    }

    // ── 2) Compute the new value, error flag, and any sleep delta using *only* immutable borrows ──
    let (new_value, string_value, is_error, is_error2) = {
        // Immutable borrow of the whole sheet  
        let mut err = false;
        let mut val = 0;
        let mut strval = String::new();
        let mut err1: bool = false;

        match op_code {
            CellEqualsCell => {
                let ref_cell = get_or_create_cell(sheet, cell1.row as i32, cell1.col as i32);
                if ref_cell.is_error {
                    err = true;
                }else {
                    if ref_cell.string.is_some(){
                        strval = ref_cell.string.as_ref().unwrap().clone();
                    }else{ 
                        val = ref_cell.value; 
                    } 
                }
            }
            CellPlusCell | CellMinusCell | CellTimesCell | CellDivideCell => {
                let (a_value, a_is_error, a_string) = {
                    let a = get_or_create_cell(sheet, cell1.row as i32, cell1.col as i32);
                    (a.value, a.is_error, a.string.clone())
                };
                let (b_value, b_is_error, b_string) = {
                    let b = get_or_create_cell(sheet, cell2.row as i32, cell2.col as i32);
                    (b.value, b.is_error, b.string.clone())
                };
                if a_is_error || b_is_error || a_string.is_some() || b_string.is_some() {
                    err = true;
                } else {
                    (val, err1) = compute_cell(op_code, a_value, b_value, &mut String::new());
                }
            }
            CellPlusConstant | CellMinusConstant | CellTimesConstant | CellDivideConstant | ConstantDividesCell => {
                let a = get_or_create_cell(sheet, cell1.row as i32, cell1.col as i32);
                if a.is_error || a.string.is_some() {
                    err = true;
                } else {
                    (val, err1) = compute_cell(op_code, a.value, (cell2.row as i32) << 16 | cell2.col as i32, &mut String::new());
                }
            }
            Sleep => {
                let a = get_or_create_cell(sheet, cell1.row as i32, cell1.col as i32);
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
                            if let Some(ref_cell) = sheet.data.get(&(i as i16, j as i16)) {
                                if ref_cell.is_error || ref_cell.string.is_some() {
                                    err = true;
                                    break;
                                }
                            }
                        }
                        if err { break; }
                    }
                     
                    if !err{
                        val = compute_range_func(
                            sheet, op_code,
                            cell1.row as i16, cell1.col as i16,
                            cell2.row as i16, cell2.col as i16,
                            &mut String::new()
                        );
                    }
                    
            }
            _ => {}
        }
        (val,strval,err, err1)
    };
    {
        let c = get_or_create_cell(sheet, row as i32, col as i32); 
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

        for &dep in &sheet.data.get(&((node % 1000) as i16, (node / 1000 )as i16)).expect("not in map").dependencies {
            if let Some(indegree) = avl_tree.get_mut(&dep) {
            *indegree -= 1;
            if *indegree == 0 {
                queue.push_back(dep);
            }
            }
        }
        // get the range dependencies
        let (row,col) = (node % 1000, node / 1000);
        if sheet.buul[row as usize][col as usize] {
            for (((start_col,start_row),(end_col,end_row)), target_vector) in sheet.tuup.iter() {
                for (target_col, target_row) in target_vector {
                    if col >= *start_col as i32 && col <= *end_col as i32 && row >= *start_row as i32 && row <= *end_row as i32 {
                        let key = *target_col as i32 * 1000 + *target_row as i32;
                        if let Some(indegree) = avl_tree.get_mut(&key) {
                            *indegree -= 1;
                            if *indegree == 0 {
                                queue.push_back(key);
                            }
                        }
                    }
                }
            }
        }


        

    }

    result
} 

use OpCode::*;

pub fn remove_dependency(cell: &CellInfo, sheet: &mut Sheet) {
    let row = cell.row as i32;
    let col = cell.col as i32;

    // Check if the current cell exists in the map
    let (op_code, cell1, cell2) = if let Some(curr_cell) = sheet.data.get(&(row as i16, col as i16)) {
        (curr_cell.op_code, curr_cell.cell1.clone(), curr_cell.cell2.clone())
    } else {
        // If the cell is not in the map, return early
        return;
    };

    match op_code {
        OpCode::NoConstraint => {}
        OpCode::CellEqualsCell| OpCode::CellPlusConstant| OpCode::CellTimesConstant| OpCode::CellMinusConstant| OpCode::CellDivideConstant| OpCode::ConstantDividesCell| OpCode::Sleep => {
            if let Some(dependent_cell) = sheet.data.get_mut(&(cell1.row as i16, cell1.col as i16)) {
                dependent_cell.dependencies.remove(&(col * 1000 + row));
            } 
        }
        OpCode::Sum | OpCode::Min | OpCode::Max | OpCode::Avg | OpCode::Stdev => {
            // Handle range dependencies
            if let Some(vec) = sheet.tuup.get_mut(&((cell1.col, cell1.row), (cell2.col, cell2.row))) {
                vec.retain(|&(col, row)| col != cell.col || row != cell.row);
                if vec.is_empty() {
                    sheet.tuup.remove(&((cell1.col, cell1.row), (cell2.col, cell2.row)));
                }
            }
            for i in cell1.row..=cell2.row {
                for j in cell1.col..=cell2.col {
                    let mut flag = false;
                    // check present in other range functions
                    for(((start_col,start_row),(end_col,end_row)), _) in sheet.tuup.iter() {
                        if j >= *start_col && j <= *end_col && i >= *start_row && i <= *end_row {
                            flag = true;
                        }
                    }
                    sheet.buul[i as usize][j as usize] = flag; // Mark the cell as not used
                }
            }
        }
        OpCode::CellPlusCell| OpCode::CellMinusCell| OpCode::CellTimesCell| OpCode::CellDivideCell => {
            if let Some(dependent_cell1) = sheet.data.get_mut(&(cell1.row as i16, cell1.col as i16)) {
                dependent_cell1.dependencies.remove(&(col * 1000 + row));
            }

            if let Some(dependent_cell2) = sheet.data.get_mut(&(cell2.row as i16, cell2.col as i16)) {
                dependent_cell2.dependencies.remove(&(col * 1000 + row));
            }
        }
        _ => {}
    }
    // Update the current cell to reset its dependencies
    if let Some(curr_cell) = sheet.data.get_mut(&(row as i16, col as i16)) {
        curr_cell.cell1 = CellInfo { row: -1, col: -1 };
        curr_cell.cell2 = CellInfo { row: -1, col: -1 };
    }
}

pub fn add_to_tree(avl_tree: &mut std::collections::HashMap<i32, i32>, cell: CellInfo, sheet: &Sheet) {
    // Get the current cell from the HashMap
    if let Some(curr_cell) = sheet.data.get(&(cell.row as i16, cell.col as i16)) {
        for &curr in &curr_cell.dependencies {
            if !avl_tree.contains_key(&curr) {
                avl_tree.insert(curr, 1);
                let temp = CellInfo {
                    row: (curr % 1000) as i16,
                    col: (curr / 1000) as i16,
                };
                add_to_tree(avl_tree, temp, sheet);
            } else {
                *avl_tree.entry(curr).or_insert(1) += 1;
            }
        }
    }// check range dependency
    if sheet.buul[cell.row as usize][cell.col as usize]{
        // range dependency is there
        for (((start_col,start_row),(end_col,end_row)), target_vector) in sheet.tuup.iter() {

            for (target_col, target_row) in target_vector {
            if cell.col >= *start_col && cell.col <= *end_col && cell.row >= *start_row && cell.row <= *end_row {
               // dependent
                let key = *target_col as i32 * 1000 + *target_row as i32;
                if !avl_tree.contains_key(&key) {
                    avl_tree.insert(key, 1);
                    let temp = CellInfo {
                        row: *target_row,
                        col: *target_col,
                    };
                    add_to_tree(avl_tree, temp, sheet);
                } else {
                    *avl_tree.entry(key).or_insert(1) += 1;
                }
            }
            }
        }
    }
}

pub fn add_constraints(curr_cell: CellInfo,cell1: CellInfo,cell2: CellInfo,op_code: OpCode,sheet: &mut Sheet,status: &mut String,sleep_timer: &mut i32,) {
    let key = curr_cell.col as i32 * 1000 + curr_cell.row as i32;
    let mut ans = 0;
    let mut calc_error = false;
    let mut calc_error1 = false;
    let mut stringans = String::new();
    // Create a HashMap named avl_tree where the key is key and the value is indegree (0 by default)
    let mut avl_tree: std::collections::HashMap<i32, i32> = std::collections::HashMap::new();
    avl_tree.insert(key, 0);
    let temp = CellInfo { row: -1, col: -1 };

    add_to_tree(&mut avl_tree, curr_cell.clone(), sheet); 

    match op_code {
        OpCode::NoConstraint | String => {remove_dependency(&curr_cell, sheet);
            let cell = get_or_create_cell(sheet, curr_cell.row as i32, curr_cell.col as i32); 
                ans = cell.value;
                if cell.is_error {calc_error = true;}
                cell.op_code = op_code;
                cell.cell1 = CellInfo { row: -1, col: -1 };
                cell.cell2 = CellInfo { row: -1, col: -1 }; 
        }
        OpCode::CellEqualsCell => {
            if check_cycle(&avl_tree, &cell1, &temp) {
                *status = String::from("circular error");return;
            }
            remove_dependency(&curr_cell, sheet);
            let cell = get_or_create_cell(sheet, curr_cell.row as i32, curr_cell.col as i32); 
            cell.cell1 = cell1.clone();
            cell.cell2 = CellInfo { row: -1, col: -1 };
            cell.op_code = op_code;
            let ref_cell = get_or_create_cell(sheet, cell1.row as i32, cell1.col as i32); 
            match (&ref_cell.string, ref_cell.is_error) {
                (_, true) => calc_error = true,
                (Some(s), false) => stringans = s.clone(),
                (None, false) => ans = ref_cell.value,
            }
            ref_cell.dependencies.insert(key);
        }
        OpCode::Sleep => {
            if check_cycle(&avl_tree, &cell1, &temp) {
                *status = String::from("circular error");return;
            }
            remove_dependency(&curr_cell, sheet);
            let cell = get_or_create_cell(sheet, curr_cell.row as i32, curr_cell.col as i32); 
            cell.cell1 = cell1.clone();
            cell.cell2 = CellInfo { row: -1, col: -1 };
            cell.op_code = op_code;
            
            let ref_cell = get_or_create_cell(sheet, cell1.row as i32, cell1.col as i32);
            calc_error = ref_cell.is_error || ref_cell.string.is_some();
            if !calc_error {
                ans = ref_cell.value;
                *sleep_timer += ref_cell.value;
            }
            ref_cell.dependencies.insert(key);
        }
        OpCode::CellPlusConstant| OpCode::CellMinusConstant| OpCode::CellTimesConstant| OpCode::CellDivideConstant| OpCode::ConstantDividesCell => {
            if check_cycle(&avl_tree, &cell1, &temp) {
                *status = String::from("circular error");return;
            }
            remove_dependency(&curr_cell, sheet);
            let cell = get_or_create_cell(sheet, curr_cell.row as i32, curr_cell.col as i32);
            cell.cell1 = cell1.clone();
            cell.cell2 = cell2.clone();
            cell.op_code = op_code;

            let value = (cell2.row as i32) << 16 | (cell2.col as i32 & 0xFFFF);
            let ref_cell = get_or_create_cell(sheet, cell1.row as i32, cell1.col as i32);
            calc_error = ref_cell.is_error || ref_cell.string.is_some();
            if !calc_error {
                let (a, b) = compute_cell(op_code, ref_cell.value, value, status);
                ans = a;
                calc_error1 = b;
            }
            ref_cell.dependencies.insert(key);
        }
        OpCode::Sum | OpCode::Min | OpCode::Max | OpCode::Avg | OpCode::Stdev => {
            if check_cycle_range_funcs(&avl_tree, &cell1, &cell2) {
                *status = String::from("circular error");return;
            }
            for i in cell1.row..=cell2.row {
                for j in cell1.col..=cell2.col {
                    sheet.buul[i as usize][j as usize] = true; // Mark the cell as used
                    if let Some(ref_cell) = sheet.data.get(&(i as i16, j as i16)) {
                        if ref_cell.is_error || ref_cell.string.is_some() {
                            calc_error = true;
                        }
                    }
                }
            }
            remove_dependency(&curr_cell, sheet);
            if let Some(existing) = sheet.tuup.get_mut(&((cell1.col, cell1.row), (cell2.col, cell2.row))) {
                existing.push((curr_cell.col, curr_cell.row));
            } else {
                sheet.tuup.insert(((cell1.col, cell1.row), (cell2.col, cell2.row)), vec![(curr_cell.col, curr_cell.row)]);
            }
            let cell = get_or_create_cell(sheet, curr_cell.row as i32, curr_cell.col as i32);
            cell.cell1 = cell1.clone();
            cell.cell2 = cell2.clone();
            cell.op_code = op_code;
            ans = compute_range_func(sheet,op_code,cell1.row as i16,cell1.col as i16,cell2.row as i16,cell2.col as i16,status,);
        }
        OpCode::CellPlusCell| OpCode::CellMinusCell| OpCode::CellTimesCell| OpCode::CellDivideCell => {
            if check_cycle(&avl_tree, &cell1, &cell2) {
                *status = String::from("circular error");return;
            }
            remove_dependency(&curr_cell, sheet);
            let cell = get_or_create_cell(sheet, curr_cell.row as i32, curr_cell.col as i32);
            cell.cell1 = cell1.clone();
            cell.cell2 = cell2.clone();
            cell.op_code = op_code;

            let (a_value, a_is_error, a_string, b_value, b_is_error, b_string) = {
                let a = get_or_create_cell(sheet, cell1.row as i32, cell1.col as i32);
                let a_value = a.value;
                let a_is_error = a.is_error;
                let a_string = a.string.clone();

                let b = get_or_create_cell(sheet, cell2.row as i32, cell2.col as i32);
                let b_value = b.value;
                let b_is_error = b.is_error;
                let b_string = b.string.clone();

                (a_value, a_is_error, a_string, b_value, b_is_error, b_string)
            };

            if a_is_error || b_is_error || a_string.is_some() || b_string.is_some() {
                calc_error = true;
            } else {
                let (computed_value, computed_error) = compute_cell(op_code, a_value, b_value, status);
                ans = computed_value;
                calc_error1 = computed_error;
            }

            {
                let a = get_or_create_cell(sheet, cell1.row as i32, cell1.col as i32);
                a.dependencies.insert(key);
            }

            {
                let b = get_or_create_cell(sheet, cell2.row as i32, cell2.col as i32);
                b.dependencies.insert(key);
            }
            
        }
    }
    let cell = get_or_create_cell(sheet, curr_cell.row as i32, curr_cell.col as i32); 
    cell.is_error = calc_error || calc_error1;
    if !cell.is_error && cell.op_code != OpCode::String {
        cell.value = ans;
        cell.string = None;
    }
    if op_code == OpCode::CellEqualsCell && !stringans.is_empty() {
        cell.string = Some(stringans.clone());
    }
    let sorted = topological_sort(&mut avl_tree, &sheet); 
    *status = String::from("ok");  
    for i in sorted.into_iter() {
        let row = i % 1000;
        let col = i / 1000;
        if row == curr_cell.row as i32 && col == curr_cell.col as i32 {
            continue;
        }
        recalculate(sheet, row as usize, col as usize, sleep_timer); 
    }
     
}

pub fn update_dependencies(old_cell_row: i16,old_cell_col: i16,new_cell_row: i16,new_cell_col: i16,sheet: &mut Sheet,
) {
    // Goes to the dependency set of cells depending on the old cell and updates references
    let mut avl_tree: std::collections::HashMap<i32, i32> = std::collections::HashMap::new();
    avl_tree.insert(new_cell_col as i32 * 1000 + new_cell_row as i32, 0);
    add_to_tree(&mut avl_tree,CellInfo {row: new_cell_row,col: new_cell_col,},sheet,);

    let sorted = topological_sort(&mut avl_tree, sheet);

    for i in sorted.into_iter() {
        let row = i % 1000;
        let col = i / 1000;

        if row == new_cell_row as i32 && col == new_cell_col as i32 {
            continue;
        }

        if let Some(cell) = sheet.data.get_mut(&(row as i16, col as i16)) {
            if matches!(
                cell.op_code,
                OpCode::Sum | OpCode::Min | OpCode::Max | OpCode::Avg | OpCode::Stdev
            ) {
                //TODO handle range dependencies
                recalculate(sheet, row as usize, col as usize, &mut 0);
            } else {
                if cell.cell1.row == old_cell_row && cell.cell1.col == old_cell_col {
                    cell.cell1 = CellInfo {
                        row: new_cell_row,
                        col: new_cell_col,
                    };
                }
                if cell.cell2.row == old_cell_row && cell.cell2.col == old_cell_col {
                    cell.cell2 = CellInfo {
                        row: new_cell_row,
                        col: new_cell_col,
                    };
                }
            }
        }  
    }
}

pub fn change_dependecy_set(  new_cell: &mut Cell, sheet: &mut Sheet, del_range_dependencies: bool, change_to_row: i16,  change_to_col: i16,current_row: i16,current_col: i16,) {
    // Removes the range/non-range dependencies in the given dependency set
    for &dependency in new_cell.dependencies.clone().iter() {
        let row = dependency % 1000;
        let col = dependency / 1000;

        if let Some(dependent_cell) = sheet.data.get_mut(&(row as i16, col as i16)) {
            if matches!(
                dependent_cell.op_code,
                OpCode::Sum | OpCode::Min | OpCode::Max | OpCode::Avg | OpCode::Stdev
            ) && del_range_dependencies
            {
                new_cell.dependencies.remove(&(col * 1000 + row));
            } else if !del_range_dependencies
                && !matches!(
                    dependent_cell.op_code,
                    OpCode::Sum | OpCode::Min | OpCode::Max | OpCode::Avg | OpCode::Stdev
                )
            {
                new_cell.dependencies.remove(&(col * 1000 + row));

                if dependent_cell.cell1.row == current_row && dependent_cell.cell1.col == current_col {
                    dependent_cell.cell1 = CellInfo {
                        row: change_to_row,
                        col: change_to_col,
                    };
                }
                if dependent_cell.cell2.row == current_row && dependent_cell.cell2.col == current_col {
                    dependent_cell.cell2 = CellInfo {
                        row: change_to_row,
                        col: change_to_col,
                    };
                }
            }
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


pub fn sort_sheet(
    sheet: &mut Sheet,
    col1: i32,
    row1: i32,
    col2: i32,
    row2: i32,
    sort_key: &str,
    is_column: bool,
    sort_order: &str,
) {
    let mut vec: Vec<Vec<Cell>> = Vec::new();

    if is_column {
        for i in row1..=row2 {
            let mut temp: Vec<Cell> = Vec::new();
            for j in col1..=col2 {
                if let Some(cell) = sheet.data.get(&(i as i16, j as i16)) {
                    temp.push(cell.clone());
                } else {
                    temp.push(Cell {
                        value: 0,
                        is_error: false,
                        string: None,
                        op_code: OpCode::NoConstraint,
                        cell1: CellInfo { row: -1, col: -1 },
                        cell2: CellInfo { row: -1, col: -1 },
                        dependencies: HashSet::new(),
                    });
                }
            }
            vec.push(temp);
        }

        let col_num = col_name_to_col_num(sort_key);
        if sort_order == "asc" {
            vec.sort_by_key(|k| k[(col_num - col1) as usize].clone().value);
        } else {
            vec.sort_by_key(|k| std::cmp::Reverse(k[(col_num - col1) as usize].clone().value));
        }
    } else {
        for i in col1..=col2 {
            let mut temp: Vec<Cell> = Vec::new();
            for j in row1..=row2 {
                if let Some(cell) = sheet.data.get(&(j as i16, i as i16)) {
                    temp.push(cell.clone());
                } else {
                    temp.push(Cell {
                        value: 0,
                        is_error: false,
                        string: None,
                        op_code: OpCode::NoConstraint,
                        cell1: CellInfo { row: -1, col: -1 },
                        cell2: CellInfo { row: -1, col: -1 },
                        dependencies: HashSet::new(),
                    });
                }
            }
            vec.push(temp);
        }

        let row_num = sort_key.parse::<i32>().unwrap() - 1;
        if sort_order == "asc" {
            vec.sort_by_key(|k| k[(row_num - row1) as usize].clone().value);
        } else {
            vec.sort_by_key(|k| std::cmp::Reverse(k[(row_num - row1) as usize].clone().value));
        }
    }

    for i in row1..=row2 {
        for j in col1..=col2 {
            if let Some(cell) = vec.get((i - row1) as usize).and_then(|row| row.get((j - col1) as usize)) {
                sheet.data.insert((i as i16, j as i16), cell.clone());
            }
        }
    }
}
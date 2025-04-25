use crate::sheet_functions::OpCode;
use crate::sheet_functions::{Cell, Sheet};
use crate::sheet_functions::{
    CellInfo, add_to_tree, col_name_to_col_num, recalculate, topological_sort,
};
use serde::de;
use serde::ser::SerializeTuple;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::SeqAccess, de::Visitor};
use std::collections::HashSet;
use std::fmt;

// impl Serialize for CellInfo {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         let mut tuple = serializer.serialize_tuple(2)?;
//         tuple.serialize_element(&self.row)?;
//         tuple.serialize_element(&self.col)?;
//         tuple.end()
//     }
// }

// impl<'de> Deserialize<'de> for CellInfo {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         struct CellInfoVisitor;

//         impl<'de> Visitor<'de> for CellInfoVisitor {
//             type Value = CellInfo;
//             fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//                 formatter.write_str("a tuple [row, col]")
//             }
//             fn visit_seq<V>(self, mut seq: V) -> Result<CellInfo, V::Error>
//             where
//                 V: SeqAccess<'de>,
//             {
//                 let row = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
//                 let col = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
//                 Ok(CellInfo { row, col })
//             }
//         }
//         deserializer.deserialize_tuple(2, CellInfoVisitor)
//     }
// }

// impl Serialize for Cell {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         let mut tuple = serializer.serialize_tuple(7)?;
//         tuple.serialize_element(&self.value)?;
//         tuple.serialize_element(&self.string)?;
//         tuple.serialize_element(&self.is_error)?;
//         tuple.serialize_element(&self.op_code)?;
//         tuple.serialize_element(&self.cell1)?;
//         tuple.serialize_element(&self.cell2)?;
//         tuple.serialize_element(&self.dependencies)?;
//         tuple.end()
//     }
// }

// impl<'de> Deserialize<'de> for Cell {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         struct CellVisitor;

//         impl<'de> Visitor<'de> for CellVisitor {
//             type Value = Cell;

//             fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//                 formatter.write_str("a compact Cell as a 7-element tuple")
//             }

//             fn visit_seq<V>(self, mut seq: V) -> Result<Cell, V::Error>
//             where
//                 V: SeqAccess<'de>,
//             {
//                 Ok(Cell {
//                     value: seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?,
//                     string: seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?,
//                     is_error: seq.next_element()?.ok_or_else(|| de::Error::invalid_length(2, &self))?,
//                     op_code: seq.next_element()?.ok_or_else(|| de::Error::invalid_length(3, &self))?,
//                     cell1: seq.next_element()?.ok_or_else(|| de::Error::invalid_length(4, &self))?,
//                     cell2: seq.next_element()?.ok_or_else(|| de::Error::invalid_length(5, &self))?,
//                     dependencies: seq.next_element()?.ok_or_else(|| de::Error::invalid_length(6, &self))?,
//                 })
//             }
//         }

//         deserializer.deserialize_tuple(7, CellVisitor)
//     }
// }

pub fn update_dependencies(
    old_cell_row: i16,
    old_cell_col: i16,
    new_cell_row: i16,
    new_cell_col: i16,
    sheet: &mut Sheet,
) {
    // Goes to the dependency set of cells depending on the old cell and updates references
    let mut avl_tree: std::collections::HashMap<i32, i32> = std::collections::HashMap::new();
    avl_tree.insert(new_cell_col as i32 * 1000 + new_cell_row as i32, 0);
    add_to_tree(
        &mut avl_tree,
        CellInfo {
            row: new_cell_row,
            col: new_cell_col,
        },
        sheet,
    );

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

pub fn change_dependecy_set(
    new_cell: &mut Cell,
    sheet: &mut Sheet,
    del_range_dependencies: bool,
    change_to_row: i16,
    change_to_col: i16,
    current_row: i16,
    current_col: i16,
) {
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

                if dependent_cell.cell1.row == current_row
                    && dependent_cell.cell1.col == current_col
                {
                    dependent_cell.cell1 = CellInfo {
                        row: change_to_row,
                        col: change_to_col,
                    };
                }
                if dependent_cell.cell2.row == current_row
                    && dependent_cell.cell2.col == current_col
                {
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
    for i in sorted.into_iter() {
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

    if is_column {
        for i in row1..=row2 {
            for j in col1..=col2 {
                if let Some(cell) = vec
                    .get((i - row1) as usize)
                    .and_then(|row| row.get((j - col1) as usize))
                {
                    sheet.data.insert((i as i16, j as i16), cell.clone());
                }
            }
        }
    } else {
        for i in col1..=col2 {
            for j in row1..=row2 {
                if let Some(cell) = vec
                    .get((i - col1) as usize)
                    .and_then(|row| row.get((j - row1) as usize))
                {
                    sheet.data.insert((j as i16, i as i16), cell.clone());
                }
            }
        }
    }
}

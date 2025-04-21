use crate::sheet_functions::{CellInfo, col_num_to_col_name};
use crate::parser;
use crate::sheet_functions::OpCode;
use crate::sheet_functions::OpCode::*;
use serde::{Serialize, Deserialize};
use crate::sheet_functions::{self,Sheet,Cell};
use crate::ui::themes::{self,Theme};
use crate::ui::themes::themes;

use std::string::String;

#[derive(Clone, Serialize, Deserialize)]
pub struct Sheets {
    pub sheet: Sheet,
    pub name: String,
}

#[derive(Clone)]
pub enum Action {
    CellEdit {
        sheet_index: usize,
        row: usize,
        col: usize,
        old_cell: Cell,
        new_cell: Cell,
        command: String,
    },
    NewSheet {
        sheet: Sheets,
        index: usize,
    },
    DeleteSheet {
        sheet: Sheets,
        index: usize,
    },
    ClearSheet {
        sheet_index: usize,
        old_data: Vec<Vec<Cell>>,
    },
    Cut {
        sheet_index: usize,
        row: usize,
        col: usize,
        old_cell: Cell,
    },
    Paste {
        sheet_index: usize,
        row: usize,
        col: usize,
        old_cell: Cell,
        new_cell: Cell,
        command: String,
    },
}

#[derive(Debug, PartialEq)]
pub enum Mode { Normal, Insert }

#[derive(Debug, PartialEq)]
pub enum Menu {
    Save,
    Open,
    Theme,
    NewSheet,
    DeleteSheet,
    None,
}


pub struct SpreadsheetApp {
    pub sheets: Vec<Sheets>,
    pub current_sheet_index: usize,
    pub formula: String,
    pub status: String,
    pub mode: Mode,
    pub selected_cell: Option<(usize, usize)>,
    pub row_start: i32,
    pub col_start: i32,
    pub time: f32,
    pub timer: i32,
    pub editing_value: String,
    pub is_editing: bool,
    pub show_menu: Menu,
    pub save_filename: String,
    pub open_filename: String,
    pub theme: Theme,
    pub new_sheet_rows: String,
    pub new_sheet_cols: String,
    pub new_sheet_name: String,
    pub undo_stack: Vec<Action>,
    pub redo_stack: Vec<Action>,
    pub clipboard: Option<(Cell, String, usize, usize)>, // Stores (cell, command, row, col) for copy/cut
}


impl SpreadsheetApp {
    pub fn new(mut sheets: Vec<Sheets>) -> Self {
        Self {
            sheets,
            current_sheet_index: 0,
            formula: String::new(),
            status: "ok".into(),
            mode: Mode::Normal,
            selected_cell: None,
            row_start: 0,
            col_start: 0,
            time: 0.0,
            timer: 0,
            editing_value: String::new(),
            is_editing: false,
            show_menu: Menu::None,
            save_filename: String::from("sheet"),
            open_filename: String::new(),
            theme: themes[0].clone(),
            new_sheet_rows: String::new(),
            new_sheet_cols: String::new(),
            new_sheet_name: String::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            clipboard: None,
        }
    }

    pub fn undo(&mut self) {
        if let Some(action) = self.undo_stack.pop() {
            match action {
                Action::CellEdit {
                    sheet_index,
                    row,
                    col,
                    old_cell,
                    new_cell,
                    command,
                } => {
                    let sheet_rows = self.sheets[sheet_index].sheet.rows;
                    let sheet_cols = self.sheets[sheet_index].sheet.cols;
                    let original_cmd = if old_cell.string.is_some() {
                        format!("{}{}={}", col_num_to_col_name(col as i32), row + 1, old_cell.string.as_ref().unwrap())
                    } else if old_cell.is_error {
                        format!("{}{}=Err", col_num_to_col_name(col as i32), row + 1)
                    } else {
                        format!("{}{}={}", col_num_to_col_name(col as i32), row + 1, old_cell.value)
                    };
                    // Remove dependencies of the current cell
                    sheet_functions::remove_dependency(
                        &CellInfo {
                            row: row as i16,
                            col: col as i16,
                        },
                        &mut self.sheets[sheet_index].sheet,
                    );
                    parser::parse_command(
                        &original_cmd,
                        &mut self.row_start,
                        &mut self.col_start,
                        &mut self.time,
                        &mut self.status,
                        &sheet_rows,
                        &sheet_cols,
                        &mut self.sheets[sheet_index].sheet,
                        &mut true,
                    );
                    let new_cell1 = self.sheets[sheet_index].sheet.data[row][col].clone();
                    self.redo_stack.push(Action::CellEdit {
                        sheet_index,
                        row,
                        col,
                        old_cell: old_cell.clone(),
                        new_cell: new_cell1,
                        command: command.clone(),
                    });
                    // Recalculate the sheet to update dependent cells
                    let sorted = sheet_functions::topological_sort(
                        &mut std::collections::HashMap::new(),
                        &self.sheets[sheet_index].sheet,
                    );
                    for i in sorted {
                        let r = i % 1000;
                        let c = i / 1000;
                        sheet_functions::recalculate(
                            &mut self.sheets[sheet_index].sheet,
                            r as usize,
                            c as usize,
                            &mut self.timer,
                        );
                    }
                    self.status = "Undone cell edit".to_string();
                }
                Action::NewSheet { sheet, index } => {
                    let removed_sheet = self.sheets.remove(index);
                    self.current_sheet_index = self.current_sheet_index.min(self.sheets.len().saturating_sub(1));
                    self.redo_stack.push(Action::NewSheet {
                        sheet: removed_sheet,
                        index,
                    });
                    self.status = "Undone new sheet".to_string();
                }
                Action::DeleteSheet { sheet, index } => {
                    self.sheets.insert(index, sheet.clone());
                    self.current_sheet_index = index;
                    self.redo_stack.push(Action::DeleteSheet { sheet, index });
                    self.status = "Undone delete sheet".to_string();
                }
                Action::ClearSheet { sheet_index, old_data } => {
                    let new_data = self.sheets[sheet_index].sheet.data.clone();
                    self.sheets[sheet_index].sheet.data = old_data;
                    self.redo_stack.push(Action::ClearSheet {
                        sheet_index,
                        old_data: new_data,
                    });
                    // Recalculate the sheet to update dependent cells
                    let sorted = sheet_functions::topological_sort(
                        &mut std::collections::HashMap::new(),
                        &self.sheets[sheet_index].sheet,
                    );
                    for i in sorted {
                        let r = i % 1000;
                        let c = i / 1000;
                        sheet_functions::recalculate(
                            &mut self.sheets[sheet_index].sheet,
                            r as usize,
                            c as usize,
                            &mut self.timer,
                        );
                    }
                    self.status = "Undone clear sheet".to_string();
                }
                Action::Cut {
                    sheet_index,
                    row,
                    col,
                    old_cell,
                } => {
                    let sheet_rows = self.sheets[sheet_index].sheet.rows;
                    let sheet_cols = self.sheets[sheet_index].sheet.cols;
                    let new_cell = self.sheets[sheet_index].sheet.data[row][col].clone();
                    // Restore the original cell content
                    let original_cmd = if old_cell.string.is_some() {
                        format!("{}{}={}", col_num_to_col_name(col as i32), row + 1, old_cell.string.as_ref().unwrap())
                    } else if old_cell.is_error {
                        format!("{}{}=Err", col_num_to_col_name(col as i32), row + 1)
                    } else {
                        format!("{}{}={}", col_num_to_col_name(col as i32), row + 1, old_cell.value)
                    };
                    // Remove dependencies of the current cell
                    sheet_functions::remove_dependency(
                        &CellInfo {
                            row: row as i16,
                            col: col as i16,
                        },
                        &mut self.sheets[sheet_index].sheet,
                    );
                    parser::parse_command(
                        &original_cmd,
                        &mut self.row_start,
                        &mut self.col_start,
                        &mut self.time,
                        &mut self.status,
                        &sheet_rows,
                        &sheet_cols,
                        &mut self.sheets[sheet_index].sheet,
                        &mut true,
                    );
                    self.redo_stack.push(Action::Cut {
                        sheet_index,
                        row,
                        col,
                        old_cell: new_cell,
                    });
                    // Recalculate the sheet to update dependent cells
                    let sorted = sheet_functions::topological_sort(
                        &mut std::collections::HashMap::new(),
                        &self.sheets[sheet_index].sheet,
                    );
                    for i in sorted {
                        let r = i % 1000;
                        let c = i / 1000;
                        sheet_functions::recalculate(
                            &mut self.sheets[sheet_index].sheet,
                            r as usize,
                            c as usize,
                            &mut self.timer,
                        );
                    }
                    self.status = "Undone cut".to_string();
                }
                Action::Paste {
                    sheet_index,
                    row,
                    col,
                    old_cell,
                    new_cell,
                    command,
                } => {
                    let sheet_rows = self.sheets[sheet_index].sheet.rows;
                    let sheet_cols = self.sheets[sheet_index].sheet.cols;
                    let original_cmd = if old_cell.string.is_some() {
                        format!("{}{}={}", col_num_to_col_name(col as i32), row + 1, old_cell.string.as_ref().unwrap())
                    } else if old_cell.is_error {
                        format!("{}{}=Err", col_num_to_col_name(col as i32), row + 1)
                    } else {
                        format!("{}{}={}", col_num_to_col_name(col as i32), row + 1, old_cell.value)
                    };
                    // Remove dependencies of the current cell
                    sheet_functions::remove_dependency(
                        &CellInfo {
                            row: row as i16,
                            col: col as i16,
                        },
                        &mut self.sheets[sheet_index].sheet,
                    );
                    parser::parse_command(
                        &original_cmd,
                        &mut self.row_start,
                        &mut self.col_start,
                        &mut self.time,
                        &mut self.status,
                        &sheet_rows,
                        &sheet_cols,
                        &mut self.sheets[sheet_index].sheet,
                        &mut true,
                    );
                    let new_cell1 = self.sheets[sheet_index].sheet.data[row][col].clone();
                    self.redo_stack.push(Action::Paste {
                        sheet_index,
                        row,
                        col,
                        old_cell: old_cell.clone(),
                        new_cell: new_cell1,
                        command,
                    });
                    // Recalculate the sheet to update dependent cells
                    let sorted = sheet_functions::topological_sort(
                        &mut std::collections::HashMap::new(),
                        &self.sheets[sheet_index].sheet,
                    );
                    for i in sorted {
                        let r = i % 1000;
                        let c = i / 1000;
                        sheet_functions::recalculate(
                            &mut self.sheets[sheet_index].sheet,
                            r as usize,
                            c as usize,
                            &mut self.timer,
                        );
                    }
                    self.status = "Undone paste".to_string();
                }
            }
        } else {
            self.status = "Nothing to undo".to_string();
        }
    }

    pub fn redo(&mut self) {
        if let Some(action) = self.redo_stack.pop() {
            match action {
                Action::CellEdit {
                    sheet_index,
                    row,
                    col,
                    old_cell: _,
                    new_cell: _,
                    command,
                } => {
                    let sheet_rows = self.sheets[sheet_index].sheet.rows;
                    let sheet_cols = self.sheets[sheet_index].sheet.cols;
                    let old_cell = self.sheets[sheet_index].sheet.data[row][col].clone();
                    // Remove dependencies of the current cell
                    sheet_functions::remove_dependency(
                        &CellInfo {
                            row: row as i16,
                            col: col as i16,
                        },
                        &mut self.sheets[sheet_index].sheet,
                    );
                    let mut print_enabled = true;
                    parser::parse_command(
                        &command,
                        &mut self.row_start,
                        &mut self.col_start,
                        &mut self.time,
                        &mut self.status,
                        &sheet_rows,
                        &sheet_cols,
                        &mut self.sheets[sheet_index].sheet,
                        &mut print_enabled, 
                    );
                    let new_cell = self.sheets[sheet_index].sheet.data[row][col].clone();
                    self.undo_stack.push(Action::CellEdit {
                        sheet_index,
                        row,
                        col,
                        old_cell,
                        new_cell,
                        command,
                    });
                    // Recalculate the sheet to update dependent cells
                    let sorted = sheet_functions::topological_sort(
                        &mut std::collections::HashMap::new(),
                        &self.sheets[sheet_index].sheet,
                    );
                    for i in sorted {
                        let r = i % 1000;
                        let c = i / 1000;
                        sheet_functions::recalculate(
                            &mut self.sheets[sheet_index].sheet,
                            r as usize,
                            c as usize,
                            &mut self.timer,
                        );
                    }
                    self.status = "Redone cell edit".to_string();
                }
                Action::NewSheet { sheet, index } => {
                    self.sheets.insert(index, sheet.clone());
                    self.current_sheet_index = index;
                    self.undo_stack.push(Action::NewSheet { sheet, index });
                    self.status = "Redone new sheet".to_string();
                }
                Action::DeleteSheet { sheet, index } => {
                    let removed_sheet = self.sheets.remove(index);
                    self.current_sheet_index = self.current_sheet_index.min(self.sheets.len().saturating_sub(1));
                    self.undo_stack.push(Action::DeleteSheet {
                        sheet: removed_sheet,
                        index,
                    });
                    self.status = "Redone delete sheet".to_string();
                }
                Action::ClearSheet { sheet_index, old_data } => {
                    let new_data = self.sheets[sheet_index].sheet.data.clone();
                    self.sheets[sheet_index].sheet.data = old_data;
                    self.undo_stack.push(Action::ClearSheet {
                        sheet_index,
                        old_data: new_data,
                    });
                    // Recalculate the sheet to update dependent cells
                    let sorted = sheet_functions::topological_sort(
                        &mut std::collections::HashMap::new(),
                        &self.sheets[sheet_index].sheet,
                    );
                    for i in sorted {
                        let r = i % 1000;
                        let c = i / 1000;
                        sheet_functions::recalculate(
                            &mut self.sheets[sheet_index].sheet,
                            r as usize,
                            c as usize,
                            &mut self.timer,
                        );
                    }
                    self.status = "Redone clear sheet".to_string();
                }
                Action::Cut {
                    sheet_index,
                    row,
                    col,
                    old_cell,
                } => {
                    let sheet_rows = self.sheets[sheet_index].sheet.rows;
                    let sheet_cols = self.sheets[sheet_index].sheet.cols;
                    let new_cell = self.sheets[sheet_index].sheet.data[row][col].clone();
                    // Clear the cell
                    let clear_cmd = format!("{}{}=0", col_num_to_col_name(col as i32), row + 1);
                    // Remove dependencies of the current cell
                    sheet_functions::remove_dependency(
                        &CellInfo {
                            row: row as i16,
                            col: col as i16,
                        },
                        &mut self.sheets[sheet_index].sheet,
                    );
                    parser::parse_command(
                        &clear_cmd,
                        &mut self.row_start,
                        &mut self.col_start,
                        &mut self.time,
                        &mut self.status,
                        &sheet_rows,
                        &sheet_cols,
                        &mut self.sheets[sheet_index].sheet,
                        &mut true,
                    );
                    self.undo_stack.push(Action::Cut {
                        sheet_index,
                        row,
                        col,
                        old_cell: new_cell,
                    });
                    // Recalculate the sheet to update dependent cells
                    let sorted = sheet_functions::topological_sort(
                        &mut std::collections::HashMap::new(),
                        &self.sheets[sheet_index].sheet,
                    );
                    for i in sorted {
                        let r = i % 1000;
                        let c = i / 1000;
                        sheet_functions::recalculate(
                            &mut self.sheets[sheet_index].sheet,
                            r as usize,
                            c as usize,
                            &mut self.timer,
                        );
                    }
                    self.status = "Redone cut".to_string();
                }
                Action::Paste {
                    sheet_index,
                    row,
                    col,
                    old_cell,
                    new_cell,
                    command,
                } => {
                    let sheet_rows = self.sheets[sheet_index].sheet.rows;
                    let sheet_cols = self.sheets[sheet_index].sheet.cols;
                    let prev_cell = self.sheets[sheet_index].sheet.data[row][col].clone();
                    // Remove dependencies of the current cell
                    sheet_functions::remove_dependency(
                        &CellInfo {
                            row: row as i16,
                            col: col as i16,
                        },
                        &mut self.sheets[sheet_index].sheet,
                    );
                    parser::parse_command(
                        &command,
                        &mut self.row_start,
                        &mut self.col_start,
                        &mut self.time,
                        &mut self.status,
                        &sheet_rows,
                        &sheet_cols,
                        &mut self.sheets[sheet_index].sheet,
                        &mut true,
                    );
                    let new_cell_after_redo = self.sheets[sheet_index].sheet.data[row][col].clone();
                    self.undo_stack.push(Action::Paste {
                        sheet_index,
                        row,
                        col,
                        old_cell: prev_cell,
                        new_cell: new_cell_after_redo,
                        command,
                    });
                    // Recalculate the sheet to update dependent cells
                    let sorted = sheet_functions::topological_sort(
                        &mut std::collections::HashMap::new(),
                        &self.sheets[sheet_index].sheet,
                    );
                    for i in sorted {
                        let r = i % 1000;
                        let c = i / 1000;
                        sheet_functions::recalculate(
                            &mut self.sheets[sheet_index].sheet,
                            r as usize,
                            c as usize,
                            &mut self.timer,
                        );
                    }
                    self.status = "Redone paste".to_string();
                }
            }
        } else {
            self.status = "Nothing to redo".to_string();
        }
    }
}


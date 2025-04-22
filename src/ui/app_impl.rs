use crate::sheet_functions::{CellInfo, col_num_to_col_name};
use crate::parser; 
use serde::{Serialize, Deserialize};
use crate::sheet_functions::{self,Sheet,Cell};
use crate::ui::themes::Theme;
use crate::ui::themes::THEMES;
use crate::ui::utils;
use std::string::String;

#[derive(Clone, Serialize, Deserialize)]
pub struct Sheets {
    pub sheet: Sheet,
    pub name: String,
}

#[derive(Clone,Debug)]
pub enum Action {
    // NewSheet {
    //     sheet: Sheets,
    //     index: usize,
    // },
    // DeleteSheet {
    //     sheet: Sheets,
    //     index: usize,
    // },
    // ClearSheet {
    //     sheet_index: usize,
    //     old_data: Vec<Vec<Cell>>,
    // },
    // Deleted {
    //     sheet_index: usize,
    //     row: i16,
    //     col: i16,
    //     deleted_cell: Cell,
    // },
    Inserted {
        sheet_index : usize,
        row : i16,
        col : i16,
        previous_cell: Cell,
    },
    CutAction{
        sheet_index: usize,
        row1: i16,
        col1: i16,
        previous_cell1: Cell,
        row2: i16,
        col2: i16,
        previous_cell2: Cell,
    },//1 -> cut from here , 2-> pasted here
    FindAndReplace {
        sheet_index: usize,
        changes: Vec<(usize, usize, Cell, Cell, String)>, // (row, col, old_cell, new_cell, command)
    },
}

#[derive(Clone,PartialEq)]
pub enum CutCopy{
    Cut,
    Copied,
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
    FindAndReplace,
    PlotGraph,
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
    pub clipboard: Option<(Cell,CutCopy,i16,i16)>,
    pub find_text: String,
    pub replace_text: String,
    pub cut_copied_cell: Option<(i16,i16)>,
    pub plot_column1: String, // First column name (e.g., "A")
    pub plot_column2: String, // Second column name (e.g., "B")
    pub plot_row_start: String, // Start row (e.g., "1")
    pub plot_row_end: String, // End row (e.g., "5")
    pub show_plot: bool, // Whether to show the plot window
}





impl SpreadsheetApp {
    pub fn new(sheets: Vec<Sheets>) -> Self {
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
            theme: THEMES[0].clone(),
            new_sheet_rows: String::new(),
            new_sheet_cols: String::new(),
            new_sheet_name: String::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            clipboard: None,
            find_text: String::new(),
            replace_text: String::new(),
            cut_copied_cell : None,
            plot_column1: String::new(),
            plot_column2: String::new(),
            plot_row_start: String::new(),
            plot_row_end: String::new(),
            show_plot: false,
        }
    }

    pub fn find_and_replace(&mut self) {
        self.clear_clipboard();
        let mut sheet = &mut self.sheets[self.current_sheet_index].sheet;
        let mut changes = Vec::new();
        let find_text = self.find_text.clone();
        let replace_text = self.replace_text.clone();

        for row in 0..sheet.rows as usize {
            for col in 0..sheet.cols as usize {
                let cell = &sheet.data[row][col];
                let cell_content = if let Some(s) = &cell.string {
                    s.clone()
                } else if cell.is_error {
                    "Err".to_string()
                } else {
                    cell.value.to_string()
                };

                if cell_content == find_text {
                    let sheet1 = sheet.clone();
                    let old_cell = cell.clone();
                    let command = format!("{}{}={}", col_num_to_col_name(col as i32), row + 1, replace_text);
                    sheet_functions::remove_dependency(
                        &CellInfo {
                            row: row as i16,
                            col: col as i16,
                        },
                        sheet,
                    );
                    parser::parse_command(
                        &command,
                        &mut self.row_start,
                        &mut self.col_start,
                        &mut self.time,
                        &mut self.status,
                        &sheet1.rows,
                        &sheet1.cols,
                        & mut sheet,
                        &mut true,
                    );
                    let new_cell = sheet.data[row][col].clone();
                    changes.push((row, col, old_cell, new_cell, command));
                }
            }
        }

        if !changes.is_empty() {
            let len = changes.len();
            self.undo_stack.push(Action::FindAndReplace {
                sheet_index: self.current_sheet_index,
                changes,
            });
            self.redo_stack.clear();
            let sorted = sheet_functions::topological_sort(
                &mut std::collections::HashMap::new(),
                sheet,
            );
            for i in sorted {
                let r = i % 1000;
                let c = i / 1000;
                sheet_functions::recalculate(sheet, r as usize, c as usize, &mut self.timer);
            }
            self.status = format!("Replaced {} instances", len);
        } else {
            self.status = "No matches found".to_string();
        }
    }

    pub fn undo(&mut self) {
        if self.undo_stack.is_empty() {
            self.status = "Nothing to undo".to_string();
            return;
        }
        self.clear_clipboard();
        if let Some(action) = self.undo_stack.pop() {
            match action {
                Action::Inserted {sheet_index,row,col,previous_cell} => {
                    utils::insert_undo_redo(sheet_index, row, col, previous_cell,self,false);
                    self.status = "Undone cell edit".to_string();
                }
                Action::CutAction {sheet_index,row1,col1,previous_cell1,row2,col2,previous_cell2} => {
                    utils::cut_undo_redo(sheet_index, row1, col1, previous_cell1, row2, col2, previous_cell2,self,false);
                    self.status = "Undone cut".to_string();
                }
                // Action::Deleted { sheet_index, row, col, deleted_cell } => {
                //     // utils::delete_cell_and_update_dependencies(sheet_index, row, col, deleted_cell, &mut self);
                //     self.status = "Undone delete".to_string();
                // }
                Action::FindAndReplace { sheet_index, changes } => {
                    let mut sheet = &mut self.sheets[sheet_index].sheet;
                    let sheet1 = sheet.clone();
                    let mut redo_changes = Vec::new();
                    for (row, col, old_cell, _, command) in changes {
                        let original_cmd = if old_cell.string.is_some() {
                            format!("{}{}={}", col_num_to_col_name(col as i32), row + 1, old_cell.string.as_ref().unwrap())
                        } else if old_cell.is_error {
                            format!("{}{}=Err", col_num_to_col_name(col as i32), row + 1)
                        } else {
                            format!("{}{}={}", col_num_to_col_name(col as i32), row + 1, old_cell.value)
                        };
                        sheet_functions::remove_dependency(
                            &CellInfo {
                                row: row as i16,
                                col: col as i16,
                            },
                            sheet,
                        );
                        parser::parse_command(
                            &original_cmd,
                            &mut self.row_start,
                            &mut self.col_start,
                            &mut self.time,
                            &mut self.status,
                            &sheet1.rows,
                            &sheet1.cols,
                            &mut sheet,
                            &mut true,
                        );
                        let new_cell_after_undo = sheet.data[row][col].clone();
                        redo_changes.push((row, col, old_cell.clone(), new_cell_after_undo, command));
                    }
                    self.redo_stack.push(Action::FindAndReplace {
                        sheet_index,
                        changes: redo_changes,
                    });
                    let sorted = sheet_functions::topological_sort(
                        &mut std::collections::HashMap::new(),
                        sheet,
                    );
                    for i in sorted {
                        let r = i % 1000;
                        let c = i / 1000;
                        sheet_functions::recalculate(
                            sheet,
                            r as usize,
                            c as usize,
                            &mut self.timer,
                        );
                    }
                    self.status = "Undone find and replace".to_string();
                }
            }
        } else {
            self.status = "Nothing to undo".to_string();
        }
    }

    pub fn redo(&mut self) {
        if self.redo_stack.is_empty() {
            self.status = "Nothing to redo".to_string();
            return;
        }
        self.clear_clipboard();
        if let Some(action) = self.redo_stack.pop() {
            match action {
                Action::Inserted {sheet_index,row,col,previous_cell} => {
                    utils::insert_undo_redo(sheet_index, row, col, previous_cell,self,true);
                    self.status = "Redone cell edit".to_string();
                }
                Action::CutAction {sheet_index,row1,col1,previous_cell1,row2,col2,previous_cell2} => {
                    utils::cut_undo_redo(sheet_index, row1, col1, previous_cell1, row2, col2, previous_cell2,self,true);
                    self.status = "Redone cut".to_string();
                }
                // Action::Deleted { sheet_index, row, col, deleted_cell } => {
                //     // utils::delete_cell_and_update_dependencies(sheet_index, row, col, deleted_cell, &mut self);
                //     self.status = "Redone delete".to_string();
                // }
                Action::FindAndReplace { sheet_index, changes } => {
                    let mut sheet = &mut self.sheets[sheet_index].sheet;
                    let  sheet1 = sheet.clone();
                    let mut undo_changes = Vec::new();
                    for (row, col, _, _, command) in changes {
                        let prev_cell = sheet.data[row][col].clone();
                        sheet_functions::remove_dependency(
                            &CellInfo {
                                row: row as i16,
                                col: col as i16,
                            },
                            sheet,
                        );
                        parser::parse_command(
                            &command,
                            &mut self.row_start,
                            &mut self.col_start,
                            &mut self.time,
                            &mut self.status,
                            &sheet1.rows,
                            &sheet1.cols,
                            &mut sheet,
                            &mut true,
                        );
                        let new_cell_after_redo = sheet.data[row][col].clone();
                        undo_changes.push((row, col, prev_cell, new_cell_after_redo, command));
                    }
                    self.undo_stack.push(Action::FindAndReplace {
                        sheet_index,
                        changes: undo_changes,
                    });
                    let sorted = sheet_functions::topological_sort(
                        &mut std::collections::HashMap::new(),
                        sheet,
                    );
                    for i in sorted {
                        let r = i % 1000;
                        let c = i / 1000;
                        sheet_functions::recalculate(
                            sheet,
                            r as usize,
                            c as usize,
                            &mut self.timer,
                        );
                    }
                    self.status = "Redone find and replace".to_string();
                }
            }
        } else {
            self.status = "Nothing to redo".to_string();
        }
    }

    pub fn clear_clipboard(&mut self) {
        self.clipboard = None;
        self.cut_copied_cell = None;
    }

    pub fn create_new_sheet(&mut self,rows: i32,cols: i32) {
        let new_sheet = Sheet::new(rows, cols);
        let sheet_struct = Sheets {
            sheet: new_sheet.clone(),
            name: self.new_sheet_name.clone(),
        };
        self.sheets.push(sheet_struct.clone());
        let new_index = self.sheets.len() - 1;
        self.current_sheet_index = new_index;
        self.redo_stack.clear();
        self.show_menu = Menu::None;
        self.status = String::from("Sheet created successfully");
        self.new_sheet_name = String::new();
    }
}
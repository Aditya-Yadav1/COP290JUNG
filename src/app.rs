use eframe::{egui, App, Frame};
use crate::{parser, sheet_functions};
use crate::sheet_functions::{Sheet, col_num_to_col_name }; 
use std::string::String;
use crate::ui::app_impl::*;
use crate::ui::menu;
use crate::ui::sheet_display;

impl Default for SpreadsheetApp {
    fn default() -> Self {
        let sheet = Sheet::new(20, 10);
        let sheets = vec![Sheets {
            sheet: sheet.clone(),
            name: String::from("Sheet 1"),
        }];
        SpreadsheetApp::new(sheets)
    }
}

impl App for SpreadsheetApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) { 
        if self.theme.is_light_theme {
            ctx.set_visuals(egui::Visuals::light());
        } else {
            ctx.set_visuals(egui::Visuals::dark());
        }
        
        let visible_rows = 20;
        let visible_cols = 15;
        
        let mut entered = None;

        egui::TopBottomPanel::top("formula_bar").show(ctx, |ui| {
            ui.visuals_mut().override_text_color = Some(self.theme.text_color);
            ui.horizontal(|ui| {
                ui.label("Formula:");
                let resp = ui.text_edit_singleline(&mut self.formula);
                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.clear_clipboard();
                    entered = Some(self.formula.clone());
                    self.formula.clear();
                }
                ui.separator();
                ui.label("Selected Cell:");
                ui.label("oi deepak formula idhar display kara dena");
                ui.separator();
            });
            menu::show_menu(self, ctx, ui);
        });

        sheet_display::show_spreadsheet(self, ctx, &visible_rows, &visible_cols);

        egui::TopBottomPanel::bottom("all_sheets").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Current Sheet : {}", self.sheets[self.current_sheet_index].name));
                for (index, sheet) in self.sheets.iter().enumerate() {
                    ui.separator();
                    let mut button = egui::Button::new(sheet.name.clone());
                    if index == self.current_sheet_index {
                        button = button.fill(self.theme.selected_cell_bg);
                    }
                    if ui.add(button).clicked() {
                        self.current_sheet_index = index;
                    }
                }
                ui.separator();
                if ui.button("New Sheet").clicked() {
                    self.show_menu = Menu::NewSheet;
                }
                if self.show_menu == Menu::NewSheet {
                    egui::Window::new("New Sheet").resizable(false).collapsible(false).show(ctx, |ui| {
                        ui.label("Enter number of rows:"); 
                        ui.label("Enter number of columns:"); 
                        ui.label("Enter sheet name:"); 
                        ui.horizontal(|ui| {
                            if ui.button("Create").clicked() {
                                if self.new_sheet_rows.is_empty() || self.new_sheet_cols.is_empty() {
                                    self.status = String::from("Please enter number of rows and columns");
                                } else {
                                    match (self.new_sheet_rows.parse::<i32>(), self.new_sheet_cols.parse::<i32>()) {
                                        (Ok(rows), Ok(cols)) => {
                                            if rows <= 0 || cols <= 0 || rows > 1000 || cols > 18278 {
                                                self.status = String::from("number of rows and columns not in valid range");
                                            } else {
                                                if self.new_sheet_name.is_empty() {
                                                    self.new_sheet_name = format!("Sheet {}", self.sheets.len() + 1);
                                                }
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
                                        },
                                        _ => {
                                            self.status = String::from("Please enter valid integers for rows and columns");
                                        }
                                    }
                                }
                            }
                            if ui.button("Cancel").clicked() {
                                self.show_menu = Menu::None;
                                self.status = String::from("Ok");
                            }
                        });
                    });
                }
                ui.separator();
                if ui.button("Delete Current Sheet").clicked() {
                    self.undo_stack.clear();
                    self.redo_stack.clear();
                    self.clear_clipboard();
                    if self.sheets.len() > 0 {
                        self.show_menu = Menu::DeleteSheet;
                    } else {
                        self.status = String::from("No sheet to delete:(");
                    }
                }
                if self.show_menu == Menu::DeleteSheet {
                    egui::Window::new("Delete Current Sheet").resizable(false).collapsible(false).show(ctx, |ui| {
                        ui.label("Are you sure you want to delete this sheet?");
                        if ui.button("Delete").clicked() { 
                            self.redo_stack.clear();
                            if self.current_sheet_index >= self.sheets.len() {
                                self.current_sheet_index = self.sheets.len().saturating_sub(1);
                            }
                            self.show_menu = Menu::None;
                            self.status = String::from("Sheet deleted");
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_menu = Menu::None;
                            self.status = String::from("Ok");
                        }
                    });
                }
            });
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.visuals_mut().override_text_color = Some(self.theme.text_color);
            ui.horizontal(|ui| {
                ui.label(format!("Mode: {:?}", self.mode));
                ui.separator();
                ui.label(format!("Status: {}", self.status));
                ui.separator();
                ui.label(format!("Time: {:.1}", self.time));
                ui.separator();
                ui.label(format!("View: Row {} Col {}", self.row_start + 1, col_num_to_col_name(self.col_start)));
                
                if let Some((r, c)) = self.selected_cell {
                    ui.separator();
                    let col_name = col_num_to_col_name(c as i32);
                    let row_str = (r + 1).to_string();
                    ui.label(format!("Selected: {}{}", col_name, row_str));
                }
            });
        });

        let input = ctx.input(|i| i.clone());
        let visible_rows = 20;
        let visible_cols = 15;
        
        if input.modifiers.ctrl && input.key_pressed(egui::Key::Z) {
            self.undo();
            ctx.request_repaint();
        }
        if input.modifiers.ctrl && input.key_pressed(egui::Key::Y) {
            self.redo();
            ctx.request_repaint();
        }
        if input.modifiers.ctrl && input.key_pressed(egui::Key::F) {
            self.show_menu = Menu::FindAndReplace;
            ctx.request_repaint();
        }
        if input.modifiers.ctrl && input.key_pressed(egui::Key::A) {
            menu::copy(self);
            self.status = "Copied".to_string();
            ctx.request_repaint();
        }
        if input.modifiers.ctrl && input.key_pressed(egui::Key::S) {
            menu::paste(self);
            self.status = "Pasted".to_string();
            ctx.request_repaint();
        }
        if input.modifiers.ctrl && input.key_pressed(egui::Key::Q) {
            menu::cut(self);
            self.status = "Cut".to_string();
            ctx.request_repaint();
        }

        if input.key_pressed(egui::Key::I) && self.mode == Mode::Normal {
            self.mode = Mode::Insert;
            self.is_editing = false;
        }

        if input.key_pressed(egui::Key::Escape) {
            self.mode = Mode::Normal;
            self.is_editing = false;
            self.editing_value.clear();
        }

        let mut new_selection = self.selected_cell;
        if let Some((r, c)) = self.selected_cell {
            if input.key_pressed(egui::Key::ArrowUp) && r > 0 { new_selection = Some((r - 1, c)); }
            if input.key_pressed(egui::Key::ArrowDown) && r < self.sheets[self.current_sheet_index].sheet.rows as usize - 1 { new_selection = Some((r + 1, c)); }
            if input.key_pressed(egui::Key::ArrowLeft) && c > 0 { new_selection = Some((r, c - 1)); }
            if input.key_pressed(egui::Key::ArrowRight) && c < self.sheets[self.current_sheet_index].sheet.cols as usize - 1 { new_selection = Some((r, c + 1)); }
        }
        if new_selection != self.selected_cell {
            if self.is_editing {
                self.is_editing = false;
                self.editing_value.clear();
            }
            
            self.selected_cell = new_selection;
            
            if let Some((r, c)) = self.selected_cell {
                if (r as i32) < self.row_start { self.row_start = r as i32;} 
                else if (r as i32) >= self.row_start + visible_rows { self.row_start = (r as i32) - visible_rows + 1;}
                if (c as i32) < self.col_start { self.col_start = c as i32;} 
                else if (c as i32) >= self.col_start + visible_cols { self.col_start = (c as i32) - visible_cols + 1;}
                
                self.row_start = self.row_start.max(0).min(self.sheets[self.current_sheet_index].sheet.rows - visible_rows);
                self.col_start = self.col_start.max(0).min(self.sheets[self.current_sheet_index].sheet.cols - visible_cols);
            }
        }

        if self.mode == Mode::Normal {
            if input.key_pressed(egui::Key::H) {self.col_start = (self.col_start - 1).max(0);}
            if input.key_pressed(egui::Key::L) {self.col_start = (self.col_start + 1).min(self.sheets[self.current_sheet_index].sheet.cols - 1);}
            if input.key_pressed(egui::Key::K) {self.row_start = (self.row_start - 1).max(0);}
            if input.key_pressed(egui::Key::J) {self.row_start = (self.row_start + 1).min(self.sheets[self.current_sheet_index].sheet.rows - 1);}
        }

        if let Some(cmd) = entered {
            let sheet_rows = self.sheets[self.current_sheet_index].sheet.rows;
            let sheet_cols = self.sheets[self.current_sheet_index].sheet.cols;
            let re_cell_edit = regex::Regex::new(r"^([A-Z]+)(\d+)=.*$").unwrap();
            let mut cell_edit_action = None;

            if let Some(caps) = re_cell_edit.captures(&cmd) {
                let col_name = caps.get(1).unwrap().as_str();
                let row_str = caps.get(2).unwrap().as_str();
                if let Ok(row) = row_str.parse::<i32>() {
                    let col = sheet_functions::col_name_to_col_num(col_name);
                    let row = row - 1;
                    if sheet_functions::is_valid_cell(row, col, sheet_rows, sheet_cols) {
                        let old_cell = self.sheets[self.current_sheet_index].sheet.data[row as usize][col as usize].clone();
                        cell_edit_action = Some((row as usize, col as usize, old_cell, cmd.clone()));
                    }
                }
            }

            parser::parse_command(
                &cmd,
                &mut self.row_start,
                &mut self.col_start,
                &mut self.time,
                &mut self.status,
                &sheet_rows,
                &sheet_cols,
                &mut self.sheets[self.current_sheet_index].sheet,
                &mut true,
            );

            if let Some((row, col, old_cell, _)) = cell_edit_action { 
               //ku
                self.undo_stack.push(Action::Inserted {
                    sheet_index: self.current_sheet_index,
                    row: row as i16,
                    col: col as i16,
                    previous_cell: old_cell,
                });
                self.redo_stack.clear();
            }
        }
        
        ctx.request_repaint();
    }
}
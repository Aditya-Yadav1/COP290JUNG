// src/ui.rs

use eframe::{egui, App, Frame};
use serde::{Deserialize, Serialize};
use crate::{parser, sheet_functions};
use crate::sheet_functions::{Sheet, col_num_to_col_name};
use crate::sheet_functions::CellInfo;
use crate::utils::{convert_to_csv, open_csv, save_sheet, load_sheet, save_all_sheets, load_all_sheets};
use std::collections::HashSet;
use eframe::epaint::pos2;
use crate::themes::{themes, Theme};  

#[derive(Debug, PartialEq)]
enum Mode { Normal, Insert }

#[derive(Debug, PartialEq)]
enum Menu{
    Save,
    Open,
    Theme,
    NewSheet,
    DeleteSheet,
    None,
}

#[derive(Deserialize,Serialize)]
pub struct Sheets{
    pub sheet: Sheet,
    pub name : String,
}   

pub struct SpreadsheetApp {
    sheets: Vec<Sheets>,
    current_sheet_index: usize,
    formula: String,
    status: String,
    mode: Mode,
    selected_cell: Option<(usize, usize)>,
    row_start: i32,
    col_start: i32,
    time: f32,
    editing_value: String,  // Store the currently edited value
    is_editing: bool,       // Track if we're actively editing
    show_menu: Menu,
    save_filename: String,
    open_filename: String,
    theme: Theme,
    new_sheet_rows: String,
    new_sheet_cols: String,
    new_sheet_name: String,
}




impl SpreadsheetApp {
    pub fn new(mut sheets: Vec<Sheets>) -> Self {
        Self {
            sheets: sheets,
            current_sheet_index: 0,
            formula: String::new(),
            status: "ok".into(),
            mode: Mode::Normal,
            selected_cell: None,
            row_start: 0,
            col_start: 0,
            time: 0.0,
            editing_value: String::new(),
            is_editing: false,
            show_menu: Menu::None,
            save_filename: String::from("sheet"),
            open_filename: String::new(),
            theme: themes[0].clone(),
            new_sheet_rows: String::new(),
                new_sheet_cols: String::new(),
            new_sheet_name: String::new(),
        }
    }
}

impl Default for SpreadsheetApp {
    fn default() -> Self {
        let sheet = Sheet::new(20, 10);
        let sheets = vec![Sheets{sheet:sheet.clone(), name:String::from("Sheet 1")}];
        Self::new(sheets)
    }
}

impl App for SpreadsheetApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) { 
        if self.theme.is_light_theme {
            ctx.set_visuals(egui::Visuals::light());   // light theme
        } else {
            ctx.set_visuals(egui::Visuals::dark());   // dark theme
        }
        
        // defining visible rows and colums
        let visible_rows = 20;
        let visible_cols = 15;
        
        // 1-> formula bar
        let mut entered = None;
        

        egui::TopBottomPanel::top("formula_bar").show(ctx, |ui| {
            ui.visuals_mut().override_text_color = Some(self.theme.text_color);
            ui.horizontal(|ui| {
                ui.label("Formula:");
                let resp = ui.text_edit_singleline(&mut self.formula);
                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    entered = Some(self.formula.clone());
                    self.formula.clear();
                }
            });
            egui::CollapsingHeader::new("Menu")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                if self.show_menu == Menu::None && ui.button("Save").clicked() {
                    self.show_menu = Menu::Save;
                }
                if self.show_menu == Menu::Save {
                    egui::Window::new("Save").resizable(false).collapsible(false).movable(false).show(ctx, |ui| {
                    ui.label("Enter filename:");
                    let resp = ui.text_edit_singleline(&mut self.save_filename);
                    if resp.lost_focus()  {
                        self.show_menu = Menu::None;
                    }
                    ui.horizontal(|ui|{
                    if ui.button("Save File").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if self.save_filename.is_empty() {
                            self.status = "Please enter a filename".to_string();
                        }else{
                            let save_file_name = format!("{}.290", self.save_filename);
                            save_all_sheets(&self.sheets, &save_file_name);
                            self.status = "Saved".to_string();
                            self.show_menu = Menu::None;
                        }
                    }
                    if ui.button("Save to CSV").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if self.save_filename.is_empty() {
                            self.status = "Please enter a filename".to_string();
                        }else{
                            convert_to_csv(&self.sheets[self.current_sheet_index].sheet, &self.save_filename);
                            self.status = "Saved".to_string();
                            self.show_menu = Menu::None;
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        self.show_menu = Menu::None;
                    }});
                    });

                }

                if self.show_menu == Menu::None && ui.button("Open").clicked() {
                    self.show_menu = Menu::Open;
                }
                if self.show_menu == Menu::Open {
                    egui::Window::new("Open").resizable(false).collapsible(false).movable(false).show(ctx, |ui| {
                    ui.label("Enter filename:");    
                    let resp = ui.text_edit_singleline(&mut self.open_filename);
                    if resp.lost_focus()  {
                        self.show_menu = Menu::None;
                    }
                    ui.horizontal(|ui|{
                    if ui.button("Open").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if self.open_filename.is_empty() {
                            self.status = "Please enter a filename".to_string();
                        }else if self.open_filename.ends_with(".csv") {
                            self.show_menu = Menu::None;    
                            self.status = open_csv(&self.open_filename, &mut self.sheets[self.current_sheet_index].sheet);
                        }
                        else if self.open_filename.ends_with(".290") {
                            self.show_menu = Menu::None;
                            self.sheets = load_all_sheets(&self.open_filename);
                            self.current_sheet_index = 0;
                            self.status = "Loaded".to_string();
                        }
                        else {
                            self.status = "Please enter a valid filename".to_string();
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        self.show_menu = Menu::None;
                    }});
                    });

                }

                if ui.button("Clear").clicked() {
                    for row in &mut self.sheets[self.current_sheet_index].sheet.data {
                        for cell in row {
                            cell.value = 0;
                            cell.string = None;
                            cell.is_error = false;
                            cell.op_code = 'X';
                            cell.cell1 = CellInfo { row: -1, col: -1 };
                            cell.cell2 = CellInfo { row: -1, col: -1 };
                            cell.dependencies = HashSet::new();
                        }
                    }
                    self.status = "Cleared".to_string();
                };
                
                if ui.button("Theme").clicked() {
                    self.show_menu = Menu::Theme;
                }
                if self.show_menu == Menu::Theme {
                    egui::Window::new("Theme").resizable(false).collapsible(false).show(ctx, |ui| {
                        ui.label("Select theme:");
                        for theme in &themes {
                            if ui.button(theme.name).clicked() {
                                self.theme = theme.clone();
                                self.show_menu = Menu::None;
                            }
                        }
                    });
                }
            })
            });
        });

        // 2) Central panel with sticky headers
        egui::CentralPanel::default().show(ctx, |ui| {
            let cell_size = egui::vec2(120.0, 30.0);
            
            // before redering checking mouse scroll
            let scroll_delta = ui.input(|i| i.smooth_scroll_delta);
            if scroll_delta.y != 0.0 {
                // vectical scroll with mouse
                let scroll_rows = (scroll_delta.y / 30.0).round() as i32;
                self.row_start = (self.row_start - scroll_rows).max(0).min(self.sheets[self.current_sheet_index].sheet.rows - 1);
            }
            if scroll_delta.x != 0.0 {
                // horizontal scroll with mouse  (Shift+scroll)
                let scroll_cols = (scroll_delta.x / 30.0).round() as i32;
                self.col_start = (self.col_start - scroll_cols).max(0).min(self.sheets[self.current_sheet_index].sheet.cols - 1);
            }
            
            // creating grid with headers
            egui::Grid::new("header_grid")
                .min_col_width(cell_size.x)
                .min_row_height(cell_size.y)
                .spacing([0.0, 0.0]) // No spacing for perfect alignment
                .show(ui, |ui| {
                    // Corner cell
                    let corner_rect = ui.available_rect_before_wrap();
                    ui.painter().rect_filled(
                        corner_rect,
                        0.0,
                        self.theme.header_bg
                    );
                    ui.painter().rect_stroke(
                        corner_rect,
                        0.0,
                        egui::Stroke::new(1.0, self.theme.grid_line_color)
                    );
                    ui.add_sized(
                        cell_size,
                        egui::Label::new(
                            egui::RichText::new("")
                                .color(self.theme.header_text_color)
                                .text_style(egui::TextStyle::Heading)
                        )
                    );
                    
                    // Column headers with exact same width as cells below
                    for c in self.col_start..(self.col_start + visible_cols).min(self.sheets[self.current_sheet_index].sheet.cols) {
                        let header_rect = ui.available_rect_before_wrap();
                        ui.painter().rect_filled(
                            header_rect,
                            0.0,
                            self.theme.header_bg
                        );
                        ui.painter().rect_stroke(
                            header_rect,
                            0.0,
                            egui::Stroke::new(1.0, self.theme.grid_line_color)
                        );
                        
                        // Centring column header text
                        ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                            ui.add_sized(
                                cell_size,
                                egui::Label::new(
                                    egui::RichText::new(col_num_to_col_name(c))
                                        .strong()
                                        .color(self.theme.header_text_color)
                                        .text_style(egui::TextStyle::Heading)
                                )
                            );
                        });
                    }
                    ui.end_row();
                });
            
            // rendering scrollable grid below headers
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    egui::Grid::new("sheet_grid")
                        .min_col_width(cell_size.x)
                        .min_row_height(cell_size.y)
                        .spacing([0.0, 0.0]) // No spacing for perfect alignment
                        .show(ui, |ui| {
                            // Data rows
                            for r in self.row_start..(self.row_start + visible_rows).min(self.sheets[self.current_sheet_index].sheet.rows) {
                                // Row headers with proper styling
                                let row_header_rect = ui.available_rect_before_wrap();
                                ui.painter().rect_filled(
                                    row_header_rect,
                                    0.0,
                                    self.theme.header_bg
                                );
                                ui.painter().rect_stroke(
                                    row_header_rect,
                                    0.0,
                                    egui::Stroke::new(1.0, self.theme.grid_line_color)
                                );
                                
                                // Center the row number
                                ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                                    ui.add_sized(
                                        cell_size,
                                        egui::Label::new(
                                            egui::RichText::new((r + 1).to_string())
                                                .strong()
                                                .color(self.theme.header_text_color)
                                                .text_style(egui::TextStyle::Heading)
                                        )
                                    );
                                });
                                
                                // Data cells
                                for c in self.col_start..(self.col_start + visible_cols).min(self.sheets[self.current_sheet_index].sheet.cols) {
                                    let cell = &mut self.sheets[self.current_sheet_index].sheet.data[r as usize][c as usize];
                                    let display = if cell.string.is_some() {
                                        cell.string.as_ref().unwrap().clone()
                                    } else if cell.is_error {
                                        "Err".to_string()
                                    } else {
                                        cell.value.to_string()
                                    };
                                    
                                    let is_sel = self.selected_cell == Some((r as usize, c as usize));
                                    
                                    // Draw cell background with border for grid-like appearance
                                    let rect = ui.available_rect_before_wrap();
                                    
                                    // Cell background
                                    ui.painter().rect_filled(
                                        rect,
                                        0.0,
                                        if is_sel { self.theme.selected_cell_bg } else { self.theme.cell_bg }
                                    );
                                    
                                    // Cell border
                                    ui.painter().rect_stroke(
                                        rect,
                                        0.0,
                                        egui::Stroke::new(1.0, self.theme.grid_line_color)
                                    );
                                    
                                    if is_sel && self.mode == Mode::Insert {
                                        // Initialize editing value if we just started editing
                                        if !self.is_editing {
                                            self.editing_value = display.clone();
                                            self.is_editing = true;
                                        }
                                        
                                        // Editable cell
                                        let edit = ui.add_sized(
                                            cell_size,
                                            egui::TextEdit::singleline(&mut self.editing_value)
                                                .frame(false) // No additional frame
                                                .desired_width(cell_size.x)
                                                .text_color(self.theme.text_color)
                                                .cursor_at_end(true)
                                        );
                                        
                                        // Handle Enter key to commit changes
                                        if edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                            // 1) build command "A2=value"
                                            let col_name = col_num_to_col_name(c);
                                            let row_str = (r + 1).to_string();
                                            let cmd = format!("{}{}={}", col_name, row_str, self.editing_value);
                                            
                                            // 2) call parser to recalc
                                            let sheet_rows = self.sheets[self.current_sheet_index].sheet.rows;
                                            let sheet_cols = self.sheets[self.current_sheet_index].sheet.cols;
                                            parser::parse_command(
                                                &cmd,
                                                &mut self.row_start,
                                                &mut self.col_start,
                                                &mut self.time,
                                                &mut self.status,
                                                &sheet_rows,
                                                &sheet_cols,
                                                &mut self.sheets[self.current_sheet_index].sheet,
                                            );
                                            
                                            // 3) reset editing state
                                            // self.mode = Mode::Normal;
                                            self.is_editing = false;
                                            self.editing_value.clear();
                                        }
                                        
                                        // use esc to go to normal mode
                                        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                                            self.mode = Mode::Normal;
                                            self.is_editing = false;
                                            self.editing_value.clear();
                                        }
                                        
                                        // if outside clicks current cell -> cancel editing
                                        if ui.input(|i| i.pointer.any_click()) {
                                            let pointer_pos = ui.input(|i| i.pointer.interact_pos());
                                            if let Some(pos) = pointer_pos {
                                                if !rect.contains(pos) { 
                                                    // self.mode = Mode::Normal;
                                                    self.is_editing = false;
                                                    self.editing_value.clear();
                                                }
                                            }
                                        }
                                    } else {
                                        // Non-editable cell with click detection
                                        let response = ui.add_sized(
                                            cell_size,
                                            egui::Label::new(
                                                egui::RichText::new(display)
                                                    .color(self.theme.text_color)
                                            ).sense(egui::Sense::click()),
                                        );
                                        
                                        if response.clicked() {
                                            // If we were editing another cell, discard those changes
                                            if self.is_editing {
                                                self.is_editing = false;
                                                self.editing_value.clear();
                                            }
                                            
                                            self.selected_cell = Some((r as usize, c as usize));
                                            
                                            // Double click enter edit mode
                                            if response.clicked_by(egui::PointerButton::Primary) && 
                                               response.double_clicked() {
                                                self.mode = Mode::Insert;
                                            }
                                        }
                                    }
                                }
                                ui.end_row();
                            }
                        });
                });
            
            // Add horizontal scrollbar for better UX
            ui.horizontal(|ui| {
                if ui.button("◀").clicked() {
                    self.col_start = (self.col_start - 1).max(0);
                }
                
                let progress = self.col_start as f32 / (self.sheets[self.current_sheet_index].sheet.cols - visible_cols).max(1) as f32;
                let mut progress_bar = progress;
                if ui.add(egui::Slider::new(&mut progress_bar, 0.0..=1.0).text("Scroll")).changed() {
                    self.col_start = ((self.sheets[self.current_sheet_index].sheet.cols - visible_cols).max(1) as f32 * progress_bar) as i32;
                    self.col_start = self.col_start.max(0).min(self.sheets[self.current_sheet_index].sheet.cols - 1);
                }
                
                if ui.button("▶").clicked() {
                    self.col_start = (self.col_start + 1).min((self.sheets[self.current_sheet_index].sheet.cols - visible_cols).max(0));
                }
            });
        });

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
                        let rows = ui.text_edit_singleline(&mut self.new_sheet_rows);
                        ui.label("Enter number of columns:");
                        let cols = ui.text_edit_singleline(&mut self.new_sheet_cols);
                        ui.label("Enter sheet name:");
                        let name = ui.text_edit_singleline(&mut self.new_sheet_name);
                        ui.horizontal(|ui| {
                        if ui.button("Create").clicked() {
                            if self.new_sheet_rows.is_empty() || self.new_sheet_cols.is_empty() {
                                self.status = String::from("Please enter number of rows and columns");
                            }
                            else {
                                match (self.new_sheet_rows.parse::<i32>(), self.new_sheet_cols.parse::<i32>()) {
                                    (Ok(rows), Ok(cols)) => {
                                        if rows <= 0 || cols <= 0 || rows >1000 || cols > 18278 {
                                            self.status = String::from("number of rows and columns not in valid range");
                                        } else {
                                            if self.new_sheet_name.is_empty() {
                                                self.new_sheet_name = format!("Sheet {}", self.sheets.len() + 1);
                                            }
                                            let new_sheet = Sheet::new(rows, cols);
                                            self.sheets.push(Sheets{sheet:new_sheet.clone(), name:self.new_sheet_name.clone()});
                                            self.current_sheet_index = self.sheets.len() - 1;
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
                            self.status = String :: from("Ok");
                        }
                        });
                    });
                }
                ui.separator();
                if ui.button("Delete Current Sheet").clicked() {
                    if self.sheets.len() > 0 {
                        self.show_menu = Menu::DeleteSheet;
                    }
                    else{
                        self.status = String::from("No sheet to delete:(");
                    }
                }
                if self.show_menu == Menu::DeleteSheet {
                    egui::Window::new("Delete Current Sheet").resizable(false).collapsible(false).show(ctx, |ui| {
                        ui.label("Are you sure you want to delete this sheet?");
                        if ui.button("Delete").clicked() {
                            self.sheets.remove(self.current_sheet_index);
                            if self.current_sheet_index >= self.sheets.len() {
                                self.current_sheet_index = self.sheets.len() - 1;
                            }
                            self.show_menu = Menu::None;
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

        // 4) Keyboard handling for modes & scrolling
        let input = ctx.input(|i| i.clone());
        let visible_rows = 20;
        let visible_cols = 15;
        
        // Mode switching
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
        // If selection changed due to arrow keys
        if new_selection != self.selected_cell {
            // If we were editing-> discard changes
            if self.is_editing {
                self.is_editing = false;
                self.editing_value.clear();
            }
            
            // Update the selection
            self.selected_cell = new_selection;
            
            // scrolling with selected cell
            if let Some((r, c)) = self.selected_cell {
                // checking selected cell out of screen 
                if (r as i32) < self.row_start { self.row_start = r as i32;} 
                else if (r as i32) >= self.row_start + visible_rows { self.row_start = (r as i32) - visible_rows + 1;}
                if (c as i32) < self.col_start { self.col_start = c as i32;} 
                else if (c as i32) >= self.col_start + visible_cols { self.col_start = (c as i32) - visible_cols + 1;}
                
                self.row_start = self.row_start.max(0).min(self.sheets[self.current_sheet_index].sheet.rows - visible_rows);
                self.col_start = self.col_start.max(0).min(self.sheets[self.current_sheet_index].sheet.cols - visible_cols);
            }
        }

        // Vim-style scroll navigation
        if self.mode == Mode::Normal {
            if input.key_pressed(egui::Key::H) {self.col_start = (self.col_start - 1).max(0);}
            if input.key_pressed(egui::Key::L) {self.col_start = (self.col_start + 1).min(self.sheets[self.current_sheet_index].sheet.cols - 1);}
            if input.key_pressed(egui::Key::K) {self.row_start = (self.row_start - 1).max(0);}
            if input.key_pressed(egui::Key::J) {self.row_start = (self.row_start + 1).min(self.sheets[self.current_sheet_index].sheet.rows - 1);}
        }

        // parsing the command from formula bar if entered
        if let Some(cmd) = entered {
            let sheet_rows = self.sheets[self.current_sheet_index].sheet.rows;
            let sheet_cols = self.sheets[self.current_sheet_index].sheet.cols;
            parser::parse_command(
                &cmd,
                &mut self.row_start,
                &mut self.col_start,
                &mut self.time,
                &mut self.status,
                &sheet_rows,
                &sheet_cols,
                &mut self.sheets[self.current_sheet_index].sheet,
            );
        }
        
        // continuous repainting to ensure UI stays responsive
        ctx.request_repaint();
    }
}
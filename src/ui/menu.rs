use crate::ui::app_impl::{Menu,Action,SpreadsheetApp};
use crate::parser;
use crate::ui::themes::themes;
use crate::sheet_functions::{self,CellInfo,col_num_to_col_name,OpCode};
use crate::sheet_functions::OpCode::*;
use std::collections::HashSet;
use crate::ui::utils::{load_all_sheets,save_all_sheets,convert_to_csv,open_csv};

pub fn show_menu(app: &mut SpreadsheetApp, ctx: &egui::Context, ui: &mut egui::Ui)->egui::CollapsingResponse<()>{
    egui::CollapsingHeader::new("Menu")
    .default_open(true)
    .show(ui, |ui| {
        ui.horizontal(|ui| {
            if app.show_menu == Menu::None && ui.button("Save").clicked() {
                app.show_menu = Menu::Save;
            }
            if app.show_menu == Menu::Save {
                egui::Window::new("Save").resizable(false).collapsible(false).movable(false).show(ctx, |ui| {
                    ui.label("Enter filename:");
                    let resp = ui.text_edit_singleline(&mut app.save_filename);
                    if resp.lost_focus() {
                        app.show_menu = Menu::None;
                    }
                    ui.horizontal(|ui| {
                        if ui.button("Save File").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            if app.save_filename.is_empty() {
                                app.status = "Please enter a filename".to_string();
                            } else {
                                let save_file_name = format!("{}.290", app.save_filename);
                                save_all_sheets(&app.sheets, &save_file_name);
                                app.status = "Saved".to_string();
                                app.show_menu = Menu::None;
                            }
                        }
                        if ui.button("Save to CSV").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            if app.save_filename.is_empty() {
                                app.status = "Please enter a filename".to_string();
                            } else {
                                convert_to_csv(&app.sheets[app.current_sheet_index].sheet, &app.save_filename);
                                app.status = "Saved".to_string();
                                app.show_menu = Menu::None;
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            app.show_menu = Menu::None;
                        }
                    });
                });
            }

            if app.show_menu == Menu::None && ui.button("Open").clicked() {
                app.show_menu = Menu::Open;
            }
            if app.show_menu == Menu::Open {
                egui::Window::new("Open").resizable(false).collapsible(false).movable(false).show(ctx, |ui| {
                    ui.label("Enter filename:");    
                    let resp = ui.text_edit_singleline(&mut app.open_filename);
                    if resp.lost_focus() {
                        app.show_menu = Menu::None;
                    }
                    ui.horizontal(|ui| {
                        if ui.button("Open").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            if app.open_filename.is_empty() {
                                app.status = "Please enter a filename".to_string();
                            } else if app.open_filename.ends_with(".csv") {
                                app.show_menu = Menu::None;    
                                app.status = open_csv(&app.open_filename, &mut app.sheets[app.current_sheet_index].sheet);
                            } else if app.open_filename.ends_with(".290") {
                                app.show_menu = Menu::None;
                                app.sheets = load_all_sheets(&app.open_filename);
                                app.current_sheet_index = 0;
                                app.status = "Loaded".to_string();
                            } else {
                                app.status = "Please enter a valid filename".to_string();
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            app.show_menu = Menu::None;
                        }
                    });
                });
            }

            if app.show_menu == Menu::None && ui.button("Find and Replace").clicked() {
                app.show_menu = Menu::FindAndReplace;
            }
            if app.show_menu == Menu::FindAndReplace {
                egui::Window::new("Find and Replace").resizable(false).collapsible(false).show(ctx, |ui| {
                    ui.label("Find:");
                    ui.text_edit_singleline(&mut app.find_text);
                    ui.label("Replace with:");
                    ui.text_edit_singleline(&mut app.replace_text);
                    ui.horizontal(|ui| {
                        if ui.button("Replace All").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            if app.find_text.is_empty() {
                                app.status = "Please enter text to find".to_string();
                            } else {
                                app.find_and_replace();
                                app.show_menu = Menu::None;
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            app.show_menu = Menu::None;
                            app.find_text.clear();
                            app.replace_text.clear();
                        }
                    });
                });
            }

            if ui.button("Clear").clicked() {
                let old_data = app.sheets[app.current_sheet_index].sheet.data.clone();
                for row in &mut app.sheets[app.current_sheet_index].sheet.data {
                    for cell in row {
                        cell.value = 0;
                        cell.string = None;
                        cell.is_error = false;
                        cell.op_code = NoConstraint;
                        cell.cell1 = CellInfo { row: -1, col: -1 };
                        cell.cell2 = CellInfo { row: -1, col: -1 };
                        cell.dependencies = HashSet::new();
                    }
                }
                app.undo_stack.push(Action::ClearSheet {
                    sheet_index: app.current_sheet_index,
                    old_data,
                });
                app.redo_stack.clear();
                app.status = "Cleared".to_string();
            }

            if ui.button("Cut").clicked() {
                if let Some((row, col)) = app.selected_cell {
                    let sheet_rows = app.sheets[app.current_sheet_index].sheet.rows;
                    let sheet_cols = app.sheets[app.current_sheet_index].sheet.cols;
                    let old_cell = app.sheets[app.current_sheet_index].sheet.data[row][col].clone();
                    let command = if old_cell.string.is_some() {
                        format!("{}{}={}", col_num_to_col_name(col as i32), row + 1, old_cell.string.as_ref().unwrap())
                    } else if old_cell.is_error {
                        format!("{}{}=Err", col_num_to_col_name(col as i32), row + 1)
                    } else {
                        format!("{}{}={}", col_num_to_col_name(col as i32), row + 1, old_cell.value)
                    };
                    app.clipboard = Some((old_cell.clone(), command, row, col));
                    sheet_functions::remove_dependency(
                        &CellInfo {
                            row: row as i16,
                            col: col as i16,
                        },
                        &mut app.sheets[app.current_sheet_index].sheet,
                    );
                    let clear_cmd = format!("{}{}=0", col_num_to_col_name(col as i32), row + 1);
                    parser::parse_command(
                        &clear_cmd,
                        &mut app.row_start,
                        &mut app.col_start,
                        &mut app.time,
                        &mut app.status,
                        &sheet_rows,
                        &sheet_cols,
                        &mut app.sheets[app.current_sheet_index].sheet,
                        &mut true,
                    );
                    app.undo_stack.push(Action::Cut {
                        sheet_index: app.current_sheet_index,
                        row,
                        col,
                        old_cell,
                    });
                    app.redo_stack.clear();
                    let sorted = sheet_functions::topological_sort(
                        &mut std::collections::HashMap::new(),
                        &app.sheets[app.current_sheet_index].sheet,
                    );
                    for i in sorted {
                        let r = i % 1000;
                        let c = i / 1000;
                        sheet_functions::recalculate(
                            &mut app.sheets[app.current_sheet_index].sheet,
                            r as usize,
                            c as usize,
                            &mut app.timer,
                        );
                    }
                    app.status = "Cut".to_string();
                } else {
                    app.status = "No cell selected".to_string();
                }
            }

            if ui.button("Copy").clicked() {
                if let Some((row, col)) = app.selected_cell {
                    let cell = app.sheets[app.current_sheet_index].sheet.data[row][col].clone();
                    let command = if cell.string.is_some() {
                        format!("{}{}={}", col_num_to_col_name(col as i32), row + 1, cell.string.as_ref().unwrap())
                    } else if cell.is_error {
                        format!("{}{}=Err", col_num_to_col_name(col as i32), row + 1)
                    } else {
                        format!("{}{}={}", col_num_to_col_name(col as i32), row + 1, cell.value)
                    };
                    app.clipboard = Some((cell, command, row, col));
                    app.status = "Copied".to_string();
                } else {
                    app.status = "No cell selected".to_string();
                }
            }

            if ui.button("Paste").clicked() {
                if let Some((row, col)) = app.selected_cell {
                    if let Some((cell, command, src_row, src_col)) = app.clipboard.clone() {
                        let sheet_rows = app.sheets[app.current_sheet_index].sheet.rows;
                        let sheet_cols = app.sheets[app.current_sheet_index].sheet.cols;
                        let old_cell = app.sheets[app.current_sheet_index].sheet.data[row][col].clone();
                        sheet_functions::remove_dependency(
                            &CellInfo {
                                row: row as i16,
                                col: col as i16,
                            },
                            &mut app.sheets[app.current_sheet_index].sheet,
                        );
                        let content = if let Some(s) = &cell.string {
                            s.clone()
                        } else if cell.is_error {
                            "Err".to_string()
                        } else {
                            cell.value.to_string()
                        };
                        let paste_cmd = format!("{}{}={}", col_num_to_col_name(col as i32), row + 1, content);
                        parser::parse_command(
                            &paste_cmd,
                            &mut app.row_start,
                            &mut app.col_start,
                            &mut app.time,
                            &mut app.status,
                            &sheet_rows,
                            &sheet_cols,
                            &mut app.sheets[app.current_sheet_index].sheet,
                            &mut true,
                        );
                        let new_cell = app.sheets[app.current_sheet_index].sheet.data[row][col].clone();
                        app.undo_stack.push(Action::Paste {
                            sheet_index: app.current_sheet_index,
                            row,
                            col,
                            old_cell,
                            new_cell,
                            command: paste_cmd,
                        });
                        app.redo_stack.clear();
                        let sorted = sheet_functions::topological_sort(
                            &mut std::collections::HashMap::new(),
                            &app.sheets[app.current_sheet_index].sheet,
                        );
                        for i in sorted {
                            let r = i % 1000;
                            let c = i / 1000;
                            sheet_functions::recalculate(
                                &mut app.sheets[app.current_sheet_index].sheet,
                                r as usize,
                                c as usize,
                                &mut app.timer,
                            );
                        }
                        app.status = "Pasted".to_string();
                    } else {
                        app.status = "Nothing to paste".to_string();
                    }
                } else {
                    app.status = "No cell selected".to_string();
                }
            }

            if ui.button("Undo").clicked() {
                app.undo();
                ctx.request_repaint();
            }

            if ui.button("Redo").clicked() {
                app.redo();
                ctx.request_repaint();
            }

            if ui.button("Theme").clicked() {
                app.show_menu = Menu::Theme;
            }
            if app.show_menu == Menu::Theme {
                egui::Window::new("Theme").resizable(false).collapsible(false).show(ctx, |ui| {
                    ui.label("Select theme:");
                    for theme in &themes {
                        if ui.button(theme.name).clicked() {
                            app.theme = theme.clone();
                            app.show_menu = Menu::None;
                        }
                    }
                });
            }
        });
    })
}
// use crate::ui::app_impl::{Menu,Action,SpreadsheetApp,CutCopy};
// use crate::ui::themes::THEMES;
// use crate::sheet_functions::{self,CellInfo};
// use crate::sheet_functions::OpCode::*;
// use std::collections::HashSet;
// use crate::ui::utils::{load_all_sheets,save_all_sheets,convert_to_csv,open_csv};

// pub fn cut(app: &mut SpreadsheetApp){
//     if let Some((row, col)) = app.selected_cell {
//         let old_cell = app.sheets[app.current_sheet_index].sheet.data[row][col].clone();
//         app.clipboard = Some((old_cell, CutCopy::Cut, row as i16, col as i16));
//         app.cut_copied_cell = Some((row as i16, col as i16));
//         app.status = "Cut".to_string();
//     } else {
//         app.status = "No cell selected".to_string();
//     }
// }

// pub fn copy(app: &mut SpreadsheetApp){
//     if let Some((row, col)) = app.selected_cell {
//         let mut cell = app.sheets[app.current_sheet_index].sheet.data[row][col].clone();
//         cell.dependencies = HashSet::new();
//         cell.cell1 = CellInfo { row: -1, col: -1 };
//         cell.cell2 = CellInfo { row: -1, col: -1 };
//         cell.op_code = NoConstraint;
//         app.clipboard = Some((cell, CutCopy::Copied, row as i16, col as i16));
//         app.cut_copied_cell = Some((row as i16, col as i16));
//         app.status = "Copied".to_string();
//     } else {
//         app.status = "No cell selected".to_string();
//     }
// }

// pub fn paste(app: &mut SpreadsheetApp){
//     if let Some((row, col)) = app.selected_cell {
//         if let Some((cell, cut_copy, old_row, old_col)) = app.clipboard.clone() { 
//             let old_cell1 = app.sheets[app.current_sheet_index].sheet.data[old_row as usize][old_col as usize].clone();
//             let old_cell2 = app.sheets[app.current_sheet_index].sheet.data[row as usize][col as usize].clone();

//             if cut_copy == CutCopy::Cut {
//                 let mut new_cell1=old_cell1.clone();
//                 new_cell1.value = 0;
//                 new_cell1.string = None;
//                 new_cell1.is_error = false;
//                 new_cell1.op_code = NoConstraint;
//                 new_cell1.cell1 = CellInfo { row: -1, col: -1 };
//                 new_cell1.cell2 = CellInfo { row: -1, col: -1 };
//                 let depended_cell1=app.sheets[app.current_sheet_index].sheet.data[old_row as usize][old_col as usize].cell1.clone();
//                 let depended_cell2=app.sheets[app.current_sheet_index].sheet.data[old_row as usize][old_col as usize].cell2.clone();
                
//                 if depended_cell1.row != -1 && depended_cell1.col != -1 {
//                     app.sheets[app.current_sheet_index].sheet.data[depended_cell1.row as usize][depended_cell1.col as usize].dependencies.remove(&(old_col as i32 * 1000 + old_row as i32));
//                     app.sheets[app.current_sheet_index].sheet.data[depended_cell1.row as usize][depended_cell1.col as usize].dependencies.insert(col as i32 * 1000 + row as i32);
//                 }
//                 if depended_cell2.row != -1 && depended_cell2.col != -1 {
//                     app.sheets[app.current_sheet_index].sheet.data[depended_cell2.row as usize][depended_cell2.col as usize].dependencies.remove(&(old_col as i32 * 1000 + old_row as i32));
//                     app.sheets[app.current_sheet_index].sheet.data[depended_cell2.row as usize][depended_cell2.col as usize].dependencies.insert(col as i32 * 1000 + row as i32);
//                 }

//                 sheet_functions::remove_dependency(&CellInfo { row: row as i16, col: col as i16 }, &mut app.sheets[app.current_sheet_index].sheet);
                
                
//                 app.sheets[app.current_sheet_index].sheet.data[old_row as usize][old_col as usize] = new_cell1.clone();
//                 sheet_functions::change_dependecy_set(&mut new_cell1, &mut app.sheets[app.current_sheet_index].sheet, true);
//                 let mut new_cell2=old_cell1.clone();
//                 sheet_functions::change_dependecy_set(&mut new_cell2, &mut app.sheets[app.current_sheet_index].sheet, false);
//                 sheet_functions::update_dependencies(old_row, old_col, row as i16, col as i16, &mut app.sheets[app.current_sheet_index].sheet);
//                 for dependency in old_cell2.dependencies.clone() {
//                     let dependency_row = dependency % 1000;
//                     let dependency_col = dependency / 1000;
//                     app.sheets[app.current_sheet_index].sheet.data[dependency_row as usize][dependency_col as usize].value=0;
//                     app.sheets[app.current_sheet_index].sheet.data[dependency_row as usize][dependency_col as usize].is_error=true;
//                     app.sheets[app.current_sheet_index].sheet.data[dependency_row as usize][dependency_col as usize].string=None;
//                     app.sheets[app.current_sheet_index].sheet.data[dependency_row as usize][dependency_col as usize].op_code=NoConstraint;
//                     app.sheets[app.current_sheet_index].sheet.data[dependency_row as usize][dependency_col as usize].cell1 = CellInfo { row: -1, col: -1 };
//                     app.sheets[app.current_sheet_index].sheet.data[dependency_row as usize][dependency_col as usize].cell2 = CellInfo { row: -1, col: -1 };
//                     sheet_functions::remove_dependency(&CellInfo { row: dependency_row as i16, col: dependency_col as i16 }, &mut app.sheets[app.current_sheet_index].sheet);
//                 }
//                 app.sheets[app.current_sheet_index].sheet.data[row as usize][col as usize] = new_cell2;
//                 app.redo_stack.clear();
//                 app.undo_stack.push(Action::CutAction {
//                     sheet_index: app.current_sheet_index,
//                     row1: old_row as i16,
//                     col1: old_col as i16,
//                     previous_cell1: old_cell1.clone(),
//                     row2: row as i16,
//                     col2: col as i16,
//                     previous_cell2: old_cell2.clone(),
//                 });
//                 app.clipboard = None;
//             }
//             if cut_copy == CutCopy::Copied {
//                 let mut new_cell=cell.clone();
//                 new_cell.dependencies = old_cell2.dependencies.clone();
//                 app.sheets[app.current_sheet_index].sheet.data[row][col] = new_cell.clone();
//                 sheet_functions::recalculate_dependecy(CellInfo{ row: row as i16, col: col as i16 }, &mut app.sheets[app.current_sheet_index].sheet);
//                 app.redo_stack.clear();
//                 app.undo_stack.push(Action::Inserted{
//                     sheet_index: app.current_sheet_index,
//                     row: row as i16,
//                     col: col as i16,
//                     previous_cell: old_cell2.clone(),
//                 });
//             }
//             app.cut_copied_cell = None;
//             app.status = "Pasted".to_string();
//         } else {
//             app.status = "Nothing to paste".to_string();
//         }
//     } else {
//         app.status = "No cell selected".to_string();
//     }
// }

// pub fn show_menu(mut app: &mut SpreadsheetApp, ctx: &egui::Context, ui: &mut egui::Ui)->egui::CollapsingResponse<()>{
//     egui::CollapsingHeader::new("Menu")
//     .default_open(true)
//     .show(ui, |ui| {
//         ui.horizontal(|ui| {
//             if app.show_menu == Menu::None && ui.button("Save").clicked() {
//                 app.show_menu = Menu::Save;
//             }
//             if app.show_menu == Menu::Save {
//                 egui::Window::new("Save").resizable(false).collapsible(false).movable(false).show(ctx, |ui| {
//                     ui.label("Enter filename:");
//                     let resp = ui.text_edit_singleline(&mut app.save_filename);
//                     if resp.lost_focus() {
//                         app.show_menu = Menu::None;
//                     }
//                     ui.horizontal(|ui| {
//                         if ui.button("Save File").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
//                             if app.save_filename.is_empty() {
//                                 app.status = "Please enter a filename".to_string();
//                             } else {
//                                 let save_file_name = format!("{}.290", app.save_filename);
//                                 save_all_sheets(&app.sheets, &save_file_name);
//                                 app.status = "Saved".to_string();
//                                 app.show_menu = Menu::None;
//                             }
//                         }
//                         if ui.button("Save to CSV").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
//                             if app.save_filename.is_empty() {
//                                 app.status = "Please enter a filename".to_string();
//                             } else {
//                                 convert_to_csv(&app.sheets[app.current_sheet_index].sheet, &app.save_filename);
//                                 app.status = "Saved".to_string();
//                                 app.show_menu = Menu::None;
//                             }
//                         }
//                         if ui.button("Cancel").clicked() {
//                             app.show_menu = Menu::None;
//                         }
//                     });
//                 });
//             }

//             if app.show_menu == Menu::None && ui.button("Open").clicked() {
//                 app.show_menu = Menu::Open;
//             }
//             if app.show_menu == Menu::Open {
//                 egui::Window::new("Open").resizable(false).collapsible(false).movable(false).show(ctx, |ui| {
//                     ui.label("Enter filename:");    
//                     let resp = ui.text_edit_singleline(&mut app.open_filename);
//                     if resp.lost_focus() {
//                         app.show_menu = Menu::None;
//                     }
//                     ui.horizontal(|ui| {
//                         if ui.button("Open").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
//                             if app.open_filename.is_empty() {
//                                 app.status = "Please enter a filename".to_string();
//                             } else if app.open_filename.ends_with(".csv") {
//                                 app.show_menu = Menu::None;
//                                 let name = app.open_filename.clone();
//                                 app.status = open_csv(&name, &mut app);
//                             } else if app.open_filename.ends_with(".290") {
//                                 app.show_menu = Menu::None;
//                                 app.sheets = load_all_sheets(&app.open_filename);
//                                 app.current_sheet_index = 0;
//                                 app.status = "Loaded".to_string();
//                             } else {
//                                 app.status = "Please enter a valid filename".to_string();
//                             }
//                         }
//                         if ui.button("Cancel").clicked() {
//                             app.show_menu = Menu::None;
//                         }
//                     });
//                 });
//             }

//             if app.show_menu == Menu::None && ui.button("Find and Replace").clicked() {
//                 app.show_menu = Menu::FindAndReplace;
//             }
//             if app.show_menu == Menu::FindAndReplace {
//                 egui::Window::new("Find and Replace").resizable(false).collapsible(false).show(ctx, |ui| {
//                     ui.label("Find:");
//                     ui.text_edit_singleline(&mut app.find_text);
//                     ui.label("Replace with:");
//                     ui.text_edit_singleline(&mut app.replace_text);
//                     ui.horizontal(|ui| {
//                         if ui.button("Replace All").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
//                             if app.find_text.is_empty() {
//                                 app.status = "Please enter text to find".to_string();
//                             } else {
//                                 app.find_and_replace();
//                                 app.show_menu = Menu::None;
//                             }
//                         }
//                         if ui.button("Cancel").clicked() {
//                             app.show_menu = Menu::None;
//                             app.find_text.clear();
//                             app.replace_text.clear();
//                         }
//                     });
//                 });
//             }

//             if ui.button("Clear").clicked() { 
//                 for row in &mut app.sheets[app.current_sheet_index].sheet.data {
//                     for cell in row {
//                         cell.value = 0;
//                         cell.string = None;
//                         cell.is_error = false;
//                         cell.op_code = NoConstraint;
//                         cell.cell1 = CellInfo { row: -1, col: -1 };
//                         cell.cell2 = CellInfo { row: -1, col: -1 };
//                         cell.dependencies = HashSet::new();
//                     }
//                 }
//                 // app.undo_stack.push(Action::ClearSheet {
//                 //     sheet_index: app.current_sheet_index,
//                 //     old_data,
//                 // });
//                 app.redo_stack.clear();
//                 app.status = "Cleared".to_string();
//             }

//             if ui.button("Cut").clicked() {
//                 cut(app);
//             }

//             if ui.button("Copy").clicked() {
//                 copy(app);
//             }

//             if ui.button("Paste").clicked() {
//                 paste(app);
//             }

//             if ui.button("Undo").clicked() {
//                 app.undo();
//                 ctx.request_repaint();
//             }

//             if ui.button("Redo").clicked() {
//                 app.redo();
//                 ctx.request_repaint();
//             }

//             if ui.button("Theme").clicked() {
//                 app.show_menu = Menu::Theme;
//             }
//             if app.show_menu == Menu::Theme {
//                 egui::Window::new("Theme").resizable(false).collapsible(false).show(ctx, |ui| {
//                     ui.label("Select theme:");
//                     for theme in &THEMES {
//                         if ui.button(theme.name).clicked() {
//                             app.theme = theme.clone();
//                             app.show_menu = Menu::None;
//                         }
//                     }
//                 });
//             }
//         });
//     })
// }


use crate::ui::app_impl::{Menu, Action, SpreadsheetApp, CutCopy};
use crate::ui::themes::THEMES;
use crate::sheet_functions::{self, CellInfo};
use crate::sheet_functions::OpCode::*;
use std::collections::HashSet;
use crate::ui::utils::{load_all_sheets, save_all_sheets, convert_to_csv, open_csv};
use egui_plot::{Line, PlotPoints, Plot};
use crate::ui::fonts::FONTS;


pub fn cut(app: &mut SpreadsheetApp) {
    if let Some((row, col)) = app.selected_cell {
        let old_cell = app.sheets[app.current_sheet_index].sheet.data[row][col].clone();
        app.clipboard = Some((old_cell, CutCopy::Cut, row as i16, col as i16));
        app.cut_copied_cell = Some((row as i16, col as i16));
        app.status = "Cut".to_string();
    } else {
        app.status = "No cell selected".to_string();
    }
}

pub fn copy(app: &mut SpreadsheetApp) {
    if let Some((row, col)) = app.selected_cell {
        let mut cell = app.sheets[app.current_sheet_index].sheet.data[row][col].clone();
        cell.dependencies = HashSet::new();
        cell.cell1 = CellInfo { row: -1, col: -1 };
        cell.cell2 = CellInfo { row: -1, col: -1 };
        cell.op_code = NoConstraint;
        app.clipboard = Some((cell, CutCopy::Copied, row as i16, col as i16));
        app.cut_copied_cell = Some((row as i16, col as i16));
        app.status = "Copied".to_string();
    } else {
        app.status = "No cell selected".to_string();
    }
}

pub fn paste(app: &mut SpreadsheetApp) {
    if let Some((row, col)) = app.selected_cell {
        if let Some((cell, cut_copy, old_row, old_col)) = app.clipboard.clone() { 
            let old_cell1 = app.sheets[app.current_sheet_index].sheet.data[old_row as usize][old_col as usize].clone();
            let old_cell2 = app.sheets[app.current_sheet_index].sheet.data[row as usize][col as usize].clone();

            if cut_copy == CutCopy::Cut {
                let mut new_cell1 = old_cell1.clone();
                new_cell1.value = 0;
                new_cell1.string = None;
                new_cell1.is_error = false;
                new_cell1.op_code = NoConstraint;
                new_cell1.cell1 = CellInfo { row: -1, col: -1 };
                new_cell1.cell2 = CellInfo { row: -1, col: -1 };
                let depended_cell1 = app.sheets[app.current_sheet_index].sheet.data[old_row as usize][old_col as usize].cell1.clone();
                let depended_cell2 = app.sheets[app.current_sheet_index].sheet.data[old_row as usize][old_col as usize].cell2.clone();
                
                if depended_cell1.row != -1 && depended_cell1.col != -1 {
                    app.sheets[app.current_sheet_index].sheet.data[depended_cell1.row as usize][depended_cell1.col as usize].dependencies.remove(&(old_col as i32 * 1000 + old_row as i32));
                    app.sheets[app.current_sheet_index].sheet.data[depended_cell1.row as usize][depended_cell1.col as usize].dependencies.insert(col as i32 * 1000 + row as i32);
                }
                if depended_cell2.row != -1 && depended_cell2.col != -1 {
                    app.sheets[app.current_sheet_index].sheet.data[depended_cell2.row as usize][depended_cell2.col as usize].dependencies.remove(&(old_col as i32 * 1000 + old_row as i32));
                    app.sheets[app.current_sheet_index].sheet.data[depended_cell2.row as usize][depended_cell2.col as usize].dependencies.insert(col as i32 * 1000 + row as i32);
                }

                sheet_functions::remove_dependency(&CellInfo { row: row as i16, col: col as i16 }, &mut app.sheets[app.current_sheet_index].sheet);
                
                app.sheets[app.current_sheet_index].sheet.data[old_row as usize][old_col as usize] = new_cell1.clone();
                sheet_functions::change_dependecy_set(&mut new_cell1, &mut app.sheets[app.current_sheet_index].sheet, true);
                let mut new_cell2 = old_cell1.clone();
                sheet_functions::change_dependecy_set(&mut new_cell2, &mut app.sheets[app.current_sheet_index].sheet, false);
                sheet_functions::update_dependencies(old_row, old_col, row as i16, col as i16, &mut app.sheets[app.current_sheet_index].sheet);
                for dependency in old_cell2.dependencies.clone() {
                    let dependency_row = dependency % 1000;
                    let dependency_col = dependency / 1000;
                    app.sheets[app.current_sheet_index].sheet.data[dependency_row as usize][dependency_col as usize].value = 0;
                    app.sheets[app.current_sheet_index].sheet.data[dependency_row as usize][dependency_col as usize].is_error = true;
                    app.sheets[app.current_sheet_index].sheet.data[dependency_row as usize][dependency_col as usize].string = None;
                    app.sheets[app.current_sheet_index].sheet.data[dependency_row as usize][dependency_col as usize].op_code = NoConstraint;
                    app.sheets[app.current_sheet_index].sheet.data[dependency_row as usize][dependency_col as usize].cell1 = CellInfo { row: -1, col: -1 };
                    app.sheets[app.current_sheet_index].sheet.data[dependency_row as usize][dependency_col as usize].cell2 = CellInfo { row: -1, col: -1 };
                    sheet_functions::remove_dependency(&CellInfo { row: dependency_row as i16, col: dependency_col as i16 }, &mut app.sheets[app.current_sheet_index].sheet);
                }
                app.sheets[app.current_sheet_index].sheet.data[row as usize][col as usize] = new_cell2;
                app.redo_stack.clear();
                app.undo_stack.push(Action::CutAction {
                    sheet_index: app.current_sheet_index,
                    row1: old_row as i16,
                    col1: old_col as i16,
                    previous_cell1: old_cell1.clone(),
                    row2: row as i16,
                    col2: col as i16,
                    previous_cell2: old_cell2.clone(),
                });
                app.clipboard = None;
            }
            if cut_copy == CutCopy::Copied {
                let mut new_cell = cell.clone();
                new_cell.dependencies = old_cell2.dependencies.clone();
                app.sheets[app.current_sheet_index].sheet.data[row][col] = new_cell.clone();
                sheet_functions::recalculate_dependecy(CellInfo { row: row as i16, col: col as i16 }, &mut app.sheets[app.current_sheet_index].sheet);
                app.redo_stack.clear();
                app.undo_stack.push(Action::Inserted {
                    sheet_index: app.current_sheet_index,
                    row: row as i16,
                    col: col as i16,
                    previous_cell: old_cell2.clone(),
                });
            }
            app.cut_copied_cell = None;
            app.status = "Pasted".to_string();
        } else {
            app.status = "Nothing to paste".to_string();
        }
    } else {
        app.status = "No cell selected".to_string();
    }
}

pub fn show_menu(mut app: &mut SpreadsheetApp, ctx: &egui::Context, ui: &mut egui::Ui) -> egui::CollapsingResponse<()> {
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
                                    let name = app.open_filename.clone();
                                    app.status = open_csv(&name, &mut app);
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

                if app.show_menu == Menu::None && ui.button("Plot Graph").clicked() {
                    app.show_menu = Menu::PlotGraph;
                }
                if app.show_menu == Menu::PlotGraph {
                    egui::Window::new("Plot Graph").resizable(false).collapsible(false).show(ctx, |ui| {
                        ui.label("Enter first column (e.g., A):");
                        ui.text_edit_singleline(&mut app.plot_column1);
                        ui.label("Enter second column (e.g., B):");
                        ui.text_edit_singleline(&mut app.plot_column2);
                        ui.label("Enter start row (e.g., 1):");
                        ui.text_edit_singleline(&mut app.plot_row_start);
                        ui.label("Enter end row (e.g., 5):");
                        ui.text_edit_singleline(&mut app.plot_row_end);
                        ui.horizontal(|ui| {
                            if ui.button("Plot").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                // Validate inputs
                                let col1 = sheet_functions::col_name_to_col_num(&app.plot_column1);
                                let col2 = sheet_functions::col_name_to_col_num(&app.plot_column2);
                                let row_start = app.plot_row_start.parse::<i32>().ok().map(|r| r - 1);
                                let row_end = app.plot_row_end.parse::<i32>().ok().map(|r| r - 1);
                                let sheet = &app.sheets[app.current_sheet_index].sheet;

                                if col1 >= 0 && col2 >= 0 && col1 < sheet.cols && col2 < sheet.cols && 
                                   row_start.is_some() && row_end.is_some() && 
                                   row_start.unwrap() >= 0 && row_end.unwrap() < sheet.rows && 
                                   row_start.unwrap() <= row_end.unwrap() {
                                    app.show_plot = true;
                                    app.show_menu = Menu::None;
                                    app.status = "Plotting graph".to_string();
                                } else {
                                    app.status = "Invalid column or row range".to_string();
                                }
                            }
                            if ui.button("Cancel").clicked() {
                                app.show_menu = Menu::None;
                                app.plot_column1.clear();
                                app.plot_column2.clear();
                                app.plot_row_start.clear();
                                app.plot_row_end.clear();
                            }
                        });
                    });
                }

                if app.show_plot {
                    egui::Window::new("Graph").resizable(true).show(ctx, |ui| {
                        let col1 = sheet_functions::col_name_to_col_num(&app.plot_column1) as usize;
                        let col2 = sheet_functions::col_name_to_col_num(&app.plot_column2) as usize;
                        let row_start = app.plot_row_start.parse::<usize>().unwrap() - 1;
                        let row_end = app.plot_row_end.parse::<usize>().unwrap() - 1;
                        let sheet = &app.sheets[app.current_sheet_index].sheet;

                        // Collect plot points
                        let mut points: Vec<[f64; 2]> = Vec::new();
                        for row in row_start..=row_end {
                            let x = if sheet.data[row][col1].is_error {
                                0.0
                            } else {
                                sheet.data[row][col1].value as f64
                            };
                            let y = if sheet.data[row][col2].is_error {
                                0.0
                            } else {
                                sheet.data[row][col2].value as f64
                            };
                            points.push([x, y]);
                        }

                        let plot_points = PlotPoints::from(points);
                        let line = Line::new(plot_points).color(app.theme.text_color);
                        Plot::new("Spreadsheet Plot")
                            .view_aspect(2.0)
                            .show(ui, |plot_ui| plot_ui.line(line));
                        
                        if ui.button("Close").clicked() {
                            app.show_plot = false;
                            app.plot_column1.clear();
                            app.plot_column2.clear();
                            app.plot_row_start.clear();
                            app.plot_row_end.clear();
                        }
                    });
                }

                if ui.button("Clear").clicked() { 
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
                    app.redo_stack.clear();
                    app.status = "Cleared".to_string();
                }

                if ui.button("Cut").clicked() {
                    cut(app);
                }

                if ui.button("Copy").clicked() {
                    copy(app);
                }

                if ui.button("Paste").clicked() {
                    paste(app);
                }

                if ui.button("Undo").clicked() {
                    app.undo();
                    ctx.request_repaint();
                }

                if ui.button("Redo").clicked() {
                    app.redo();
                    ctx.request_repaint();
                }
                if ui.button("Font").clicked() {
                    app.show_menu = Menu::Font;
                }
                if app.show_menu == Menu::Font {
                    egui::Window::new("Font Selection").resizable(false).collapsible(false).show(ctx, |ui| {
                        ui.label("Select font:");
                        for (index, font) in FONTS.iter().enumerate() {
                            if ui.button(font.name).clicked() {
                                if app.current_font_index != index {
                                    app.current_font_index = index;
                                    crate::ui::fonts::setup_custom_fonts(ctx, &FONTS[index]);
                                }
                                app.show_menu = Menu::None;
                            }
                        }
                    });
                }
                if ui.button("Theme").clicked() {
                    app.show_menu = Menu::Theme;
                }
                if app.show_menu == Menu::Theme {
                    egui::Window::new("Theme").resizable(false).collapsible(false).show(ctx, |ui| {
                        ui.label("Select theme:");
                        for theme in &THEMES {
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
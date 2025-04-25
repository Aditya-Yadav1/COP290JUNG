# Spreadsheet Application

## Overview

This project is a Rust-based spreadsheet application built using the `eframe` framework with `egui` for the graphical user interface. It provides a fully functional spreadsheet with features similar to modern spreadsheet software, including cell editing, formulas, undo/redo functionality, cut/copy/paste, find and replace, sorting, graphing, and theme customization. The application supports multiple sheets, CSV import/export, and a robust dependency tracking system for formula calculations.

## Features

### Core Spreadsheet Functionality
- **Cell Editing**: Supports direct cell editing with numeric, string, and formula inputs.
- **Formulas**: Implements a wide range of operations, including:
  - Arithmetic operations (`+`, `-`, `*`, `/`) between cells or constants.
  - Range functions (`SUM`, `MIN`, `MAX`, `AVG`, `STDEV`).
  - Cell referencing (e.g., `A1=B2`).
  - Sleep function (`SLEEP`) for timing operations.
- **Dependency Tracking**: Automatically recalculates dependent cells when a cell's value changes, with cycle detection to prevent infinite loops.
- **Undo/Redo**: Supports undoing and redoing actions like cell edits, cut/paste, find and replace, and sorting.
- **Cut/Copy/Paste**: Allows cutting, copying, and pasting cell contents, with dependency updates for cut operations.

### User Interface
- **Grid Display**: Displays a scrollable grid with row and column headers, styled with customizable themes.
- **Formula Bar**: Shows the formula or value of the selected cell and allows direct formula input.
- **Status Bar**: Displays the current mode, status messages, execution time, and selected cell information.
- **Menu System**: Provides options for:
  - File operations (Save, Open, Save to CSV).
  - Editing (Cut, Copy, Paste, Undo, Redo, Find and Replace).
  - Visualization (Plot Graph).
  - Customization (Font, Theme).
  - Sheet management (New Sheet, Delete Sheet).
  - Sorting and cell navigation (Scroll to).

### Data Management
- **Multiple Sheets**: Supports creating, switching, and deleting sheets, each with configurable rows and columns.
- **CSV Import/Export**: Import data from CSV files and export sheets to CSV format.
- **Serialization**: Saves and loads entire spreadsheet state (including multiple sheets) in a custom `.290` format using JSON.

### Advanced Features
- **Find and Replace**: Replaces text or values across the sheet with undo support.
- **Sorting**: Sorts a range of cells by a specified column or row, with ascending or descending order.
- **Graphing**: Plots data from two columns as a line graph, with customizable row ranges.
- **Theme Customization**: Supports light and dark themes, with customizable colors for cells, headers, and text.
- **Font Selection**: Allows changing the font used in the UI.

## Project Structure

The project is organized into several key modules, each responsible for specific functionality. Below is a description of the main files provided:

- **`app.rs`**: The main application logic, implementing the `eframe::App` trait. Handles the overall UI layout, including the formula bar, status bar, and sheet tabs. Manages user input (e.g., keyboard shortcuts) and integrates other modules.
- **`app_impl.rs`**: Defines the core `SpreadsheetApp` struct and its methods, including initialization, find and replace, undo/redo, and sheet creation. Manages the application's state, such as sheets, clipboard, and stacks.
- **`sheet_display.rs`**: Handles the rendering of the spreadsheet grid using `egui`. Manages cell selection, editing, and scrolling, with visual styling based on the selected theme.
- **`menu.rs`**: Implements the menu system, including buttons and dialogs for file operations, editing, graphing, sorting, and customization. Handles cut/copy/paste logic and plotting.
- **`utils.rs`**: Provides utility functions for CSV import/export, serialization, and undo/redo support for specific actions (e.g., insertions, cuts, sorting).
- **`sheet_functions.rs`**: Defines the core data structures (`Sheet`, `Cell`, `CellInfo`) and functions for managing spreadsheet data, including cell creation, dependency tracking, topological sorting, and recalculation.
- **`parser.rs`**: Parses user commands (e.g., formulas, scroll commands) using regular expressions. Supports a variety of formula types and updates the sheet accordingly.

### Assumed Additional Files
The following files are not provided but are assumed to exist based on references in the code:
- **`calculate_functions.rs`**: Contains functions like `compute_cell` and `compute_range_func` for performing calculations based on `OpCode` (e.g., arithmetic, range functions).
- **`ui_sheet_functions.rs`**: Includes functions for managing dependencies (`change_dependecy_set`, `update_dependencies`, `recalculate_dependecy`) and sorting (`sort_sheet`).
- **`fonts.rs`**: Defines the `FONTS` array and `setup_custom_fonts` for font customization.
- **`themes.rs`**: Defines the `THEMES` array and `Theme` struct for theme customization.


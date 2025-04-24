#[cfg(test)]
mod tests {
    use crate::parser::{parse_command, get_op_code, get_op_code2, func_to_op_code};
    use crate::sheet_functions::{Sheet, CellInfo, OpCode, col_name_to_col_num, col_num_to_col_name, is_valid_cell, add_constraints, recalculate, topological_sort, sort_sheet};
    use crate::calculate_functions::{compute_cell, compute_range_func};
    use std::collections::HashMap;
    use std::string::String;

    // Helper function to create a new sheet
    fn new_sheet(rows: i32, cols: i32) -> Sheet {
        Sheet::new(rows, cols)
    }

    #[test]
    fn test_col_name_conversion() {
        assert_eq!(col_name_to_col_num("A"), 0);
        assert_eq!(col_name_to_col_num("B"), 1);
        assert_eq!(col_name_to_col_num("Z"), 25);
        assert_eq!(col_name_to_col_num("AA"), 26);
        assert_eq!(col_name_to_col_num("AB"), 27);

        assert_eq!(col_num_to_col_name(0), "A");
        assert_eq!(col_num_to_col_name(1), "B");
        assert_eq!(col_num_to_col_name(25), "Z");
        assert_eq!(col_num_to_col_name(26), "AA");
        assert_eq!(col_num_to_col_name(27), "AB");
    }

    #[test]
    fn test_is_valid_cell() {
        assert!(is_valid_cell(0, 0, 10, 10));
        assert!(is_valid_cell(9, 9, 10, 10));
        assert!(!is_valid_cell(-1, 0, 10, 10));
        assert!(!is_valid_cell(0, -1, 10, 10));
        assert!(!is_valid_cell(10, 0, 10, 10));
        assert!(!is_valid_cell(0, 10, 10, 10));
    }

    #[test]
    fn test_get_op_code() {
        assert_eq!(get_op_code('+', false), OpCode::CellPlusConstant);
        assert_eq!(get_op_code('-', false), OpCode::CellMinusConstant);
        assert_eq!(get_op_code('*', false), OpCode::CellTimesConstant);
        assert_eq!(get_op_code('/', false), OpCode::CellDivideConstant);
        assert_eq!(get_op_code('/', true), OpCode::ConstantDividesCell);
        assert_eq!(get_op_code('?', false), OpCode::NoConstraint);
    }

    #[test]
    fn test_get_op_code2() {
        assert_eq!(get_op_code2('+'), OpCode::CellPlusCell);
        assert_eq!(get_op_code2('-'), OpCode::CellMinusCell);
        assert_eq!(get_op_code2('*'), OpCode::CellTimesCell);
        assert_eq!(get_op_code2('/'), OpCode::CellDivideCell);
        assert_eq!(get_op_code2('?'), OpCode::NoConstraint);
    }

    #[test]
    fn test_func_to_op_code() {
        assert_eq!(func_to_op_code("SUM"), OpCode::Sum);
        assert_eq!(func_to_op_code("MIN"), OpCode::Min);
        assert_eq!(func_to_op_code("MAX"), OpCode::Max);
        assert_eq!(func_to_op_code("AVG"), OpCode::Avg);
        assert_eq!(func_to_op_code("STDEV"), OpCode::Stdev);
        assert_eq!(func_to_op_code("INVALID"), OpCode::NoConstraint);
    }

    #[test]
    fn test_parse_command_navigation() {
        let mut sheet = new_sheet(20, 20);
        let mut row_start = 5;
        let mut col_start = 5;
        let mut time = 0.0;
        let mut status = String::new();
        let mut print_enabled = true;
        let  total_rows = 20;
        let  total_cols = 20;

        // Test 'w' (move up)
        parse_command("w", &mut row_start, &mut col_start, &mut time, &mut status, &total_rows, &total_cols, &mut sheet, &mut print_enabled);
        assert_eq!(row_start, 0);
        assert_eq!(status, "ok");

        // Test 's' (move down)
        parse_command("s", &mut row_start, &mut col_start, &mut time, &mut status,  &total_rows, &total_cols, &mut sheet, &mut print_enabled);
        assert_eq!(row_start, 10);
        assert_eq!(status, "ok");

        // Test 'a' (move left)
        parse_command("a", &mut row_start, &mut col_start, &mut time, &mut status,  &total_rows, &total_cols, &mut sheet, &mut print_enabled);
        assert_eq!(col_start, 0);
        assert_eq!(status, "ok");

        // Test 'd' (move right)
        parse_command("d", &mut row_start, &mut col_start, &mut time, &mut status,  &total_rows, &total_cols, &mut sheet, &mut print_enabled);
        assert_eq!(col_start, 10);
        assert_eq!(status, "ok");

        // Test 'scroll_to'
        parse_command("scroll_toB3", &mut row_start, &mut col_start, &mut time, &mut status,  &total_rows, &total_cols, &mut sheet, &mut print_enabled);
        assert_eq!(row_start, 2);
        assert_eq!(col_start, 1);
        assert_eq!(status, "ok");

        // Test invalid scroll_to
        parse_command("scroll_toB21", &mut row_start, &mut col_start, &mut time, &mut status,  &total_rows, &total_cols, &mut sheet, &mut print_enabled);
        assert_eq!(status, "err");
    }

    #[test]
    fn test_parse_command_cell_operations() {
        let mut sheet = new_sheet(10, 10);
        let mut row_start = 0;
        let mut col_start = 0;
        let mut time = 0.0;
        let mut status = String::new();
        let mut print_enabled = true;
        let  total_rows = 10;
        let  total_cols = 10;

        // Test cell = int
        parse_command("A1=42", &mut row_start, &mut col_start, &mut time, &mut status, &total_rows, &total_cols, &mut sheet, &mut print_enabled);
        assert_eq!(sheet.data[0][0].value, 42);
        assert_eq!(sheet.data[0][0].is_error, false);
        assert_eq!(status, "ok");

        // Test cell = string
        parse_command("A2=\"hello\"", &mut row_start, &mut col_start, &mut time, &mut status, &total_rows, &total_cols, &mut sheet, &mut print_enabled);
        assert_eq!(sheet.data[1][0].string, Some("hello".to_string()));
        assert_eq!(status, "ok");

        // Test cell = int op int
        parse_command("A3=5+3", &mut row_start, &mut col_start, &mut time, &mut status, &total_rows, &total_cols, &mut sheet, &mut print_enabled);
        assert_eq!(sheet.data[2][0].value, 8);
        assert_eq!(status, "ok");

        // Test cell = cell
        parse_command("A4=A1", &mut row_start, &mut col_start, &mut time, &mut status, &total_rows, &total_cols, &mut sheet, &mut print_enabled);
        assert_eq!(sheet.data[3][0].value, 42);
        assert_eq!(status, "ok");

        // Test cell = cell op cell
        parse_command("B1=A1+A1", &mut row_start, &mut col_start, &mut time, &mut status, &total_rows, &total_cols, &mut sheet, &mut print_enabled);
        assert_eq!(sheet.data[0][1].value, 84);
        assert_eq!(status, "ok");

        // Test cell = int op cell
        parse_command("B2=10+A1", &mut row_start, &mut col_start, &mut time, &mut status, &total_rows, &total_cols, &mut sheet, &mut print_enabled);
        assert_eq!(sheet.data[1][1].value, 52);
        assert_eq!(status, "ok");

        // Test cell = cell op int
        parse_command("B3=A1+10", &mut row_start, &mut col_start, &mut time, &mut status, &total_rows, &total_cols, &mut sheet, &mut print_enabled);
        assert_eq!(sheet.data[2][1].value, 52);
        assert_eq!(status, "ok");

        // Test cell = func(cell:cell)
        parse_command("C1=SUM(A1:A1)", &mut row_start, &mut col_start, &mut time, &mut status, &total_rows, &total_cols, &mut sheet, &mut print_enabled);
        assert_eq!(sheet.data[0][2].value, 42);
        assert_eq!(status, "ok");

        // Test sleep(int)
        parse_command("C2=SLEEP(2)", &mut row_start, &mut col_start, &mut time, &mut status, &total_rows, &total_cols, &mut sheet, &mut print_enabled);
        assert_eq!(sheet.data[1][2].value, 2);
        assert_eq!(status, "ok");
        assert_eq!(time, 2.0);

        // Test sleep(cell)
        parse_command("C3=SLEEP(C2)", &mut row_start, &mut col_start, &mut time, &mut status, &total_rows, &total_cols, &mut sheet, &mut print_enabled);
        assert_eq!(status, "ok");
    }

    #[test]
    fn test_parse_command_error_cases() {
        let mut sheet = new_sheet(5, 5);
        let mut row_start = 0;
        let mut col_start = 0;
        let mut time = 0.0;
        let mut status = String::new();
        let mut print_enabled = true;
        let  total_rows = 5;
        let  total_cols = 5;

        // Invalid cell reference
        parse_command("Z1=42", &mut row_start, &mut col_start, &mut time, &mut status, &total_rows, &total_cols, &mut sheet, &mut print_enabled);
        assert_eq!(status, "ok");

        // Division by zero
        parse_command("A1=10/0", &mut row_start, &mut col_start, &mut time, &mut status, &total_rows, &total_cols, &mut sheet, &mut print_enabled);
        assert_eq!(sheet.data[0][0].is_error, true);
        assert_eq!(status, "err");

        // Invalid command
        parse_command("INVALID", &mut row_start, &mut col_start, &mut time, &mut status, &total_rows, &total_cols, &mut sheet, &mut print_enabled);
        assert_eq!(status, "err");

        // Type error in func
        parse_command("A1=\"text\"", &mut row_start, &mut col_start, &mut time, &mut status, &total_rows, &total_cols, &mut sheet, &mut print_enabled);
        parse_command("A2=SUM(A1:A1)", &mut row_start, &mut col_start, &mut time, &mut status, &total_rows, &total_cols, &mut sheet, &mut print_enabled);
        assert_eq!(status, "type error");
    }

    #[test]
    fn test_compute_cell() {
        let mut status = String::new();
        assert_eq!(compute_cell(OpCode::CellPlusCell, 5, 3, &mut status), (8, false));
        assert_eq!(compute_cell(OpCode::CellMinusCell, 5, 3, &mut status), (2, false));
        assert_eq!(compute_cell(OpCode::CellTimesCell, 5, 3, &mut status), (15, false));
        assert_eq!(compute_cell(OpCode::CellDivideCell, 6, 2, &mut status), (3, false));
        assert_eq!(compute_cell(OpCode::CellDivideCell, 6, 0, &mut status), (-1, true));
        assert_eq!(compute_cell(OpCode::ConstantDividesCell, 0, 6, &mut status), (-1, true));
        assert_eq!(compute_cell(OpCode::NoConstraint, 0, 0, &mut status), (-1, true));
    }

    #[test]
    fn test_compute_range_func() {
        let mut sheet = new_sheet(5, 5);
        let mut row_start = 0;
        let mut col_start = 0;
        let mut time = 0.0;
        let mut status = String::new();
        let mut print_enabled = true;
        let total_rows = 5;
        let total_cols = 5;
    
        // Set up test data: A1=10, B1=20, A2=30, B2=40
        parse_command("A1=10", &mut row_start, &mut col_start, &mut time, &mut status, &total_rows, &total_cols, &mut sheet, &mut print_enabled);
        parse_command("B1=20", &mut row_start, &mut col_start, &mut time, &mut status, &total_rows, &total_cols, &mut sheet, &mut print_enabled);
        parse_command("A2=30", &mut row_start, &mut col_start, &mut time, &mut status, &total_rows, &total_cols, &mut sheet, &mut print_enabled);
        parse_command("B2=40", &mut row_start, &mut col_start, &mut time, &mut status, &total_rows, &total_cols, &mut sheet, &mut print_enabled);
    
        // Test SUM: C1 = SUM(A1:B2)
        parse_command("C1=SUM(A1:B2)", &mut row_start, &mut col_start, &mut time, &mut status, &total_rows, &total_cols, &mut sheet, &mut print_enabled);
        assert_eq!(status, "ok");
        assert_eq!(sheet.data[0][2].value, 100); // 10 + 20 + 30 + 40 = 100
    
        // Test MIN: C2 = MIN(A1:B2)
        parse_command("C2=MIN(A1:B2)", &mut row_start, &mut col_start, &mut time, &mut status, &total_rows, &total_cols, &mut sheet, &mut print_enabled);
        assert_eq!(status, "ok");
        assert_eq!(sheet.data[1][2].value, 10); // Min is 10
    
        // Test MAX: C3 = MAX(A1:B2)
        parse_command("C3=MAX(A1:B2)", &mut row_start, &mut col_start, &mut time, &mut status, &total_rows, &total_cols, &mut sheet, &mut print_enabled);
        assert_eq!(status, "ok");
        assert_eq!(sheet.data[2][2].value, 40); // Max is 40
    
        // Test AVG: C4 = AVG(A1:B2)
        parse_command("C4=AVG(A1:B2)", &mut row_start, &mut col_start, &mut time, &mut status, &total_rows, &total_cols, &mut sheet, &mut print_enabled);
        assert_eq!(status, "ok");
        assert_eq!(sheet.data[3][2].value, 25); // (10 + 20 + 30 + 40) / 4 = 25
    
        // Test STDEV: C5 = STDEV(A1:B2)
        parse_command("C5=STDEV(A1:B2)", &mut row_start, &mut col_start, &mut time, &mut status, &total_rows, &total_cols, &mut sheet, &mut print_enabled);
        assert_eq!(status, "ok");
        assert_eq!(sheet.data[4][2].value, 13); // Standard deviation ≈ 12.9, rounded to 13
    
        // Test error case: invalid range
        parse_command("D1=SUM(B2:A1)", &mut row_start, &mut col_start, &mut time, &mut status, &total_rows, &total_cols, &mut sheet, &mut print_enabled);
        assert_eq!(status, "Invalid cmd");
    }

    #[test]
fn test_add_constraints() {
    // Test Cell = int
    let mut sheet = new_sheet(5, 5);
    let mut status = String::new();
    let mut sleep_timer = 0;
    let cell = CellInfo { row: 0, col: 0 };
    let cell1 = CellInfo { row: -1, col: -1 };
    let cell2 = CellInfo { row: -1, col: -1 };
    sheet.data[0][0].value = 42;
    add_constraints(cell, cell1, cell2, OpCode::NoConstraint, &mut sheet, &mut status, &mut sleep_timer);
    assert_eq!(sheet.data[0][0].value, 42);
    assert_eq!(status, "ok");

    // Test Cell = cell
    let mut sheet = new_sheet(5, 5);
    let mut status = String::new();
    let mut sleep_timer = 0;
    sheet.data[0][0].value = 42; // Set A1 to 42
    let cell = CellInfo { row: 1, col: 0 };
    let cell1 = CellInfo { row: 0, col: 0 };
    let cell2 = CellInfo { row: -1, col: -1 };
    add_constraints(cell, cell1, cell2, OpCode::CellEqualsCell, &mut sheet, &mut status, &mut sleep_timer);
    assert_eq!(sheet.data[1][0].value, 42);
    assert_eq!(status, "ok");

    // Test circular dependency
    let mut sheet = new_sheet(5, 5);
    let mut status = String::new();
    let mut sleep_timer = 0;
    let cell = CellInfo { row: 0, col: 0 };
    let cell1 = CellInfo { row: 0, col: 0 };
    let cell2 = CellInfo { row: -1, col: -1 };
    add_constraints(cell, cell1, cell2, OpCode::CellEqualsCell, &mut sheet, &mut status, &mut sleep_timer);
    assert_eq!(status, "circular error");
}

    #[test]
    fn test_recalculate() {
        let mut sheet = new_sheet(5, 5);
        let mut sleep_timer = 0;

        // Set up A1 = 10, A2 = A1 + 5
        sheet.data[0][0].value = 10;
        sheet.data[1][0].op_code = OpCode::CellPlusConstant;
        sheet.data[1][0].cell1 = CellInfo { row: 0, col: 0 };
        sheet.data[1][0].cell2 = CellInfo { row: 0, col: 5 };
        sheet.data[0][0].dependencies.insert(1000);

        recalculate(&mut sheet, 1, 0, &mut sleep_timer);
        assert_eq!(sheet.data[1][0].value, 15);
        assert_eq!(sheet.data[1][0].is_error, false);
    }

    #[test]
    fn test_topological_sort() {
        let mut sheet = new_sheet(5, 5);
        let mut avl_tree = HashMap::new();
    
        // Set up dependencies:
        // A1 = 10
        // A2 = A1 + 5 (depends on A1)
        // A3 = A2 + 5 (depends on A2)
        sheet.data[0][0].value = 10; // A1
        sheet.data[1][0].op_code = OpCode::CellPlusConstant;
        sheet.data[1][0].cell1 = CellInfo { row: 0, col: 0 };
        sheet.data[1][0].cell2 = CellInfo { row: 0, col: 5 };
        sheet.data[0][0].dependencies.insert(1000); // A1 -> A2
        sheet.data[2][0].op_code = OpCode::CellPlusConstant;
        sheet.data[2][0].cell1 = CellInfo { row: 1, col: 0 };
        sheet.data[2][0].cell2 = CellInfo { row: 0, col: 5 };
        sheet.data[1][0].dependencies.insert(2000); // A2 -> A3
    
        // Set up avl_tree with indegrees
        avl_tree.insert(0, 0); // A1 has no dependencies
        avl_tree.insert(1000, 1); // A2 depends on A1
        avl_tree.insert(2000, 1); // A3 depends on A2
    
        let sorted = topological_sort(&mut avl_tree, &sheet);
        assert_eq!(sorted, vec![0, 1000, 2000]); // A1, A2, A3 in order
    
        // Verify values after recalculation
        let mut sleep_timer = 0;
        recalculate(&mut sheet, 1, 0, &mut sleep_timer); // A2
        recalculate(&mut sheet, 2, 0, &mut sleep_timer); // A3
        assert_eq!(sheet.data[1][0].value, 15); // A2 = 10 + 5
        assert_eq!(sheet.data[2][0].value, 20); // A3 = 15 + 5
    }

    #[test]
    fn test_sort_sheet() {
        let mut sheet = new_sheet(5, 5);
        sheet.data[0][0].value = 30;
        sheet.data[1][0].value = 10;
        sheet.data[2][0].value = 20;
        sort_sheet(&mut sheet, 0, 0, 0, 2, "A", true, "asc");
        assert_eq!(sheet.data[0][0].value, 10);
        assert_eq!(sheet.data[1][0].value, 20);
        assert_eq!(sheet.data[2][0].value, 30);
    }
}
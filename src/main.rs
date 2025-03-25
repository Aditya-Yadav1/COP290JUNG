use std::env;
mod sheet_functions;
mod parser;
mod calculate_functions;


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3{
        println!("Enter the row and column number");
        return;
    }
    let row : i32 = args[1].parse().unwrap();
    let col : i32 = args[2].parse().unwrap();

    let mut row_start = 0;
    let mut col_start = 0;
    let mut time: f32 = 0.0;
    let mut status = String::new();
    status.push_str("ok");
    let mut sheet = sheet_functions::Sheet::new(row, col);
    loop{
        sheet_functions::print_sheet(row_start, col_start, row, col, &mut sheet);
        println!("[{:.1}] ({}) >", time, status);   
        let mut command = String::new();
        std::io::stdin().read_line(&mut command).unwrap();
        parser::parse_command(&command, &mut row_start, &mut col_start, &mut time, &mut status, &row, &col, &mut sheet);
    }
    
    
}





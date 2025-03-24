pub fn parse_command(command:&str, row_start:&mut i32, col_start:&mut i32, time:&mut f32, status:&mut String, total_rows:&i32, total_cols:&i32){
    match command.trim() {
        "w" =>{
            *row_start = std::cmp::max(0, *row_start - 10);
            *status = String::from("ok");
        }
        "s" =>{
            *row_start = std::cmp::min(*row_start + 10, total_rows - 1);   
            *status = String::from("ok");
        }
        "a" =>{
            *col_start = std::cmp::max(0, *col_start - 10);
            *status = String::from("ok");
        }
        "d" =>{
            *col_start = std::cmp::min(*col_start + 10, total_cols - 1);
            *status = String::from("ok");
        }
        "q" =>{
            std::process::exit(0);
            *status = String::from("ok");
        }
        _ =>{
            *status = String::from("Invalid command");
        }
    }
}
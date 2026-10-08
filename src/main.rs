fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match working_stack::json_input::process_args(&args).and_then(|args| working_stack::cli::run(&args)) {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("{}", e.message());
            std::process::exit(e.exit_code());
        }
    }
}

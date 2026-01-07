use clap::Parser;
use stripe_csv::{
    args::{Args, CsvType},
    parser,
};

fn main() {
    let arguments = Args::parse();

    if let Err(error) = match arguments.csv_type {
        CsvType::Fees => parser::fees::parse(arguments.file, arguments.output_file),
    } {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }

    println!("done.");
}

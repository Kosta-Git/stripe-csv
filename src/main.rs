use clap::Parser;
use stripe_csv::{
    args::{Args, CsvType},
    parser,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = Args::parse();

    match arguments.csv_type {
        CsvType::Fees => {
            parser::fees::parse(arguments.file, arguments.output_file)?;
        }
    }

    println!("done.");
    Ok(())
}

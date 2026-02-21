use clap::Parser;
use rustyapa::cli_format::Format;
use std::fs::File;

#[derive(Parser, Debug)]

struct CliArgs {
    #[arg(long)]
    input: String,
    #[arg(long)]
    input_format: Format,
    #[arg(long)]
    output_format: Format,
}

fn run(args: CliArgs) -> Result<(), Box<dyn std::error::Error>> {
    let f = File::open(&args.input).map_err(|e| {
        std::io::Error::new(
            e.kind(),
            format!("Error opening a file {} {}", args.input, e),
        )
    })?;

    let stdout = &mut std::io::stdout().lock();
    let data = args.input_format.codec().parse(f)?;
    println!("{} records successfully ingested\n", data.len());

    args.output_format.codec().write(stdout, &data)?;
    Ok(())
}

fn main() {
    // parse args
    let args = CliArgs::parse();

    // run app
    println!(
        "Converting from '{}':{} to :{}",
        args.input, args.input_format, args.output_format
    );
    let app_result = run(args);

    // handle errors
    if let Err(e) = app_result {
        eprintln!("Error occured during application execution: {}", e);
        std::process::exit(1);
    }
}

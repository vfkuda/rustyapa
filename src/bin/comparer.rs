use clap::Parser;
use parser::{codecs::base::Format, domain::tx::TxRecord};
use std::{collections::HashMap, fs::File};

#[derive(Parser, Debug)]
struct CliArgs {
    #[arg(long)]
    file1: String,
    #[arg(long)]
    format1: Format,
    #[arg(long)]
    file2: String,
    #[arg(long)]
    format2: Format,
}

fn read_records_from_file(
    file_format: &Format,
    filename: &str,
) -> Result<Vec<TxRecord>, Box<dyn std::error::Error>> {
    let f = File::open(filename).map_err(|e| {
        std::io::Error::new(e.kind(), format!("Error opening a file {} {}", filename, e))
    })?;
    Ok(file_format.parse(f)?)
}

fn run(args: CliArgs) -> Result<(), Box<dyn std::error::Error>> {
    // read and 'count' transactions
    // count is number_of_occurences_in_file1 - number_of_occurences_in_file2 for each unique (by hash) transaction
    let mut record_count = HashMap::new();
    {
        // reading first file
        let ds1_records = read_records_from_file(&args.format1, &args.file1)?;
        for item in ds1_records.into_iter() {
            *record_count.entry(item).or_insert(0) += 1;
        }
    }
    {
        // reading second file
        let ds2_records = read_records_from_file(&args.format2, &args.file2)?;
        for item in ds2_records {
            *record_count.entry(item).or_insert(0) -= 1;
        }
    }
    // cleaning up recrods with 0 counts
    record_count.retain(|_, v| 0 != *v);

    // 0 count mean the exact record appears same number of times in both files
    if record_count.len() == 0 {
        println!("All transaction records are identical.");
    } else {
        println!(
            "There are {} unique transactions that don't match between the files",
            record_count.len()
        );
        // number of occurences is zero - means there are no
        for (item, count) in record_count.into_iter() {
            println!(
                "There is no equivivalent for transaction {} in the file '{}'",
                item.id,
                if count > 0 { "#1" } else { "#2" }
            );
        }
    }

    Ok(())
}

fn main() {
    // parse args
    let args = CliArgs::parse();

    // run app
    println!(
        "Comparing 2 files\n\t1:'{}':{}\n\t2:'{}':{}\n",
        args.file1, args.format1, args.file2, args.format2
    );
    let app_result = run(args);

    // handle errors
    if let Err(e) = app_result {
        eprintln!("Error occured during application execution: {}", e);
        std::process::exit(1);
    }
}

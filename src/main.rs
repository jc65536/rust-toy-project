use std::collections::hash_map::HashMap;
use std::fs;
use std::io::BufRead;
use std::io::BufReader;

use clap::Parser;

mod recdir;
use recdir::RecDir;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    dir: String,
    /// Aggregate files with the same extension
    #[clap(short = 'A')]
    aggregate: bool,
}

struct LineInfo {
    count: i32,
    blank_count: i32,
}

fn main() {
    let args = Args::parse();
    let mut ext_totals: HashMap<String, LineInfo> = HashMap::new();
    let mut all_total = LineInfo {
        count: 0,
        blank_count: 0,
    };

    for filename in RecDir::new(&args.dir) {
        let file = fs::File::open(&filename).expect("Error opening file.");
        let mut reader = BufReader::new(&file);
        let mut line = String::new();
        let ext = filename.extension().map(|os_str| os_str.to_str()).flatten().unwrap_or("miscellaneous").to_owned();

        let total = if args.aggregate {
            ext_totals.entry(ext).or_insert(LineInfo {
                count: 0,
                blank_count: 0,
            })
        } else {
            &mut all_total
        };

        loop {
            match reader.read_line(&mut line) {
                Ok(v) => {
                    if v == 0 {
                        break;
                    }
                    if line.starts_with('\n') {
                        total.blank_count += 1;
                    }
                    line = String::new();
                    total.count += 1;
                }
                Err(_) => break,
            }
        }
    }

    if args.aggregate {
        for pair in ext_totals.iter() {
            let ext = pair.0;
            let total = pair.1;
            println!("There are {} lines of code in {} files.", total.count, ext);
            println!("There are {} empty lines in {} files.", total.blank_count, ext);
            println!(
                "{:.2}% of the lines in {} files are empty.",
                100. * total.blank_count as f64 / total.count as f64,
                ext
            );
        }
    } else {
        println!("There are {} lines of code.", all_total.count);
        println!("There are {} empty lines.", all_total.blank_count);
        println!(
            "{:.2}% of the lines are empty.",
            100. * all_total.blank_count as f64 / all_total.count as f64
        );
    }
}

use std::collections::hash_map::HashMap;
use std::ffi::OsStr;
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

#[derive(Clone, Copy)]
struct CodeInfo {
    lines: i32,
    blanks: i32,
}

fn output(info: CodeInfo, ext: Option<&String>) {
    let ext_info = ext.map_or(String::new(), |ext| format!(" in {} files", ext));
    let pct = 100. * info.blanks as f64 / info.lines as f64;
    println!("There are {} lines of code{}.", info.lines, ext_info);
    println!("There are {} empty lines{}.", info.blanks, ext_info);
    println!("{:.2}% of the lines{} are empty.", pct, ext_info);
}

fn main() {
    let args = Args::parse();
    let mut totals_by_ext: HashMap<String, CodeInfo> = HashMap::new();
    let mut total = CodeInfo {
        lines: 0,
        blanks: 0,
    };

    for filename in RecDir::new(&args.dir) {
        let file = fs::File::open(&filename).expect("Error opening file.");
        let mut reader = BufReader::new(&file);
        let mut line = String::new();
        let ext = filename
            .extension()
            .and_then(OsStr::to_str)
            .unwrap_or("miscellaneous")
            .to_owned();

        let total = if args.aggregate {
            totals_by_ext.entry(ext).or_insert(CodeInfo {
                lines: 0,
                blanks: 0,
            })
        } else {
            &mut total
        };

        loop {
            match reader.read_line(&mut line) {
                Ok(bytes) => {
                    if bytes == 0 {
                        break;
                    }
                    if line.starts_with('\n') {
                        total.blanks += 1;
                    }
                    total.lines += 1;
                    line.clear();
                }
                Err(_) => break,
            }
        }
    }

    if args.aggregate {
        for pair in totals_by_ext.iter() {
            output(pair.1.to_owned(), Some(pair.0));
        }
    } else {
        output(total, None);
    }
}

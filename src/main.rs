use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::thread::JoinHandle;

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

fn output(info: CodeInfo, ext: Option<&str>) {
    let ext_info = ext.map_or(String::new(), |ext| format!(" in {} files", ext));
    let pct = 100. * info.blanks as f64 / info.lines as f64;
    println!("There are {} lines of code{}.", info.lines, ext_info);
    println!("There are {} empty lines{}.", info.blanks, ext_info);
    println!("{:.2}% of the lines{} are empty.", pct, ext_info);
}

fn main() {
    let args = Args::parse();
    let totals_by_ext: HashMap<String, CodeInfo> = HashMap::new();
    let totals_by_ext = Arc::new(Mutex::new(totals_by_ext));
    let total = Arc::new(Mutex::new(CodeInfo {
        lines: 0,
        blanks: 0,
    }));
    let misc_files = Arc::new(Mutex::new(false));
    let file_iterator = Arc::new(Mutex::new(RecDir::new(&args.dir)));
    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    for _ in 1..4 {
        let totals_by_ext = Arc::clone(&totals_by_ext);
        let total = Arc::clone(&total);
        let misc_files = Arc::clone(&misc_files);
        let file_iterator = Arc::clone(&file_iterator);
        handles.push(thread::spawn(move || {
            loop {
                let mut file_iterator_guard = file_iterator.lock().unwrap();
                let filename = file_iterator_guard.next();
                drop(file_iterator_guard);
                if filename.is_none() {
                    break;
                }
                let filename = filename.unwrap();
                let file = fs::File::open(&filename).expect("Error opening file.");
                let mut reader = BufReader::new(&file);
                let mut line = String::new();
                let ext = filename
                    .extension()
                    .and_then(OsStr::to_str)
                    .map(str::to_owned);

                let mut totals_by_ext_guard;
                let mut total_guard;
                let counter = if args.aggregate && ext.is_some() {
                    totals_by_ext_guard = totals_by_ext.lock().unwrap();
                    totals_by_ext_guard.entry(ext.unwrap()).or_insert(CodeInfo {
                        lines: 0,
                        blanks: 0,
                    })
                } else {
                    *misc_files.lock().unwrap() = false;
                    total_guard = total.lock().unwrap();
                    &mut total_guard
                };

                while let Ok(bytes) = reader.read_line(&mut line) {
                    if bytes == 0 {
                        break;
                    }
                    if bytes == 1 {
                        counter.blanks += 1;
                    }
                    counter.lines += 1;
                    line.clear();
                }
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let totals_by_ext = totals_by_ext.lock().unwrap();
    let total = *total.lock().unwrap();
    let misc_files = *misc_files.lock().unwrap();
    if args.aggregate {
        for pair in totals_by_ext.iter() {
            output(pair.1.to_owned(), Some(pair.0));
        }
        if misc_files {
            output(total, Some("miscellaneous"));
        }
    } else {
        output(total, None);
    }
}

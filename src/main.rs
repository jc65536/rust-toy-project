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
    /// Number of threads to use
    #[clap(short = 'j', long = "threads", default_value = "1")]
    threads: i32,
}

#[derive(Clone, Copy)]
struct CodeInfo {
    lines: i32,
    blanks: i32,
}

impl CodeInfo {
    fn new() -> Self {
        CodeInfo {
            lines: 0,
            blanks: 0,
        }
    }
    fn add(&mut self, o: Self) {
        self.lines += o.lines;
        self.blanks += o.blanks;
    }
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
    // total is used to count lines in all files normally, or lines in
    // miscellaneous files (files without extension) if -A is on
    let total = Arc::new(Mutex::new(CodeInfo {
        lines: 0,
        blanks: 0,
    }));
    let misc_files = Arc::new(Mutex::new(false));
    let file_iterator = Arc::new(Mutex::new(RecDir::new(&args.dir)));
    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    for _ in 0..args.threads {
        // Clone pointers so that each thread can own a pointer
        let totals_by_ext = Arc::clone(&totals_by_ext);
        let total = Arc::clone(&total);
        let misc_files = Arc::clone(&misc_files);
        let file_iterator = Arc::clone(&file_iterator);

        handles.push(thread::spawn(move || loop {
            // Loops until there are no files left
            let filename = match file_iterator.lock().unwrap().next() {
                Some(v) => v,
                None => {
                    return;
                }
            };
            let file = fs::File::open(&filename).expect("Error opening file.");
            let mut reader = BufReader::new(file);
            let mut line = String::new();
            let ext = filename
                .extension()
                .and_then(OsStr::to_str)
                .map(str::to_owned);
            let mut counter = CodeInfo::new();

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

            if args.aggregate && ext.is_some() {
                totals_by_ext
                    .lock()
                    .unwrap()
                    .entry(ext.unwrap())
                    .or_insert(CodeInfo::new())
                    .add(counter);
            } else {
                *misc_files.lock().unwrap() = true;
                total.lock().unwrap().add(counter);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let total = *total.lock().unwrap();
    if args.aggregate {
        for pair in totals_by_ext.lock().unwrap().iter() {
            output(pair.1.to_owned(), Some(pair.0));
        }
        if *misc_files.lock().unwrap() {
            output(total, Some("miscellaneous"));
        }
    } else {
        output(total, None);
    }
}

use std::env;
use std::fs;
use std::io::Read;
use std::io::BufReader;
use std::io::BufRead;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let file: Box<dyn Read> = if filename == "-" {
        println!("Reading from stdin.");
        Box::new(std::io::stdin())
    } else {
        println!("{}", filename);
        Box::new(fs::File::open(filename).expect("Error while opening file."))
    };

    let mut reader = BufReader::new(file);
    let mut line: String = String::new();
    let mut count = 0;

    loop {
        match reader.read_line(&mut line) {
            Ok(v) => {
                print!("{}", line);
                line = String::new();
                if v == 0 {
                    break;
                }
                count += 1;
            },
            Err(_) => break
        }
    }

    println!("====\nLines: {}", count);
}

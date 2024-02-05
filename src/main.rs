use std::io::{self, BufRead};
use std::env;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;


// The output is wrapped in a Result to allow matching on errors.
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn main() {
    let mut filename = "";
    let args: Vec<_> = env::args().collect();
    if args.len() == 2 {
        filename = &args[1];
    }

    let mut map: HashMap<String, String> = HashMap::new();

    if let Ok(lines) = read_lines(filename) {
        for line in lines.flatten() {
            let ln = line.trim();
            if let Some(i) = ln.find('\t') {
                let key = (&ln[..i]).to_string();
                let val = (&ln[i+1..]).to_string();
                map.insert(key, val);
            }
        }
    }

    let stdin = io::stdin();
    for line_data in stdin.lock().lines() {
        if let Ok(line) = line_data {
            let ln = line.trim();

            if let Some(val) = map.get(ln) {
                println!("{}\t{}", ln, val);
            } else {
                println!("{}", ln);
            }
        }
    }
}

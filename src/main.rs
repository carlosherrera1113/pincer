use std::{
    fs::File,
    io::{BufRead, BufReader},
};

fn main() {
    let file_path = "/Users/carlosherrera/Library/Logs/Wizards Of The Coast/MTGA/Player.log";

    let file = File::open(file_path).expect("Should open file.");
    let mut reader = BufReader::new(file);

    let mut line = String::new();

    loop {
        line.clear();
        let num_bytes = reader.read_line(&mut line).expect("read error");

        if num_bytes == 0 {
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        println!("{line}");
    }
}

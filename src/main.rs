use std::{
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Debug)]
enum Event {
    Raw { name: String, payload: String },
}

struct Parser {
    pending: Option<String>,
}

impl Parser {
    fn new() -> Self {
        Parser { pending: None }
    }

    fn push(&mut self, line: &str) -> Option<Event> {
        if let Some(name) = self.pending.take() {
            let payload = line.trim();
            if payload.starts_with('{') {
                return Some(Event::Raw {
                    name,
                    payload: payload.to_string(),
                });
            }
        }

        let message = line.strip_prefix("[UnityCrossThreadLogger]")?;

        let (header, payload) = match message.find('{') {
            Some(byte_index) => (&message[..byte_index], message[byte_index..].to_string()),
            None => (message, String::new()),
        };

        let name = if let Some(arrow) = header.find("==>").or_else(|| header.find("<==")) {
            let after_arrow = header[arrow + 3..].trim();
            let name_end = after_arrow.find('(').unwrap_or(after_arrow.len());
            after_arrow[..name_end].trim().to_string()
        } else if header.contains("Match to") {
            self.pending = Some(header.rsplit(":").next()?.trim().to_string());
            return None;
        } else {
            return None;
        };

        Some(Event::Raw { name, payload })
    }
}

fn follow(file_path: &str) {
    let file = File::open(file_path).expect("Should open file.");
    let mut reader = BufReader::new(file);

    let mut line = String::new();
    let mut parser = Parser::new();

    loop {
        line.clear();
        let num_bytes = reader.read_line(&mut line).expect("read error");

        if num_bytes == 0 {
            std::thread::sleep(std::time::Duration::from_millis(500));
            continue;
        }

        if let Some(event) = parser.push(&line) {
            match event {
                Event::Raw { name, payload } => println!("{name}: {payload}"),
            }
        }
    }
}

fn main() {
    let file_path = "/Users/carlosherrera/Library/Logs/Wizards Of The Coast/MTGA/Player.log";

    follow(file_path);
}

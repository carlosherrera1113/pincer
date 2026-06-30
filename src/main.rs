use std::{
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Debug)]
enum Event {
    Raw { name: String, payload: String },
}

enum State {
    Idle,
    AwaitingArrow,
    AwaitingBody { name: String, body: String },
}

struct Parser {
    state: State,
}

impl Parser {
    fn new() -> Self {
        Parser { state: State::Idle }
    }

    fn push(&mut self, line: &str) -> Option<Event> {
        match std::mem::replace(&mut self.state, State::Idle) {
            State::AwaitingBody { name, mut body } => {
                body.push_str(&line);
                if json_complete(&body) {
                    return Some(Event::Raw {
                        name,
                        payload: body.trim().to_string(),
                    });
                }
                self.state = State::AwaitingBody { name, body };
                return None;
            }

            State::AwaitingArrow => {
                if let Some(after) = line.trim().strip_prefix("<==") {
                    let name_end = after.find('(').unwrap_or(after.len());
                    self.state = State::AwaitingBody {
                        name: after[..name_end].trim().to_string(),
                        body: String::new(),
                    }
                };
                return None;
            }
            State::Idle => {}
        }

        let message = line.strip_prefix("[UnityCrossThreadLogger]")?;

        let (header, payload) = match message.find('{') {
            Some(byte_index) => (&message[..byte_index], message[byte_index..].to_string()),
            None => (message, String::new()),
        };

        let name = if let Some(arrow) = header.find("==>") {
            let after_arrow = header[arrow + 3..].trim();
            let name_end = after_arrow.find('(').unwrap_or(after_arrow.len());
            after_arrow[..name_end].trim().to_string()
        } else if header.contains("Match to") {
            self.state = State::AwaitingBody {
                name: header.rsplit(":").next()?.trim().to_string(),
                body: String::new(),
            };
            return None;
        } else {
            if payload.is_empty() {
                self.state = State::AwaitingArrow;
            }
            return None;
        };

        Some(Event::Raw { name, payload })
    }
}

fn json_complete(s: &str) -> bool {
    let mut depth = 0;
    let mut opened = false;
    let mut in_string = false;
    let mut escaped = false;

    for ch in s.chars() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
        } else {
            match ch {
                '"' => in_string = true,
                '{' => {
                    depth += 1;
                    opened = true;
                }
                '}' => depth -= 1,
                _ => {}
            }
        }
    }

    opened && depth == 0 && !in_string
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

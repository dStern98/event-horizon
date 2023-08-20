use anyhow::Result;
use serde_json;
use std::io::{self, BufRead};
mod body;
mod message;
use message::{node_init, MaelstromMessage};

fn main() {
    let (node_id, _) = node_init();

    let stdin_handle = io::stdin().lock();
    let mut stdout_handle = io::stdout().lock();
    for line in stdin_handle.lines() {
        let line = line.expect("Nothing recieved via stdin...");
        let message: Result<MaelstromMessage, _> = serde_json::from_str(&line);
        match message {
            Ok(maelstrom_inbound) => maelstrom_inbound.reply(&mut stdout_handle, &node_id),
            Err(_) => eprintln!("Received the following invalid maelstrom message: {}", line),
        }
    }
}

use serde_json;
use std::io::{self, BufRead};
mod body;
mod init;
mod message;
use body::Body;
use init::NodeMetadata;
use message::MaelstromMessage;
use std::io::Write;
use std::sync::mpsc;
use std::thread;

fn main() {
    let mut node_metadata = NodeMetadata::node_init();
    let mut stdout_handle = io::stdout().lock();

    let (mut tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let stdin_handle = io::stdin().lock();
        for line in stdin_handle.lines() {
            let line = line.expect("Nothing recieved via stdin...");
            let message: Result<MaelstromMessage, _> = serde_json::from_str(&line);
            match message {
                Ok(maelstrom_inbound) => {
                    maelstrom_inbound.reply(&mut tx, &mut node_metadata);
                }
                Err(_) => eprintln!("Received the following invalid maelstrom message: {}", line),
            }
        }
    });

    while let Ok(maelstrom_message) = rx.recv() {
        serde_json::to_writer(&mut stdout_handle, &maelstrom_message)
            .expect("Unable to serialize to writer.");
        //Maelstrom requires a new line character
        stdout_handle
            .write_all(b"\n")
            .expect("Unable to write newline character");
    }
}

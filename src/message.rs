use super::body::Body;
use serde::{self, Deserialize, Serialize};
use std::io::{self, BufRead, Write};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MaelstromMessage {
    src: String,
    dest: String,
    body: Body,
}

impl MaelstromMessage {
    pub fn reply(self, mut stdout_handle: &mut io::StdoutLock, message_id: &String) {
        //! For a given MaelStromMessage, Build and send a reply
        //! if one exists.
        let reply_body = self.body.into_reply(&message_id);
        if let Some(reply_body) = reply_body {
            let maelstrom_reply = MaelstromMessage {
                src: self.dest,
                dest: self.src,
                body: reply_body,
            };
            serde_json::to_writer(&mut stdout_handle, &maelstrom_reply)
                .expect("Unable to serialize to writer.");
            //Maelstrom requires a new line character
            stdout_handle
                .write_all(b"\n")
                .expect("Unable to write newline character");
        }
    }
}

pub fn node_init() -> (String, Vec<String>) {
    //!Get the Node Init Message from stdin, returning
    //! the node_id and the node_ids.
    let stdin_handle = io::stdin().lock();
    let first_line = stdin_handle
        .lines()
        .next()
        .expect("lines Iterator was empty")
        .expect("Improper Init...");
    let init_message: MaelstromMessage =
        serde_json::from_str(&first_line).expect("Could not deserialize Init JSON");

    let node_info: (String, Vec<String>);
    if let Body::Init {
        node_id, node_ids, ..
    } = &init_message.body
    {
        node_info = (node_id.clone(), node_ids.clone());
    } else {
        panic!("First maelstrom message recieved was not an Init.")
    }
    let mut stdout_handle = std::io::stdout().lock();
    init_message.reply(&mut stdout_handle, &node_info.0);
    return node_info;
}

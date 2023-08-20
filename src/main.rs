use anyhow::Result;
use serde::{self, Deserialize, Serialize};
use serde_json;
use std::io::{self, BufRead, Write};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Body {
    Init {
        msg_id: usize,
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk {
        in_reply_to: usize,
    },
    Echo {
        msg_id: usize,
        echo: String,
    },
    EchoOk {
        msg_id: usize,
        in_reply_to: usize,
        echo: String,
    },
    Generate {
        msg_id: usize,
    },
    GenerateOk {
        id: String,
        msg_id: usize,
        in_reply_to: usize,
    },
}

impl Body {
    fn into_reply(self, node_id: &String) -> Option<Body> {
        //!Consumes self, returns a Some(reply_body)
        //! if one exists.
        match self {
            Body::Init { msg_id, .. } => Some(Body::InitOk {
                in_reply_to: msg_id,
            }),
            Body::Echo { msg_id, echo } => Some(Body::EchoOk {
                msg_id,
                in_reply_to: msg_id,
                echo,
            }),
            Body::Generate { msg_id } => {
                //Because node_id is unique for a given node, and
                //Message IDs are unique per source node
                let unique_id = format!("{}|{}", node_id, msg_id);
                Some(Body::GenerateOk {
                    id: unique_id,
                    msg_id,
                    in_reply_to: msg_id,
                })
            }
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct MaelstromMessage {
    src: String,
    dest: String,
    body: Body,
}

impl MaelstromMessage {
    fn reply(self, mut stdout_handle: &mut io::StdoutLock, message_id: &String) {
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
                .expect("Unable to serialize to write.");
            stdout_handle.write_all(b"\n").unwrap();
        }
    }
}

fn node_init() -> (String, Vec<String>) {
    //!Get the Node Init Message from stdin, returning
    //! the node_id and the node_ids.
    let stdin_handle = io::stdin();
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

fn main() {
    let (node_id, _) = node_init();

    let stdin_handle = io::stdin().lock();
    let mut stdout_handle = io::stdout().lock();
    for line in stdin_handle.lines() {
        let line = line.expect("Nothing recieved via stdin...");
        let message: Result<MaelstromMessage, _> = serde_json::from_str(&line);
        match message {
            Ok(maelstrom_inbound) => maelstrom_inbound.reply(&mut stdout_handle, &node_id),
            Err(_) => eprintln!("Received the following invalid inbound: {}", line),
        }
    }
}

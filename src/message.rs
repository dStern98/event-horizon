use super::body::Body;
use serde::{self, Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::{self, BufRead, Write};

#[derive(Debug, Clone)]
pub struct NodeMetadata {
    pub node_id: String,
    pub node_ids: Vec<String>,
    pub seen_message_ids: HashSet<String>,
    pub kv_store: HashMap<usize, usize>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MaelstromMessage {
    src: String,
    dest: String,
    body: Body,
}

impl MaelstromMessage {
    pub fn reply(
        self,
        mut stdout_handle: &mut io::StdoutLock,
        mut node_metadata: &mut NodeMetadata,
    ) {
        //! For a given MaelStromMessage, Build and send a reply
        //! if one exists.
        let reply_body = self.body.into_reply(&mut node_metadata);
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

pub fn node_init() -> NodeMetadata {
    //!Get the Node Init Message from stdin, returning
    //! the node metadata.
    let stdin_handle = io::stdin().lock();
    let first_line = stdin_handle
        .lines()
        .next()
        .expect("lines Iterator was empty")
        .expect("Improper Init...");
    let init_message: MaelstromMessage =
        serde_json::from_str(&first_line).expect("Could not deserialize Init JSON");

    let mut node_metadata: NodeMetadata;
    if let Body::Init {
        node_id, node_ids, ..
    } = &init_message.body
    {
        node_metadata = NodeMetadata {
            node_id: node_id.clone(),
            node_ids: node_ids.clone(),
            seen_message_ids: HashSet::new(),
            kv_store: HashMap::new(),
        }
    } else {
        panic!("First maelstrom message recieved was not an Init.")
    }

    let mut stdout_handle = std::io::stdout().lock();
    init_message.reply(&mut stdout_handle, &mut node_metadata);
    return node_metadata;
}

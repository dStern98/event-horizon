use super::{Body, MaelstromMessage};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};

#[derive(Debug, Clone)]
pub struct NodeMetadata {
    pub node_id: String,
    pub current_msg_id: usize,
    //Note that the node_ids DOES NOT
    //INCLUDE this nodes node_id
    pub node_ids: Vec<String>,
    //The kv_store as a HashMap
    pub kv_store: HashMap<usize, usize>,
}

impl NodeMetadata {
    pub fn node_init() -> Self {
        //!Get the Node Init Message from stdin, reply to the init, and return
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
                node_ids: node_ids
                    .iter()
                    .filter(|&item| item != node_id)
                    .map(|item| item.clone())
                    .collect(),
                current_msg_id: 0,
                kv_store: HashMap::new(),
            }
        } else {
            panic!("First maelstrom message recieved was not an Init.")
        }

        let mut stdout_handle = std::io::stdout().lock();
        let init_reply = MaelstromMessage {
            src: init_message.dest,
            dest: init_message.src,
            body: init_message
                .body
                .into_reply(&mut node_metadata)
                .expect("Into Reply failed to Return InitOK."),
        };
        serde_json::to_writer(&mut stdout_handle, &init_reply)
            .expect("Unable to serialize to writer.");
        //Maelstrom requires a new line character
        stdout_handle
            .write_all(b"\n")
            .expect("Unable to write newline character");
        node_metadata
    }
}

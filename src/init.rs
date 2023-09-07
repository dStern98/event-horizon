use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};

#[derive(Debug, Clone)]
pub struct NodeMetadata {
    //This nodes node_id
    pub node_id: String,
    //The rest of the node_ids
    pub node_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum InitBody {
    Init {
        msg_id: usize,
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk {
        in_reply_to: usize,
    },
}

impl InitBody {
    fn into_reply(self) -> Self {
        match self {
            InitBody::Init { msg_id, .. } => InitBody::InitOk {
                in_reply_to: msg_id,
            },
            _ => panic!("First message was not an init!"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MaelstromInit {
    pub src: String,
    pub dest: String,
    pub body: InitBody,
}

impl MaelstromInit {
    fn read_stdin() -> Self {
        //!Get the Node Init Message from stdin, reply to the init, and return
        //! the node metadata.
        let stdin_handle = io::stdin().lock();
        let first_line = stdin_handle
            .lines()
            .next()
            .expect("lines Iterator was empty")
            .expect("Improper Init...");
        serde_json::from_str(&first_line).expect("Could not deserialize Init JSON")
    }

    pub fn init_node() -> NodeMetadata {
        //Read from stdin to receive the Init from Maelstrom,
        //reply InitOk. Construct and return a struct containing
        //the key metadata for the node runtime
        let init_message = MaelstromInit::read_stdin();
        let init_copy = init_message.clone();

        //Reply to the MaelStromInit message.
        let init_reply = MaelstromInit {
            src: init_message.dest,
            dest: init_message.src,
            body: init_message.body.into_reply(),
        };
        let mut stdout_handle = std::io::stdout().lock();
        serde_json::to_writer(&mut stdout_handle, &init_reply)
            .expect("Unable to serialize to writer.");
        //Maelstrom requires a new line character
        stdout_handle
            .write_all(b"\n")
            .expect("Unable to write newline character");

        //Construct a NodeMetadata from the cloned Init.
        let node_metadata: NodeMetadata;
        if let InitBody::Init {
            node_id, node_ids, ..
        } = init_copy.body
        {
            node_metadata = NodeMetadata {
                node_id: node_id,
                node_ids: node_ids,
            };
            return node_metadata;
        } else {
            panic!("First maelstrom message recieved was not an Init.")
        }
    }
}

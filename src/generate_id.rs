use super::{Event, Node, NodeMetadata, Reply};
use serde::{self, Deserialize, Serialize};
use std::io;
use std::sync::mpsc::Sender;

pub struct GenerateGuidNode {
    pub current_msg_id: usize,
    pub node_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum GenerateGuidBody {
    Generate {
        msg_id: usize,
    },
    GenerateOk {
        id: String,
        msg_id: usize,
        in_reply_to: usize,
    },
}

impl Node<GenerateGuidBody> for GenerateGuidNode {
    fn node_init(node_metadata: NodeMetadata, _: Sender<Event<GenerateGuidBody>>) -> Self {
        GenerateGuidNode {
            current_msg_id: 0,
            node_id: node_metadata.node_id,
        }
    }

    fn handle_event(&mut self, event: Event<GenerateGuidBody>, stdout_handle: &mut io::StdoutLock) {
        match event {
            Event::Message(message) => {
                message.message_reply(stdout_handle, self);
                self.current_msg_id += 1;
            }
            _ => {}
        }
    }
}

impl Reply<GenerateGuidNode> for GenerateGuidBody {
    fn into_reply(self, node_state: &mut GenerateGuidNode) -> Option<Self> {
        match self {
            GenerateGuidBody::Generate { msg_id } => {
                //Because node_id is unique for a given node, and
                //Message IDs are unique per source node
                let unique_id = format!("{}|{}", &node_state.node_id, node_state.current_msg_id);
                Some(GenerateGuidBody::GenerateOk {
                    id: unique_id,
                    msg_id: node_state.current_msg_id,
                    in_reply_to: msg_id,
                })
            }
            _ => None,
        }
    }
}

use super::{MaelstromReply, NodeMetadata};
use serde::{self, Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum GenerateBody {
    Generate {
        msg_id: usize,
    },
    GenerateOk {
        id: String,
        msg_id: usize,
        in_reply_to: usize,
    },
}

impl MaelstromReply for GenerateBody {
    fn into_reply(self, node_metadata: &mut NodeMetadata) -> Option<Self> {
        match self {
            GenerateBody::Generate { msg_id } => {
                //Because node_id is unique for a given node, and
                //Message IDs are unique per source node
                let unique_id = format!(
                    "{}|{}",
                    &node_metadata.node_id, node_metadata.current_msg_id
                );
                Some(GenerateBody::GenerateOk {
                    id: unique_id,
                    msg_id: node_metadata.current_msg_id,
                    in_reply_to: msg_id,
                })
            }
            _ => None,
        }
    }
}

use super::{MaelstromReply, NodeMetadata};
use serde::{self, Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum EchoBody {
    Echo {
        msg_id: usize,
        echo: String,
    },
    EchoOk {
        msg_id: usize,
        in_reply_to: usize,
        echo: String,
    },
}

impl MaelstromReply for EchoBody {
    fn into_reply(self, node_metadata: &mut NodeMetadata) -> Option<EchoBody> {
        match self {
            EchoBody::Echo { msg_id, echo, .. } => Some(EchoBody::EchoOk {
                msg_id: node_metadata.current_msg_id,
                in_reply_to: msg_id,
                echo,
            }),
            _ => None,
        }
    }
}

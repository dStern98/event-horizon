use super::{Event, Node, NodeMetadata, Reply};
use serde::{self, Deserialize, Serialize};
use std::io;
use std::sync::mpsc::Sender;

pub struct EchoNode {
    pub current_msg_id: usize,
}

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

impl Reply<EchoNode> for EchoBody {
    fn into_reply(self, echo_node: &mut EchoNode) -> Option<EchoBody> {
        match self {
            EchoBody::Echo { msg_id, echo, .. } => Some(EchoBody::EchoOk {
                msg_id: echo_node.current_msg_id,
                in_reply_to: msg_id,
                echo,
            }),
            _ => None,
        }
    }
}

impl Node<EchoBody> for EchoNode {
    fn node_init(node_metadata: NodeMetadata, _: Sender<Event<EchoBody>>) -> Self {
        let _ = node_metadata;
        EchoNode { current_msg_id: 0 }
    }

    fn handle_event(&mut self, event: Event<EchoBody>, stdout_handle: &mut io::StdoutLock) {
        match event {
            Event::Message(message) => {
                message.message_reply(stdout_handle, self);
                self.current_msg_id += 1;
            }
            _ => {}
        }
    }
}

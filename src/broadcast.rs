use super::{Event, Node, Reply};
use serde::{self, Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io;
use std::sync::mpsc::Sender;

pub struct BroadcastNode {
    pub node_id: String,
    pub other_node_ids: Vec<String>,
    pub current_msg_id: usize,
    pub messages: HashSet<usize>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum BroadcastBody {
    Broadcast {
        msg_id: usize,
        message: usize,
    },
    BroadcastOk {
        msg_id: usize,
        in_reply_to: usize,
    },
    Read {
        msg_id: usize,
    },
    ReadOk {
        msg_id: usize,
        in_reply_to: usize,
        messages: Vec<usize>,
    },
    Topology {
        msg_id: usize,
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk {
        in_reply_to: usize,
        msg_id: usize,
    },
}

impl Reply<BroadcastNode> for BroadcastBody {
    fn into_reply(self, node_state: &mut BroadcastNode) -> Option<Self> {
        match self {
            BroadcastBody::Topology { msg_id, .. } => Some(BroadcastBody::TopologyOk {
                in_reply_to: msg_id,
                msg_id: node_state.current_msg_id,
            }),
            BroadcastBody::Broadcast { msg_id, message } => {
                node_state.messages.insert(message);
                Some(BroadcastBody::BroadcastOk {
                    msg_id: node_state.current_msg_id,
                    in_reply_to: msg_id,
                })
            }
            BroadcastBody::Read { msg_id } => {
                let seen_messages: Vec<_> = node_state.messages.clone().into_iter().collect();
                Some(BroadcastBody::ReadOk {
                    msg_id: node_state.current_msg_id,
                    in_reply_to: msg_id,
                    messages: seen_messages,
                })
            }
            _ => None,
        }
    }
}

impl Node<BroadcastBody> for BroadcastNode {
    fn node_init(
        node_metadata: crate::init::NodeMetadata,
        event_tx: Sender<Event<BroadcastBody>>,
    ) -> Self {
        let _ = event_tx;
        let other_node_ids: Vec<_> = node_metadata
            .node_ids
            .into_iter()
            .filter(|node_id| node_id != &node_metadata.node_id)
            .collect();
        BroadcastNode {
            current_msg_id: 0,
            node_id: node_metadata.node_id,
            messages: HashSet::new(),
            other_node_ids,
        }
    }
    fn handle_event(&mut self, event: Event<BroadcastBody>, stdout_handle: &mut io::StdoutLock)
    where
        BroadcastBody: Reply<Self>,
        Self: Sized,
    {
        match event {
            Event::Message(message) => message.message_reply(stdout_handle, self),
            _ => {}
        }
    }
}

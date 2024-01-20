use super::{Event, Node, Reply};
use serde::{self, Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use std::sync::mpsc::Sender;

pub struct KafkaNode {
    pub node_id: String,
    pub current_message_id: usize,
    pub committed_offsets: HashMap<String, usize>,
    pub messages: HashMap<String, Vec<usize>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum KafkaBody {
    Send {
        msg_id: usize,
        key: String,
        msg: usize,
    },
    SendOk {
        offset: usize,
        in_reply_to: usize,
        msg_id: usize,
    },
    Poll {
        msg_id: usize,
        offsets: HashMap<String, usize>,
    },
    PollOk {
        msgs: HashMap<String, Vec<(usize, usize)>>,
        in_reply_to: usize,
        msg_id: usize,
    },
    CommitOffsets {
        msg_id: usize,
        offsets: HashMap<String, usize>,
    },
    CommitOffsetsOk {
        msg_id: usize,
        in_reply_to: usize,
    },
    ListCommittedOffsets {
        msg_id: usize,
        keys: Vec<String>,
    },
    ListCommittedOffsetsOk {
        in_reply_to: usize,
        msg_id: usize,
        offsets: HashMap<String, usize>,
    },
}

impl Reply<KafkaNode> for KafkaBody {
    fn into_reply(self, node_state: &mut KafkaNode, _: &String) -> Option<Self> {
        match self {
            KafkaBody::Send { key, msg, msg_id } => {
                let messages = node_state.messages.entry(key).or_default();
                messages.push(msg);
                //Currently, offsets start at 0 and increment by 1.
                Some(KafkaBody::SendOk {
                    offset: messages.len() - 1,
                    in_reply_to: msg_id,
                    msg_id: node_state.current_message_id,
                })
            }
            KafkaBody::Poll { msg_id, offsets } => {
                let mut poll_response = HashMap::new();
                for (key, mut offset) in offsets.into_iter() {
                    //If the key is already in the messages HashMap,
                    //add up to 10 items to the poll response.
                    if let Some(messages) = node_state.messages.get(&key) {
                        let mut tuple_messages = Vec::new();
                        while let Some(item) = messages.get(offset) {
                            if tuple_messages.len() >= 10 {
                                break;
                            }
                            tuple_messages.push((offset, *item));
                            offset += 1;
                        }
                        poll_response.insert(key, tuple_messages);
                    }
                }
                Some(KafkaBody::PollOk {
                    msgs: poll_response,
                    in_reply_to: msg_id,
                    msg_id: node_state.current_message_id,
                })
            }
            KafkaBody::CommitOffsets { msg_id, offsets } => {
                offsets
                    .into_iter()
                    .map(|(key, offset)| node_state.committed_offsets.insert(key, offset))
                    .count();

                Some(KafkaBody::CommitOffsetsOk {
                    msg_id: node_state.current_message_id,
                    in_reply_to: msg_id,
                })
            }

            KafkaBody::ListCommittedOffsets { msg_id, keys } => {
                let mut committed_offsets = HashMap::new();
                keys.into_iter()
                    .map(|key| {
                        if let Some(offset) = node_state.committed_offsets.get(&key) {
                            committed_offsets.insert(key, *offset);
                        }
                    })
                    .count();
                Some(KafkaBody::ListCommittedOffsetsOk {
                    in_reply_to: msg_id,
                    msg_id: node_state.current_message_id,
                    offsets: committed_offsets,
                })
            }

            _ => None,
        }
    }
}

impl Node<KafkaBody> for KafkaNode {
    fn node_init(
        node_metadata: crate::init::NodeMetadata,
        event_tx: Sender<Event<KafkaBody>>,
    ) -> Self {
        let _ = event_tx;
        KafkaNode {
            node_id: node_metadata.node_id,
            current_message_id: 0,
            committed_offsets: HashMap::new(),
            messages: HashMap::new(),
        }
    }

    fn handle_event(&mut self, event: Event<KafkaBody>, stdout_handle: &mut io::StdoutLock)
    where
        KafkaBody: Reply<Self>,
        Self: Sized,
    {
        if let Event::Message(message) = event {
            message.message_reply(stdout_handle, self);
            self.current_message_id += 1;
        }
    }
}

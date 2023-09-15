use crate::node::MaelstromMessage;

use super::{Event, Node, Reply};
use serde::{self, Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::Sender;
use std::{io, thread};

pub struct BroadcastNode {
    pub node_id: String,
    pub neighbors: Vec<String>,
    pub other_node_ids: Vec<String>,
    pub current_msg_id: usize,
    pub messages: HashSet<usize>,
    //Stores a Mapping of Node ID to messages we know the other node
    //has seen
    pub confirmed_seen: HashMap<String, HashSet<usize>>,
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
    Gossip {
        msg_id: usize,
        message: Vec<usize>,
    },
    GossipOk {
        in_reply_to: usize,
        msg_id: usize,
        ack_message: Vec<usize>,
    },
}

impl Reply<BroadcastNode> for BroadcastBody {
    fn into_reply(self, node_state: &mut BroadcastNode, src: &String) -> Option<Self> {
        match self {
            BroadcastBody::Topology {
                msg_id,
                mut topology,
            } => {
                let (_, neighbors) = match topology.entry(node_state.node_id.clone()) {
                    Entry::Occupied(entry) => entry.remove_entry(),
                    Entry::Vacant(_) => {
                        panic!("This nodes node_id was not in topology map");
                    }
                };
                node_state.neighbors = neighbors;
                Some(BroadcastBody::TopologyOk {
                    in_reply_to: msg_id,
                    msg_id: node_state.current_msg_id,
                })
            }
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
            BroadcastBody::Gossip { msg_id, message } => {
                //When recieving a Gossip message, first add the gossip messages to
                // the messages Hashset.
                node_state.messages.extend(message.clone().into_iter());
                //Then, add the Messages to the confirmed seen HashMap with
                //key=src node. (If a node sent you a message, if must already have seen those messages).
                node_state
                    .confirmed_seen
                    .entry(src.clone())
                    .or_insert(HashSet::new())
                    .extend(message.clone().into_iter());
                Some(BroadcastBody::GossipOk {
                    in_reply_to: msg_id,
                    msg_id: node_state.current_msg_id,
                    ack_message: message,
                })
            }

            BroadcastBody::GossipOk { ack_message, .. } => {
                // If recieving a GossipOk, add the ack_messages to
                //the confirmed seen HashMap for the src Node (if they acked,
                //then we know they recieved these messages
                node_state
                    .confirmed_seen
                    .entry(src.clone())
                    .or_insert(HashSet::new())
                    .extend(ack_message);
                None
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
        thread::spawn(move || loop {
            thread::sleep(std::time::Duration::from_millis(250));
            event_tx
                .send(Event::PropogateWrites)
                .expect("Unable to drop PropogateWrites down the channel");
        });
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
            neighbors: Vec::new(),
            confirmed_seen: HashMap::new(),
        }
    }
    fn handle_event(&mut self, event: Event<BroadcastBody>, stdout_handle: &mut io::StdoutLock)
    where
        BroadcastBody: Reply<Self>,
        Self: Sized,
    {
        match event {
            Event::Message(message) => {
                message.message_reply(stdout_handle, self);
                self.current_msg_id += 1;
            }
            Event::PropogateWrites => {
                // In the event of a PropogateWrites Event, for each Neightbor, either Gossip
                // the difference between the Current Messages and the Confirmed seen for the specified
                //Node or send all messages.
                for neighbor in self.neighbors.iter() {
                    let message: Vec<usize> = match self.confirmed_seen.get(neighbor) {
                        Some(known) => self
                            .messages
                            .difference(known)
                            .map(|item| item.clone())
                            .collect(),
                        None => self.messages.clone().into_iter().collect(),
                    };
                    if message.len() > 0 {
                        let mut maelstrom_message = MaelstromMessage {
                            src: self.node_id.clone(),
                            dest: neighbor.clone(),
                            body: BroadcastBody::Gossip {
                                msg_id: self.current_msg_id,
                                message,
                            },
                        };
                        maelstrom_message.send(stdout_handle);
                        self.current_msg_id += 1;
                    }
                }
            }
        }
    }
}

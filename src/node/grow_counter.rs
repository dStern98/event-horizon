use crate::{Event, MaelstromMessage, Node, Reply};
use serde::{self, Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io;
use std::sync::mpsc::Sender;
use std::thread;

pub struct CounterNode {
    pub node_id: String,
    pub other_node_ids: Vec<String>,
    pub current_msg_id: usize,
    //A local copy of the current Nodes and their counter value
    pub node_counter_map: HashMap<String, usize>,
}

impl Node<CounterBody> for CounterNode {
    fn node_init(
        node_metadata: crate::init::NodeMetadata,
        event_tx: Sender<Event<CounterBody>>,
    ) -> Self {
        let other_node_ids: Vec<_> = node_metadata
            .node_ids
            .into_iter()
            .filter(|node_id| node_id != &node_metadata.node_id)
            .collect();

        thread::spawn(move || loop {
            //Every few seconds, send a copy of your nodes
            //counter values to the other nodes.
            thread::sleep(std::time::Duration::from_secs(1));
            event_tx
                .send(Event::PropogateWrites)
                .expect("Write Propogrator Thread failed to transmit");
        });

        CounterNode {
            node_id: node_metadata.node_id,
            current_msg_id: 0,
            other_node_ids,
            node_counter_map: HashMap::new(),
        }
    }

    fn handle_event(&mut self, event: Event<CounterBody>, stdout_handle: &mut io::StdoutLock)
    where
        CounterBody: Reply<Self>,
        Self: Sized,
    {
        match event {
            Event::Message(message) => {
                message.message_reply(stdout_handle, self);
                self.current_msg_id += 1;
            }
            Event::PropogateWrites => {
                //When a PropogateWrite is triggered, send a copy
                //of this Nodes counter_map to every other node.
                for other_node in self.other_node_ids.iter() {
                    let mut counter_value = MaelstromMessage {
                        src: self.node_id.clone(),
                        dest: other_node.clone(),
                        body: CounterBody::UpdateCounters {
                            msg_id: self.current_msg_id,
                            node_counter_map: self.node_counter_map.clone(),
                        },
                    };
                    counter_value.send(stdout_handle);
                    self.current_msg_id += 1;
                }
            }
        }
    }
}

#[derive(Serialize, Debug, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum CounterBody {
    Add {
        msg_id: usize,
        delta: usize,
    },
    AddOk {
        msg_id: usize,
        in_reply_to: usize,
    },
    Read {
        msg_id: usize,
    },
    ReadOk {
        msg_id: usize,
        in_reply_to: usize,
        value: usize,
    },
    UpdateCounters {
        msg_id: usize,
        node_counter_map: HashMap<String, usize>,
    },
}

impl Reply<CounterNode> for CounterBody {
    fn into_reply(self, node_state: &mut CounterNode, _: &str) -> Option<Self> {
        match self {
            CounterBody::Add { msg_id, delta } => {
                *node_state
                    .node_counter_map
                    .entry(node_state.node_id.clone())
                    .or_insert(0) += delta;
                Some(CounterBody::AddOk {
                    msg_id: node_state.current_msg_id,
                    in_reply_to: msg_id,
                })
            }
            CounterBody::Read { msg_id } => {
                //When asked to read the current value, return the sum of all values
                // in the HashMap
                let current_sum: usize = node_state.node_counter_map.values().sum();
                Some(CounterBody::ReadOk {
                    msg_id: node_state.current_msg_id,
                    in_reply_to: msg_id,
                    value: current_sum,
                })
            }
            CounterBody::UpdateCounters {
                node_counter_map, ..
            } => {
                for (node_id, counter_value) in node_counter_map.into_iter() {
                    //When recieving a UpdateCounters message from another Node,
                    //compare this Node's value for a given NodeID to the UpdateCounters value.
                    match node_state.node_counter_map.entry(node_id) {
                        Entry::Occupied(mut occ_entry) => {
                            let current_value = occ_entry.get_mut();
                            if &counter_value > current_value {
                                //If another node has a greater value for a key,
                                //set this Node's value to that (greater) value.
                                *current_value = counter_value;
                            }
                        }
                        Entry::Vacant(vac_entry) => {
                            vac_entry.insert(counter_value);
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }
}

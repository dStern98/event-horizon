use crate::MaelstromMessage;

use super::{Event, Node, NodeMetadata, Reply};
use serde::{self, Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use std::sync::mpsc::Sender;
use std::thread;

pub struct KVStoreNode {
    pub current_msg_id: usize,
    pub node_id: String,
    pub other_node_ids: Vec<String>,
    pub kv_store: HashMap<usize, usize>,
    pub unpropogated_writes: HashMap<usize, usize>,
}

impl Node<KVStoreBody> for KVStoreNode {
    fn node_init(node_metadata: NodeMetadata, event_tx: Sender<Event<KVStoreBody>>) -> Self {
        let other_node_ids: Vec<_> = node_metadata
            .node_ids
            .iter()
            .filter(|&node_id| node_id != &node_metadata.node_id)
            .map(|node_id| node_id.clone())
            .collect();

        thread::spawn(move || loop {
            thread::sleep(std::time::Duration::from_millis(10));
            event_tx
                .send(Event::PropogateWrites)
                .expect("Write Propogrator Thread failed to transmit");
        });

        KVStoreNode {
            current_msg_id: 0,
            node_id: node_metadata.node_id,
            other_node_ids,
            kv_store: HashMap::new(),
            unpropogated_writes: HashMap::new(),
        }
    }

    fn handle_event(&mut self, event: Event<KVStoreBody>, mut stdout_handle: &mut io::StdoutLock)
    where
        KVStoreBody: Reply<Self>,
        Self: Sized,
    {
        match event {
            Event::Message(message) => {
                message.message_reply(stdout_handle, self);
                self.current_msg_id += 1;
            }
            Event::PropogateWrites => {
                let unpropogated_writes: HashMap<usize, usize> =
                    self.unpropogated_writes.drain().collect();
                if unpropogated_writes.len() != 0 {
                    for other_node in self.other_node_ids.iter() {
                        let mut kv_writes = MaelstromMessage {
                            src: self.node_id.clone(),
                            dest: other_node.clone(),
                            body: KVStoreBody::WritePropogater {
                                transaction_guid: self.current_msg_id,
                                write_ops: unpropogated_writes.clone(),
                            },
                        };
                        kv_writes.send(&mut stdout_handle);
                    }
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum KVStoreBody {
    Txn {
        msg_id: usize,
        txn: Vec<(String, usize, Option<usize>)>,
    },
    TxnOk {
        msg_id: usize,
        in_reply_to: usize,
        txn: Vec<(String, usize, Option<usize>)>,
    },
    //WritePropogator is a node-to-node
    //message passing k-v writes. The write_ops
    //HashMap contains the k-v pairs to write.
    WritePropogater {
        transaction_guid: usize,
        write_ops: HashMap<usize, usize>,
    },
}

fn key_value_crud(
    rw_op: (String, usize, Option<usize>),
    kv_store: &mut HashMap<usize, usize>,
    unpropogated_writes: &mut HashMap<usize, usize>,
) -> (String, usize, Option<usize>) {
    //! Given a tuple of (operation, key, Option<value>), perform
    //! the indicated operation on the passed in HashMap. If operation is 'r',
    //! return the value associated with the key if it exists.
    //! If the operation is 'w', write the value to the associated key.

    let (operation, key, value) = rw_op;

    if operation == "r" {
        let read_value = kv_store.get(&key);
        return (operation, key, read_value.map(|value| value.clone()));
    } else if operation == "w" {
        let (key, value) = (
            key,
            value.expect("Recieved a Write Op without a value to write."),
        );
        kv_store.insert(key.clone(), value.clone());
        unpropogated_writes.insert(key.clone(), value.clone());
        return (operation, key, Some(value));
    } else {
        panic!("Recieved invalid operation that was not one of r or w");
    }
}

impl Reply<KVStoreNode> for KVStoreBody {
    fn into_reply(self, node_state: &mut KVStoreNode) -> Option<Self> {
        //!Consumes self, returns a Some(reply_body)
        //! if one exists.
        match self {
            KVStoreBody::Txn { msg_id, txn } => {
                let txn: Vec<_> = txn
                    .into_iter()
                    .map(|read_write_op| {
                        key_value_crud(
                            read_write_op,
                            &mut node_state.kv_store,
                            &mut node_state.unpropogated_writes,
                        )
                    })
                    .collect();
                Some(KVStoreBody::TxnOk {
                    msg_id: node_state.current_msg_id,
                    in_reply_to: msg_id,
                    txn,
                })
            }
            //Currently, if a WritePropogator is recieved, simply add those values
            //to the HashMap. No Ack at present.
            KVStoreBody::WritePropogater { write_ops, .. } => {
                for (key, value) in write_ops.into_iter() {
                    node_state.kv_store.insert(key, value);
                }
                None
            }

            _ => None,
        }
    }
}

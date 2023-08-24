use super::{MaelstromMessage, MaelstromReply, NodeMetadata};
use serde::{self, Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
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
        kv_store.insert(
            key.clone(),
            value.clone().expect("Recieved a Write Op without a value"),
        );
        return (operation, key, value);
    } else {
        panic!("Recieved invalid operation that was not one of r or w");
    }
}

impl MaelstromReply for KVStoreBody {
    fn into_reply(self, node_metadata: &mut NodeMetadata) -> Option<Self> {
        //!Consumes self, returns a Some(reply_body)
        //! if one exists.
        match self {
            KVStoreBody::Txn { msg_id, txn } => {
                let txn: Vec<_> = txn
                    .into_iter()
                    .map(|read_write_op| key_value_crud(read_write_op, &mut node_metadata.kv_store))
                    .collect();
                Some(KVStoreBody::TxnOk {
                    msg_id: node_metadata.current_msg_id,
                    in_reply_to: msg_id,
                    txn,
                })
            }
            //Currently, if a WritePropogator is recieved, simply add those values
            //to the HashMap. No Ack at present.
            KVStoreBody::WritePropogater { write_ops, .. } => {
                for (key, value) in write_ops.into_iter() {
                    node_metadata.kv_store.insert(key, value);
                }
                None
            }

            _ => None,
        }
    }

    fn extra_messages(
        &self,
        node_metadata: &mut NodeMetadata,
    ) -> Option<Vec<MaelstromMessage<Self>>> {
        if let KVStoreBody::TxnOk { txn, .. } = self {
            let write_ops =
                txn.clone()
                    .into_iter()
                    .filter(|(op, _, _)| op == "w")
                    .map(|(_, key, value)| {
                        (key, value.expect("Cannot recieve a write without a value"))
                    });

            //Build the writes for a transaction into a HashMap.
            let write_ops: HashMap<usize, usize> = HashMap::from_iter(write_ops);
            if write_ops.len() > 0 {
                let write_propogator = KVStoreBody::WritePropogater {
                    transaction_guid: node_metadata.current_msg_id,
                    write_ops,
                };
                let additional_messages = node_metadata
                    .node_ids
                    .iter()
                    .map(|node_id| MaelstromMessage {
                        src: node_metadata.node_id.clone(),
                        dest: node_id.clone(),
                        body: write_propogator.clone(),
                    })
                    .collect();
                return Some(additional_messages);
            }
        }
        None
    }
}

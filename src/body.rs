use super::NodeMetadata;
use serde::{self, Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Body {
    Init {
        msg_id: usize,
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk {
        in_reply_to: usize,
    },
    Echo {
        msg_id: usize,
        echo: String,
    },
    EchoOk {
        msg_id: usize,
        in_reply_to: usize,
        echo: String,
    },
    Generate {
        msg_id: usize,
    },
    GenerateOk {
        id: String,
        msg_id: usize,
        in_reply_to: usize,
    },
    Txn {
        msg_id: usize,
        txn: Vec<(String, usize, Option<usize>)>,
    },
    TxnOk {
        msg_id: usize,
        in_reply_to: usize,
        txn: Vec<(String, usize, Option<usize>)>,
    },
    WritePropogate {
        transaction_guid: usize,
        write_ops: HashMap<usize, usize>,
    },
    AckWritePropogate {},
}

impl Body {
    pub fn propogate_kv_writes(&self, node_metadata: &mut NodeMetadata) -> Option<Body> {
        if let Body::TxnOk { txn, .. } = self {
            let write_ops =
                txn.clone()
                    .into_iter()
                    .filter(|(op, _, _)| op == "w")
                    .map(|(_, key, value)| {
                        (key, value.expect("Cannot recieve a write without a value"))
                    });

            //Build the writes for a transaction into a HashMap.
            let write_ops: HashMap<usize, usize> = HashMap::from_iter(write_ops);
            eprintln!("Write Ops {:?} has len {:?}", write_ops, write_ops.len());
            if write_ops.len() > 0 {
                let write_propogator = Body::WritePropogate {
                    transaction_guid: node_metadata.current_msg_id,
                    write_ops,
                };
                return Some(write_propogator);
            }
            // let write_propogator = Body::WritePropogate {
            //     transaction_guid: node_metadata.current_msg_id,
            //     write_ops,
            // };
            // return Some(write_propogator);
        }
        None

        // //Insert into the HashMap of forwarded_writes the current_msg_id.
        // // When acks are returned, those items are removed from the HashMap.
        // let mut forwarded_writes = node_metadata
        //     .forwarded_writes
        //     .lock()
        //     .expect("Could not access forwarded writes ");
        // forwarded_writes.insert(
        //     node_metadata.current_msg_id,
        //     (write_propogator, node_metadata.node_ids.clone()),
        // );
        // };
    }
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

impl Body {
    pub fn into_reply(self, node_metadata: &mut NodeMetadata) -> Option<Body> {
        //!Consumes self, returns a Some(reply_body)
        //! if one exists.
        match self {
            Body::Init { msg_id, .. } => Some(Body::InitOk {
                in_reply_to: msg_id,
            }),
            Body::Echo { msg_id, echo, .. } => Some(Body::EchoOk {
                msg_id: node_metadata.current_msg_id,
                in_reply_to: msg_id,
                echo,
            }),
            Body::Generate { msg_id } => {
                //Because node_id is unique for a given node, and
                //Message IDs are unique per source node
                let unique_id = format!("{}|{}", &node_metadata.node_id, msg_id);
                Some(Body::GenerateOk {
                    id: unique_id,
                    msg_id: node_metadata.current_msg_id,
                    in_reply_to: msg_id,
                })
            }
            Body::Txn { msg_id, txn } => {
                let txn: Vec<_> = txn
                    .into_iter()
                    .map(|read_write_op| key_value_crud(read_write_op, &mut node_metadata.kv_store))
                    .collect();
                Some(Body::TxnOk {
                    msg_id: node_metadata.current_msg_id,
                    in_reply_to: msg_id,
                    txn,
                })
            }
            Body::WritePropogate { write_ops, .. } => {
                for (key, value) in write_ops.into_iter() {
                    node_metadata.kv_store.insert(key, value);
                }

                None
            }

            _ => None,
        }
    }
}

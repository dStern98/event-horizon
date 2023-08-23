use super::body::Body;
use super::NodeMetadata;
use serde::{self, Deserialize, Serialize};
use std::sync::mpsc;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MaelstromMessage {
    pub src: String,
    pub dest: String,
    pub body: Body,
}

impl MaelstromMessage {
    pub fn reply(
        self,
        std_out_transmitter: &mut mpsc::Sender<MaelstromMessage>,
        mut node_metadata: &mut NodeMetadata,
    ) {
        //! For a given MaelStromMessage, Build and send a reply
        //! if one exists.
        let reply_body = self.body.into_reply(&mut node_metadata);
        if let Some(reply_body) = reply_body {
            let write_propogators = reply_body.propogate_kv_writes(&mut node_metadata);
            let maelstrom_reply = MaelstromMessage {
                src: self.dest,
                dest: self.src,
                body: reply_body,
            };
            std_out_transmitter
                .send(maelstrom_reply)
                .expect("Failed to send Maelstrom Message");
            node_metadata.current_msg_id += 1;

            if let Some(write_propogator) = write_propogators {
                for other_node_id in node_metadata.node_ids.iter() {
                    let kv_write_propogator = MaelstromMessage {
                        src: node_metadata.node_id.clone(),
                        dest: other_node_id.clone(),
                        body: write_propogator.clone(),
                    };
                    std_out_transmitter
                        .send(kv_write_propogator)
                        .expect("Failed to send write propogator")
                }
            }
        }
    }
}

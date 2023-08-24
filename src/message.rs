use super::MaelstromReply;
use super::NodeMetadata;
use serde::{self, Deserialize, Serialize};
use std::sync::mpsc;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MaelstromMessage<Body> {
    pub src: String,
    pub dest: String,
    pub body: Body,
}

impl<Body> MaelstromMessage<Body>
where
    Body: MaelstromReply,
{
    pub fn reply(
        self,
        std_out_transmitter: &mut mpsc::Sender<MaelstromMessage<Body>>,
        mut node_metadata: &mut NodeMetadata,
    ) {
        //! For a given MaelStromMessage, Build and send a reply
        //! if one exists.
        let reply_body = self.body.into_reply(&mut node_metadata);
        if let Some(reply_body) = reply_body {
            //Get any Extra Messages that must be sent. In the future, it might
            //be better to make the extra_messages method operate on the inbound body.
            let extra_messages = reply_body.extra_messages(&mut node_metadata);
            let maelstrom_reply = MaelstromMessage {
                src: self.dest,
                dest: self.src,
                body: reply_body,
            };

            //Drop down the channel the direct reply.
            std_out_transmitter
                .send(maelstrom_reply)
                .expect("Failed to send Maelstrom Message");

            //Increment the current_msg_id to keep it unique per message
            //from this node.
            node_metadata.current_msg_id += 1;

            //If there are extra messages to send out, send them.
            //This could be for example in the Key-Value Store Challenge, notifying other nodes
            //of a write operation.
            if let Some(extra_messages) = extra_messages {
                for extra_message in extra_messages {
                    std_out_transmitter
                        .send(extra_message)
                        .expect("Failed to send write propogator");
                }
            }
        }
    }
}

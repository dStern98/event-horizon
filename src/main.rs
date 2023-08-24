use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json;
use std::io::Write;
use std::io::{self, BufRead};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

mod echo;
mod generate_id;
mod init;
use init::NodeMetadata;
mod kv_store;
mod message;
use message::MaelstromMessage;

pub trait MaelstromReply
where
    Self: Sized,
{
    ///Trait that represents a type that can be used for Node Message IO.
    ///into_reply, the core trait method which can return a reply to be sent
    /// back to the client.
    fn into_reply(self, node_metadata: &mut NodeMetadata) -> Option<Self>;

    ///extra_messages are any additional messages that should be sent to other Nodes/Clients
    /// in addition to the direct reply to the inbound message.
    fn extra_messages(
        &self,
        node_metadata: &mut NodeMetadata,
    ) -> Option<Vec<MaelstromMessage<Self>>> {
        let _ = node_metadata;
        None
    }
}

fn node_runtime<Body>(
    mut node_metadata: NodeMetadata,
    mut tx: Sender<MaelstromMessage<Body>>,
    rx: Receiver<MaelstromMessage<Body>>,
) where
    Body: Serialize + DeserializeOwned + MaelstromReply + Send + 'static,
{
    //Spawns a thread that reads stdin. Any messages to be sent out are dropped
    // down the channel to the main thread, which has an stdout handle.
    let mut stdout_handle = io::stdout().lock();
    thread::spawn(move || {
        let stdin_handle = io::stdin().lock();
        for line in stdin_handle.lines() {
            let line = line.expect("Nothing recieved via stdin...");
            let message: Result<MaelstromMessage<Body>, _> = serde_json::from_str(&line);
            match message {
                Ok(maelstrom_inbound) => {
                    maelstrom_inbound.reply(&mut tx, &mut node_metadata);
                }
                Err(_) => eprintln!("Received the following invalid maelstrom message: {}", line),
            }
        }
    });

    while let Ok(maelstrom_message) = rx.recv() {
        serde_json::to_writer(&mut stdout_handle, &maelstrom_message)
            .expect("Unable to serialize to writer.");
        //Maelstrom requires a new line character
        stdout_handle
            .write_all(b"\n")
            .expect("Unable to write newline character");
    }
}

fn main() {
    let node_metadata = init::MaelstromInit::init_node();
    //Turbofish on the channel specifies the Generic Body to be used in the runtime.
    // To configure for a different Maelstrom challenge, simply change the innermost body type.
    let (tx, rx) = channel::<MaelstromMessage<kv_store::KVStoreBody>>();
    node_runtime(node_metadata, tx, rx);
}

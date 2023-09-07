use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json;
use std::io::Write;
use std::io::{self, BufRead};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

mod echo;
mod generate_id;
mod grow_counter;
mod init;
use init::NodeMetadata;
mod kv_store;

pub enum Event<Body> {
    Message(MaelstromMessage<Body>),
    PropogateWrites,
}

pub trait Node<Body> {
    fn node_init(node_metadata: init::NodeMetadata, event_tx: Sender<Event<Body>>) -> Self;
    fn handle_event(&mut self, event: Event<Body>, stdout_handle: &mut io::StdoutLock)
    where
        Body: Reply<Self>,
        Self: Sized;
}

pub trait Reply<NodeState>
where
    Self: Sized,
{
    ///Trait that represents a type that can be used for Node Message IO.
    ///into_reply, the core trait method which can return a reply to be sent
    /// back to the client.
    fn into_reply(self, node_state: &mut NodeState) -> Option<Self>;
}

fn node_runtime<Body, NodeState>(
    mut node: NodeState,
    tx: Sender<Event<Body>>,
    rx: Receiver<Event<Body>>,
) where
    NodeState: Node<Body>,
    Body: Serialize + DeserializeOwned + Reply<NodeState> + Send + 'static,
{
    //Spawns a thread that reads stdin. Any messages to be sent out are dropped
    // down the channel to the main thread, which has an stdout handle.
    thread::spawn(move || {
        let stdin_handle = io::stdin().lock();
        for line in stdin_handle.lines() {
            let line = line.expect("Nothing recieved via stdin...");
            let message: Result<MaelstromMessage<Body>, _> = serde_json::from_str(&line);
            match message {
                Ok(inbound_message) => {
                    tx.send(Event::Message(inbound_message))
                        .expect("Failed to transmit to stdout thread");
                }
                Err(_) => eprintln!("Received the following invalid maelstrom message: {}", line),
            }
        }
    });
    let mut stdout_handle = io::stdout().lock();
    while let Ok(event) = rx.recv() {
        node.handle_event(event, &mut stdout_handle);
    }
}

fn main() {
    let node_metadata = init::MaelstromInit::init_node();
    let (tx, rx) = channel();
    let init_event_tx = tx.clone();
    let node = Node::node_init(node_metadata, init_event_tx);
    node_runtime::<kv_store::KVStoreBody, kv_store::KVStoreNode>(node, tx, rx);
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MaelstromMessage<Body> {
    pub src: String,
    pub dest: String,
    pub body: Body,
}

impl<Body> MaelstromMessage<Body>
where
    Body: Serialize,
{
    pub fn send(&mut self, mut stdout_handle: &mut io::StdoutLock) {
        //Given a stdout handle, write the MaelStrom Message via serde_json
        serde_json::to_writer(&mut stdout_handle, self).expect("Unable to serialize to writer.");
        //Maelstrom requires a new line character
        stdout_handle
            .write_all(b"\n")
            .expect("Unable to write newline character");
    }
    pub fn message_reply<NodeState>(
        self,
        mut stdout_handle: &mut io::StdoutLock,
        mut node_state: &mut NodeState,
    ) where
        Body: Reply<NodeState>,
    {
        //! For a given MaelStromMessage, Build and send a reply
        //! if one exists.
        let reply_body = self.body.into_reply(&mut node_state);
        if let Some(reply_body) = reply_body {
            let mut message = MaelstromMessage {
                src: self.dest,
                dest: self.src,
                body: reply_body,
            };
            message.send(&mut stdout_handle);
        }
    }
}

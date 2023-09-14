use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json;
use std::io::{self, BufRead};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

mod echo;
mod generate_id;
mod grow_counter;
mod init;
use init::NodeMetadata;
mod broadcast;
mod kv_store;
mod node;
use node::{Event, MaelstromMessage, Node, Reply};

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
    node_runtime::<broadcast::BroadcastBody, broadcast::BroadcastNode>(node, tx, rx);
}

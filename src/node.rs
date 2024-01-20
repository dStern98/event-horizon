use crate::init;
use serde::{Deserialize, Serialize};
use std::io;
use std::io::Write;
use std::sync::mpsc::Sender;

pub mod broadcast;
pub mod echo;
pub mod generate_id;
pub mod grow_counter;
pub mod kafka;
pub mod kv_store;

pub enum Event<Body> {
    //An Event can be anything that
    //the Node should react to in some way.
    Message(MaelstromMessage<Body>),
    PropogateWrites,
}

pub trait Node<Body> {
    //Core trait representing a type that can be used as a Node in a Maelstrom Challenge.
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
    //Types that impl this trait are Reply Bodies for Maelstrom Messages
    fn into_reply(self, node_state: &mut NodeState, src: &str) -> Option<Self>;
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
        stdout_handle: &mut io::StdoutLock,
        node_state: &mut NodeState,
    ) where
        Body: Reply<NodeState>,
    {
        //! For a given MaelStromMessage, Build and send a reply
        //! if one exists.
        let reply_body = self.body.into_reply(node_state, &self.src);
        if let Some(reply_body) = reply_body {
            let mut message = MaelstromMessage {
                src: self.dest,
                dest: self.src,
                body: reply_body,
            };
            message.send(stdout_handle);
        }
    }
}

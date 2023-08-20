use serde::{self, Deserialize, Serialize};

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
}

impl Body {
    pub fn into_reply(self, node_id: &String) -> Option<Body> {
        //!Consumes self, returns a Some(reply_body)
        //! if one exists.
        match self {
            Body::Init { msg_id, .. } => Some(Body::InitOk {
                in_reply_to: msg_id,
            }),
            Body::Echo { msg_id, echo } => Some(Body::EchoOk {
                msg_id,
                in_reply_to: msg_id,
                echo,
            }),
            Body::Generate { msg_id } => {
                //Because node_id is unique for a given node, and
                //Message IDs are unique per source node
                let unique_id = format!("{}|{}", node_id, msg_id);
                Some(Body::GenerateOk {
                    id: unique_id,
                    msg_id,
                    in_reply_to: msg_id,
                })
            }
            _ => None,
        }
    }
}

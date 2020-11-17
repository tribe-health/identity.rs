use core::fmt::Debug;

use crate::{client::Client, error::Result};
use iota::{Indexation, Message, Payload};

#[derive(Clone, PartialEq, Debug)]
pub struct SendMessageResponse {
    pub hash: String,
}

#[derive(Debug)]
pub struct SendMessageRequest<'a> {
    pub(crate) client: &'a Client,
    pub(crate) trace: bool,
}

impl<'a> SendMessageRequest<'a> {
    pub const fn new(client: &'a Client) -> Self {
        Self { client, trace: false }
    }

    pub fn trace(mut self, value: bool) -> Self {
        self.trace = value;
        self
    }

    pub async fn send(self, index: Indexation) -> Result<SendMessageResponse> {
        if self.trace {
            println!("[+] trace: Sending Message >");
        }

        let tips = self.client.client.get_tips().await.unwrap();

        let message = Message::builder()
            .with_parent1(tips.0)
            .with_parent2(tips.1)
            .with_payload(Payload::Indexation(Box::new(index)))
            .finish()
            .unwrap();

        let hash = self.client.client.post_message(&message).await?;

        if self.trace {
            println!("[+] trace: Message sent > {}", hash);
        }

        Ok(SendMessageResponse { hash: hash.to_string() })
    }
}

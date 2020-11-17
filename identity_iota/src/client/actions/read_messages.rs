use crate::{
    client::Client,
    did::IotaDID,
    error::{Error, Result},
};
use iota::{hex_to_message_id, Message};

#[derive(Clone, Debug)]
pub struct ReadMessagesResponse {
    pub did: IotaDID,
    pub messages: Vec<Message>,
}

#[derive(Debug)]
pub struct ReadMessagesRequest<'a> {
    pub(crate) client: &'a Client,
    pub(crate) did: IotaDID,
    pub(crate) allow_empty: bool,
    pub(crate) trace: bool,
}

impl<'a> ReadMessagesRequest<'a> {
    pub const fn new(client: &'a Client, did: IotaDID) -> Self {
        Self {
            client,
            did,
            allow_empty: true,
            trace: false,
        }
    }

    pub fn allow_empty(mut self, value: bool) -> Self {
        self.allow_empty = value;
        self
    }

    pub fn trace(mut self, value: bool) -> Self {
        self.trace = value;
        self
    }

    pub async fn send(self) -> Result<ReadMessagesResponse> {
        if self.trace {
            println!("[+] trace(1): Find messages with indexation tag: {:?}", self.did);
        }

        // Fetch all message hashes with the indexation tag.
        let response = self
            .client
            .client
            .get_message()
            .index(&self.did.method_id())
            .await
            .unwrap();

        if self.trace {
            println!("[+] trace(2): FindMessages Response: {:?}", response);
        }

        if response.is_empty() {
            if self.allow_empty {
                return Ok(ReadMessagesResponse {
                    did: self.did,
                    messages: Vec::new(),
                });
            } else {
                return Err(Error::NoMessages);
            }
        }

        if self.trace {
            println!("[+] trace(3): Tangle message hashes: {:?}", response);
        }

        let mut messages: Vec<Message> = Vec::new();
        for hash in response.iter() {
            let message = self
                .client
                .client
                .get_message()
                // Only single message
                .data(&hex_to_message_id(hash).unwrap())
                .await?;
            messages.push(message);
        }

        if self.trace {
            println!("[+] trace(4): Tangle Messages: {:?}", messages);
        }

        Ok(ReadMessagesResponse {
            did: self.did,
            messages,
        })
    }
}

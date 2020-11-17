use core::fmt::Debug;
use identity_core::common::ToJson as _;
use iota::Indexation;

use crate::{
    client::{SendMessageRequest, SendMessageResponse},
    did::{IotaDID, IotaDocument},
    error::{Error, Result},
};

#[derive(Clone, PartialEq, Debug)]
#[repr(transparent)]
pub struct PublishDocumentResponse {
    pub hash: String,
}

#[derive(Debug)]
pub struct PublishDocumentRequest<'a, 'b> {
    pub(crate) transfer: SendMessageRequest<'a>,
    pub(crate) document: &'b IotaDocument,
}

impl<'a, 'b> PublishDocumentRequest<'a, 'b> {
    pub const fn new(transfer: SendMessageRequest<'a>, document: &'b IotaDocument) -> Self {
        Self { transfer, document }
    }

    pub fn trace(mut self, value: bool) -> Self {
        self.transfer = self.transfer.trace(value);
        self
    }

    pub async fn send(self) -> Result<PublishDocumentResponse> {
        let did: &IotaDID = self.document.did();

        if self.transfer.trace {
            println!("[+] trace(1): Create Document with DID: {:?}", did);
        }

        // Ensure the correct network is selected.
        if !self.transfer.client.network.matches_did(&did) {
            return Err(Error::InvalidDIDNetwork);
        }

        if self.transfer.trace {
            println!(
                "[+] trace(2): Authentication Method: {:?}",
                self.document.authentication_key()
            );
        }

        // Verify the document signature with the authentication key.
        self.document.verify()?;

        // Create a transfer to publish the DID document with the specified indexation tag.

        let index = Indexation::new(did.method_id().into(), self.document.to_json()?.as_bytes()).unwrap();

        // Submit the transfer to the tangle.
        let response: SendMessageResponse = self.transfer.send(index).await?;

        Ok(PublishDocumentResponse { hash: response.hash })
    }
}

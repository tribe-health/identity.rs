use crate::{
    client::{ClientBuilder, PublishDocumentRequest, ReadDocumentRequest, ReadMessagesRequest, SendMessageRequest},
    did::{IotaDID, IotaDocument},
    error::Result,
    network::Network,
};
use async_trait::async_trait;
use identity_core::{
    did::DID,
    error::{Error, Result as CoreResult},
    resolver::{DocumentMetadata, InputMetadata, MetaDocument, ResolverMethod},
};

#[derive(Clone, Debug)]
pub struct Client {
    pub(crate) client: iota::Client,
    pub(crate) network: Network,
}

impl Client {
    pub fn message_url(&self, message_id: String) -> String {
        format!("{}message/{}", self.network.explorer_url(), message_id)
    }

    pub fn new() -> Result<Self> {
        Self::from_builder(ClientBuilder::new())
    }

    pub fn from_builder(builder: ClientBuilder) -> Result<Self> {
        let mut client: iota::ClientBuilder = iota::ClientBuilder::new();

        for node in builder.nodes {
            client = client.node(&node)?;
        }

        client = client.network(builder.network.into());

        Ok(Self {
            client: client.build()?,
            network: builder.network,
        })
    }

    pub fn read_messages<'a>(&'a self, did: &IotaDID) -> ReadMessagesRequest<'a> {
        ReadMessagesRequest::new(self, did.clone())
    }

    pub fn send_indexation(&self) -> SendMessageRequest {
        SendMessageRequest::new(self)
    }

    pub fn create_document<'a, 'b>(&'a self, document: &'b IotaDocument) -> PublishDocumentRequest<'a, 'b> {
        PublishDocumentRequest::new(self.send_indexation(), document)
    }

    pub fn read_document<'a, 'b>(&'a self, did: &'b IotaDID) -> ReadDocumentRequest<'a, 'b> {
        ReadDocumentRequest::new(self, did)
    }

    // Doesn't work at the moment
    // pub async fn is_message_confirmed(&self, message_id: &str) -> Result<bool> {
    //     let response = self
    //         .client
    //         .get_message()
    //         .metadata(&hex_to_message_id(message_id)?)
    //         .await?;

    //     if &response
    //         .ledger_inclusion_state
    //         .expect("Missing ledger_inclusion_state field in metadata")
    //         == "included"
    //     {
    //         Ok(true)
    //     } else {
    //         Ok(false)
    //     }
    // }
}

#[async_trait(?Send)]
impl ResolverMethod for Client {
    fn is_supported(&self, did: &DID) -> bool {
        IotaDID::is_valid(did) && self.network.matches_did(did)
    }

    async fn read(&self, did: &DID, _input: InputMetadata) -> CoreResult<Option<MetaDocument>> {
        let did: IotaDID = IotaDID::try_from_did(did.clone()).map_err(|error| Error::ResolutionError(error.into()))?;

        self.read_document(&did)
            .send()
            .await
            .map_err(|error| Error::ResolutionError(error.into()))
            .map(|response| {
                let mut metadata: DocumentMetadata = DocumentMetadata::new();

                metadata.created = response.document.created;
                metadata.updated = response.document.updated;
                metadata.properties = response.metadata;

                Some(MetaDocument {
                    data: response.document.into(),
                    meta: metadata,
                })
            })
    }
}

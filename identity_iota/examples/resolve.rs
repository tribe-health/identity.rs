//! Resolve a DID
//! cargo run --example resolve

use anyhow::Result;
use identity_core::resolver::resolve;
use identity_iota::{
    client::{Client, ClientBuilder},
    network::Network,
};

#[smol_potat::main]
async fn main() -> Result<()> {
    let client: Client = ClientBuilder::new()
        .node("http://localhost:14265")
        // .node("https://nodes.iota.org:443")
        // .node("https://nodes.thetangle.org:443")
        // .node("https://iotanode.us:14267")
        // .node("https://pow.iota.community:443")
        .network(Network::Mainnet)
        .build()?;

    let did = "did:iota:686NgwsEToCRfujQfBz5wtqE5yW5S3UPjWDLjrMhALWt";
    let res = resolve(did, Default::default(), &client).await?;

    println!("{:#?}", res);

    Ok(())
}

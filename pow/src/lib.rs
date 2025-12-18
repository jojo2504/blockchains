mod blockchain;
mod wallet;
mod mempool;
mod types;
mod consensus;
mod node;

use std::{error::Error, sync::Arc};

use crate::node::node::Node;

pub async fn start_node() -> Result<(), Box<dyn Error + Send + Sync>> {
    let node = Arc::new(Node::new());

    // Spawn P2P
    let node_clone = node.clone();
    node_clone.startp2p().await?;

    // Spawn RPC server
    // let node_clone = node.clone();
    // tokio::spawn(async move {
    //     if let Err(e) = node_clone.start_rpc().await {
    //         eprintln!("RPC server failed: {:?}", e);
    //     }
    // });

    // // Spawn mining loop
    // let node_clone = node.clone();
    // tokio::spawn(async move {
    //     node_clone.mining_loop().await;
    // });

    // Node main can do other work or just await forever
    futures::future::pending::<()>().await;

    Ok(())
}
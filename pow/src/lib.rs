mod blockchain;
mod wallet;
mod mempool;
mod types;
mod consensus;
mod node;

use std::{error::Error, sync::Arc};

use chrono::Utc;
use futures::StreamExt;
use tarpc::{client, context, serde_transport, server::{self, Channel}, tokio_serde::formats::Bincode};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::{node::node::{Node, NodeRpc, NodeRpcClient, NodeRpcServer}, types::block::Block};

pub async fn start_rpc(node: Arc<Node>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let listener = TcpListener::bind("127.0.0.1:7000").await?;
    println!("rpc listening on 127.0.0.1:7000");

    loop {
        let (stream, _) = listener.accept().await?;

        // Wrap the TCP stream with length-delimited framing
        let framed = Framed::new(stream, LengthDelimitedCodec::new());
        
        // Create the transport with bincode serialization
        let transport = serde_transport::new(framed, Bincode::default());

        let server = NodeRpcServer { node: node.clone() };

        tokio::spawn(
            server::BaseChannel::with_defaults(transport)
                .execute(server.serve())
                .for_each(|response| async move {
                    tokio::spawn(response);
                }),
        );
    }
}

// this should be in another dedicated client, but for testing purposes:
pub async fn mining(node: Arc<Node>) -> Result<(), Box<dyn Error + Send + Sync>> {
    // connect to node
    let stream = TcpStream::connect("127.0.0.1:7000").await?;
    let framed = Framed::new(stream, LengthDelimitedCodec::new());
    let transport = serde_transport::new(framed, Bincode::default());

    let client = NodeRpcClient::new(client::Config::default(), transport).spawn();
    
    loop {
        let job = client.get_mining_job(context::current()).await?;
        println!("Got mining new job: {:?}", job);
        let mut block = Block::new(job.transactions.clone(), Some(job.parent_block_hash));
        let start = Utc::now();
        loop {
            block.hash = block.calculate_hash();
            // println!("current hash: {:?} vs target: {:?}", block.hash, node.consensus.read().await.state.difficulty.target);
            if node.consensus.read().await.verify_valid_block(node.blockchain.read().await.last_block(), &block) {
                println!("mined new block: {:?}", block);
                (*node).clone().push_block(block).await;
                break;
            }
            block.nonce += 1;   
        }
        println!("time to find new block: {}", (Utc::now() - start).as_seconds_f32());
        println!("total time windows: {}", (Utc::now() - node.consensus.read().await.state.start_window_time).as_seconds_f32());
    }
}

pub async fn start_node() -> Result<(), Box<dyn Error + Send + Sync>> {
    let node = Arc::new(Node::new());

    // Spawn P2P
    let node_clone = node.clone();
    node_clone.startp2p().await?;

    // Spawn RPC server
    let node_clone = node.clone();
    tokio::spawn(async move {
        if let Err(why) = start_rpc(node_clone).await {
            eprintln!("RPC server failed: {:?}", why);
        }
    });

    // Spawn mining loop
    let node_clone = node.clone();
    tokio::spawn(async move {
        if let Err(why) = mining(node_clone).await {
            eprintln!("mining failed: {:?}", why);
        }
    });

    // Node main can do other work or just await forever
    futures::future::pending::<()>().await;

    Ok(())
}
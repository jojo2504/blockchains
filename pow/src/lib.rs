mod blockchain;
mod wallet;
mod mempool;
mod types;
mod consensus;
mod node;
mod blockchain_viewer;

use std::{error::Error, sync::Arc};

use chrono::Utc;
use futures::{StreamExt, future::ok};
use tarpc::{client, context, serde_transport, server::{self, Channel}, tokio_serde::formats::Bincode};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::{blockchain_viewer::BlockchainViewer, node::node::{Node, NodeRpc, NodeRpcClient, NodeRpcServer}, types::block::Block};

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

pub fn start_node() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Create node
    let node = Arc::new(Node::new());
    
    // Subscribe to events BEFORE spawning anything
    let rx = node.events.subscribe();

    // Spawn tokio runtime in background thread for async work
    let node_clone = node.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            // Spawn P2P
            let node_p2p = node_clone.clone();
            tokio::spawn(async move {
                if let Err(why) = node_p2p.startp2p().await {
                    eprintln!("P2P failed: {:?}", why);
                }
            });

            // Spawn RPC server
            let node_rpc = node_clone.clone();
            tokio::spawn(async move {
                if let Err(why) = start_rpc(node_rpc).await {
                    eprintln!("RPC server failed: {:?}", why);
                }
            });

            // Spawn mining loop
            let node_mining = node_clone.clone();
            tokio::spawn(async move {
                if let Err(why) = mining(node_mining).await {
                    eprintln!("Mining failed: {:?}", why);
                }
            });

            // Keep runtime alive
            futures::future::pending::<()>().await;
        });
    });

    // Run GUI on main thread
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Blockchain Viewer",
        options,
        Box::new(move |_| Ok(Box::new(BlockchainViewer::new(rx)))),
    ).unwrap();

    Ok(())
    // .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
}
mod blockchain;
mod wallet;
mod mempool;
mod types;
mod consensus;
mod node;
mod blockchain_viewer;

use std::{error::Error, sync::Arc, time::Duration};

use chrono::Utc;
use futures::{StreamExt, future::ok};
use num_bigint::BigUint;
use tarpc::{client, context, serde_transport, server::{self, Channel}, tokio_serde::formats::Bincode};
use tokio::{net::{TcpListener, TcpStream}, time::sleep};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::{blockchain_viewer::BlockchainViewer, node::node::{Node, NodeEvent, NodeRpc, NodeRpcClient, NodeRpcServer}, types::{block::Block, subtypes::{Address, Signature}, transaction::Transaction}};

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
pub async fn mining() -> Result<(), Box<dyn Error + Send + Sync>> {
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
            if BigUint::from_bytes_be(&block.hash.0) <= job.difficulty {
                println!("Found valid hash: {:?}", block.hash);
                // Submit to node - node will re-verify!
                let accepted = client.submit_block(context::current(), block.clone()).await?;
                if accepted {
                    println!("✓ Block accepted!");
                } else {
                    println!("✗ Block rejected by node");
                }
                break;
            }
            block.nonce += 1;   
        }
        println!("time to find new block: {}", (Utc::now() - start).as_seconds_f32());
    }
}

/// for testing purposes, will loop forever creating tx every x seconds
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

pub async fn creating_tx(node: Arc<Node>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut rng = StdRng::seed_from_u64(rand::random());
    let mut id = 0;
    
    loop {
        // Random from address
        let mut from_addr = [0u8; 32];
        rng.fill(&mut from_addr);
        let from = Address(from_addr);
        
        // Random to address (80% chance of having recipient, 20% message only)
        let to = if rng.gen_bool(0.8) {
            let mut to_addr = [0u8; 32];
            rng.fill(&mut to_addr);
            Some(Address(to_addr))
        } else {
            None
        };
        
        // Random nonce
        let nonce = rng.gen_range(0..1000000);
        
        // Random fee (between 100 and 10000)
        let fee = rng.gen_range(100..10000);
        
        // Random amount (if has recipient)
        let amount = if to.is_some() {
            Some(rng.gen_range(1000..1000000))
        } else {
            None
        };
        
        // Random message (50% chance)
        let message = if rng.gen_bool(0.5) {
            let messages = vec![
                "hello world",
                "test transaction",
                "blockchain rocks",
                "random message",
                "decentralized network",
                "consensus achieved",
                "mining block",
                "validating tx",
            ];
            let msg = messages[rng.gen_range(0..messages.len())];
            Some(msg.as_bytes().to_vec())
        } else {
            None
        };
        
        // Random signature
        let mut sig = [0u8; 32];
        rng.fill(&mut sig);
        let signature = Signature(sig);
        
        let new_tx = Transaction::new(
            id.to_string(),
            from,
            to,
            nonce,
            fee,
            amount,
            message.clone(),
            signature,
        );
        
        node.create_tx(new_tx).await;
        println!("Created tx #{}: fee={}, amount={:?}, has_message={}", 
                 id, fee, amount, message.is_some());
        
        id += 1;
        
        // Random delay between 100ms and 2 seconds
        let delay_ms = rng.gen_range(100..2000);
        sleep(Duration::from_millis(delay_ms)).await;
    }
}

// Bonus: Generate multiple random transactions at once
pub async fn creating_batch_tx(
    node: Arc<Node>,
    batch_size: usize,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut rng = StdRng::seed_from_u64(rand::random());
    let mut id = 0;
    
    loop {
        let mut transactions = Vec::new();
        
        // Random batch size between batch_size and batch_size * 3
        let actual_batch = rng.random_range(batch_size..(batch_size * 3));
        
        for _ in 0..actual_batch {
            let mut from_addr = [0u8; 32];
            rng.fill(&mut from_addr);
            
            // Always have a recipient
            
            // 50% money transfer, 50% message
            let is_money_transfer = rng.random_bool(0.5);
            
            let mut to = None;
            let (amount, message) = if is_money_transfer {
                // Money transfer - no message
                let mut to_addr = [0u8; 32];
                rng.fill(&mut to_addr);
                to = Some(Address(to_addr));
                (Some(rng.random_range(1000..1000000)), None)
            } else {
                // Message only - no amount
                let messages = vec![
                    "hello world",
                    "test transaction",
                    "blockchain rocks",
                    "random message",
                    "decentralized network",
                    "consensus achieved",
                    "mining block",
                    "validating tx",
                    "gm ser",
                    "wagmi",
                ];
                let msg = messages[rng.random_range(0..messages.len())];
                (None, Some(msg.as_bytes().to_vec()))
            };
            
            let tx = Transaction::new(
                id.to_string(),
                Address(from_addr),
                to,
                rng.random_range(0..1000000),
                rng.random_range(100..10000),
                amount,
                message,
                {
                    let mut sig = [0u8; 32];
                    rng.fill(&mut sig);
                    Signature(sig)
                },
            );
            
            transactions.push(tx);
            id += 1;
        }
        
        for tx in transactions {
            node.create_tx(tx).await;
        }
        
        println!("Created batch of {} transactions", actual_batch);
        sleep(Duration::from_secs(5)).await;
    }
}

pub fn start_node() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Create node
    let node = Arc::new(Node::new());
    
    // Subscribe to events BEFORE spawning anything
    let rx = node.events.subscribe();

    let genesis = node.blockchain.blocking_read().blocks[0].clone();
    let _ = node.events.send(NodeEvent::NewBlock(genesis));

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
            tokio::spawn(async move {
                if let Err(why) = mining().await {
                    eprintln!("Mining failed: {:?}", why);
                }
            });

            // Create random tx - use same runtime as other tasks
            let node_tx = node_clone.clone();
            tokio::spawn(async move {
                if let Err(why) = creating_batch_tx(node_tx, 5).await {
                    eprintln!("creating tx failed: {:?}", why);
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
use std::{collections::HashSet, error::Error, os::unix::net::SocketAddr, sync::Arc};
use chrono::{Date, Utc};
use futures::StreamExt;
use libp2p::{Multiaddr, PeerId, noise, ping, swarm::SwarmEvent, tcp, yamux};
use num_bigint::BigUint;
use tarpc::context;
use tokio::sync::{Mutex, RwLock, broadcast};
use tracing_subscriber::EnvFilter;

use crate::{blockchain::blockchain::Blockchain, consensus::{self, consensus::{ADJUSTMENT_INTERVAL, Consensus, State, TARGET_BLOCK_TIME}, difficulty::Difficulty}, mempool::mempool::Mempool, types::{block::Block, mining_job::MiningJob}};

#[tarpc::service]
pub trait NodeRpc {
    async fn get_mining_job() -> MiningJob;
    async fn submit_block(block: Block) -> bool;
}

#[derive(Clone)]
pub struct NodeRpcServer {
    pub node: Arc<Node>,
}

impl NodeRpc for NodeRpcServer {
    async fn get_mining_job(self, _: context::Context) -> MiningJob {
        let node = (*self.node).clone();
        let blockchain = node.blockchain.read().await;
        let last_block = blockchain.last_block();
        MiningJob {
            parent_block_hash: last_block.hash,
            difficulty: node.consensus.read().await.state.difficulty.target.clone(),
            transactions: node.mempool.read().await.pool.to_vec(),
            target_block_time: TARGET_BLOCK_TIME,
            height: node.blockchain.read().await.blocks.len() as u64,
        }
    }

    async fn submit_block(self, _: context::Context, block: Block) -> bool {
        let node = (*self.node).clone();
        true
    }
}

#[derive(Clone, Debug)]
pub enum NodeEvent {
    NewBlock(Block),
}

#[derive(Clone)]
pub struct Node {
    /* =======================
       Blockchain state
    ======================== */
    pub blockchain: Arc<RwLock<Blockchain>>,
    pub mempool: Arc<RwLock<Mempool>>,

    /* =======================
       Consensus
    ======================== */
    pub consensus: Arc<RwLock<Consensus>>,

    /* =======================
       Networking
    ======================== */
    pub peers: Arc<Mutex<HashSet<PeerId>>>,
    // pub swarm: Swarm<NodeBehaviour>,

    /* =======================
       Mining coordination
    ======================== */
    // pub mining_enabled: bool,
    // pub current_job: Option<MiningJob>,

    /* =======================
       API / RPC
    ======================== */
    pub rpc_addr: SocketAddr,

    /* =======================
       Node identity
    ======================== */
    pub node_id: PeerId,

    pub events: broadcast::Sender<NodeEvent>,
}

impl Node {
    pub fn new() -> Self {
        let (events, _) = broadcast::channel(1024);
        let mut genesis = Block::new(vec![], None);
        genesis.hash = genesis.calculate_hash();

        Self {
            blockchain: Arc::new(RwLock::new(Blockchain::new(genesis))),
            mempool: Arc::new(RwLock::new(Mempool::new())),
            consensus: Arc::new(RwLock::new(Consensus::new(State {
                difficulty: Difficulty { 
                    target: BigUint::from_bytes_be(&[
                        0x00, 0x00, 0x0f, 0xff, 0xff, 0xff, 0xff, 0xff,
                        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                    ]) 
                },
                start_window_time: Utc::now()
            }))),
            peers: Arc::new(Mutex::new(HashSet::new())),
            rpc_addr: SocketAddr::from_pathname("/tmp").unwrap(),
            node_id: PeerId::random(),
            events: events
        }
    }

    pub async fn startp2p(self: Arc<Self>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

        let mut swarm = libp2p::SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|_| ping::Behaviour::default())?
            .build();
        
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        // Dial the peer identified by the multi-address given as the second
        // command-line argument, if any.
        if let Some(addr) = std::env::args().nth(1) {
            let remote: Multiaddr = addr.parse()?;
            swarm.dial(remote)?;
            println!("Dialed {addr}")
        }

        tokio::spawn(async move {
            loop {
                match swarm.select_next_some().await {
                    SwarmEvent::NewListenAddr { address, .. } => println!("Listening on {address:?}"),
                    SwarmEvent::Behaviour(event) => println!("P2P event: {event:?}"),
                    _ => {}
                }
            }
        });

        Ok(())
    }

    pub async fn push_block(&mut self, block: Block) {
        self.blockchain.write().await.blocks.push(block.clone());
        println!("current blockchain height: {}", self.blockchain.read().await.blocks.len());

        let _ = self.events.send(NodeEvent::NewBlock(block));

        if self.blockchain.read().await.blocks.len() % (ADJUSTMENT_INTERVAL as usize) == 0 {
            println!("adjusting...");
            let stop_window_time = Utc::now();
            {
                let mut consensus_write = self.consensus.write().await;
                consensus_write.state.adjust_difficulty(stop_window_time);
                consensus_write.state.start_window_time = stop_window_time;
            } // Write lock is dropped here
            println!("new difficulty: {:?}", self.consensus.read().await.state.difficulty.target);
        }
    }
}

use std::{collections::HashSet, error::Error, os::unix::net::SocketAddr, sync::Arc};
use futures::StreamExt;
use libp2p::{Multiaddr, PeerId, noise, ping, swarm::SwarmEvent, tcp, yamux};
use num_bigint::BigUint;
use tarpc::context;
use tokio::sync::{Mutex, RwLock};
use tracing_subscriber::EnvFilter;

use crate::{blockchain::blockchain::Blockchain, consensus::{consensus::{Consensus, State, TARGET_BLOCK_TIME}, difficulty::Difficulty}, mempool::mempool::Mempool, types::{block::Block, mining_job::MiningJob}};

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
}

impl Node {
    pub fn new() -> Self {
        Self {
            blockchain: Arc::new(RwLock::new(Blockchain::new())),
            mempool: Arc::new(RwLock::new(Mempool::new())),
            consensus: Arc::new(RwLock::new(Consensus::new(State {
                difficulty: Difficulty { target: BigUint::from(1u64) },
            }))),
            peers: Arc::new(Mutex::new(HashSet::new())),
            rpc_addr: SocketAddr::from_pathname("/tmp").unwrap(),
            node_id: PeerId::random(),
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
}

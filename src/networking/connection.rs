use anyhow::{Context, Result};
use futures::StreamExt;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use libp2p::gossipsub::{self, IdentTopic as Topic, MessageAuthenticity, ValidationMode};
use libp2p::identity::Keypair;
use libp2p::mdns::{Config as MdnsConfig, Event as MdnsEvent, tokio::Behaviour as Mdns};
use libp2p::swarm::{NetworkBehaviour, SwarmEvent, dial_opts::DialOpts};
use libp2p::{self as libp2p_core, Multiaddr, PeerId, SwarmBuilder};
use log::info;
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TransformMsg {
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
    pub ts_millis: u128,
    pub instance_id: String,
    pub seq: u64,
}

// Generates AppBehaviourEvent via macro
#[derive(NetworkBehaviour)]
pub struct AppBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: Mdns,
}

type P2PInfo = (
    JoinHandle<()>,
    UnboundedSender<Vec<u8>>,
    UnboundedReceiver<Vec<u8>>,
    PeerId,
    UnboundedReceiver<usize>,
);

pub fn start_p2p(topic_str: &str) -> Result<P2PInfo> {
    let keypair = Keypair::generate_ed25519();
    let peer_id = PeerId::from(keypair.public());
    info!("Local peer id: {peer_id}");

    let topic = Topic::new(topic_str);

    let mut swarm = SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            libp2p_core::tcp::Config::default(),
            libp2p_core::noise::Config::new,
            libp2p_core::yamux::Config::default,
        )?
        .with_behaviour(|key| {
            let gossipsub_config = gossipsub::ConfigBuilder::default()
                .validation_mode(ValidationMode::Permissive)
                .heartbeat_interval(std::time::Duration::from_secs(1))
                .build()
                .map_err(|e| anyhow::anyhow!(e))?;

            let gossipsub = gossipsub::Behaviour::new(
                MessageAuthenticity::Signed(key.clone()),
                gossipsub_config,
            )
            .map_err(|e| anyhow::anyhow!(e))?;

            let local_peer_id = PeerId::from(key.public());
            let mdns = Mdns::new(MdnsConfig::default(), local_peer_id)?;

            Ok(AppBehaviour { gossipsub, mdns })
        })?
        .build();

    swarm
        .behaviour_mut()
        .gossipsub
        .subscribe(&topic)
        .map_err(|e| anyhow::anyhow!(e))?;
    info!("subscribed to topic '{}'", topic_str);

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse::<Multiaddr>()?)?;

    let (outbound_tx, mut outbound_rx) = unbounded::<Vec<u8>>();
    let (inbound_tx, inbound_rx) = unbounded::<Vec<u8>>();
    let (peers_tx, peers_rx) = unbounded::<usize>();
    let mut peer_set: std::collections::HashSet<PeerId> = Default::default();

    let task = tokio::spawn(async move {
        loop {
            tokio::select! {
                Some(bytes) = outbound_rx.next() => {
                    match swarm.behaviour_mut().gossipsub.publish(topic.clone(), bytes) {
                        Ok(_) => {}
                        Err(e) => info!("gossipsub publish failed: {e:?}"),
                    }
                }
                event = swarm.select_next_some() => {
                    match event {
                        SwarmEvent::NewListenAddr { address, .. } => {
                            info!("listening on {address}");
                        }
                        SwarmEvent::Behaviour(AppBehaviourEvent::Gossipsub(gossipsub::Event::Message{ message, .. })) => {
                            info!("gossipsub message from {} ({} bytes)", message.source.map(|p| p.to_string()).unwrap_or_else(|| "unknown".into()), message.data.len());
                            let _ = inbound_tx.unbounded_send(message.data);
                        }
                        SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                            info!("connection established to {peer_id}");
                        }
                        SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                            info!("dial failed to {:?}: {error:?}", peer_id);
                        }
                        SwarmEvent::IncomingConnectionError { error, .. } => {
                            info!("incoming conn error: {error:?}");
                        }
                        SwarmEvent::Behaviour(AppBehaviourEvent::Gossipsub(gossipsub::Event::Subscribed{ peer_id, topic })) => {
                            info!("peer {peer_id} subscribed to {topic}");
                        }
                        SwarmEvent::Behaviour(AppBehaviourEvent::Gossipsub(gossipsub::Event::Unsubscribed{ peer_id, topic })) => {
                            info!("peer {peer_id} unsubscribed from {topic}");
                        }
                        SwarmEvent::Behaviour(AppBehaviourEvent::Mdns(MdnsEvent::Discovered(list))) => {
                            for (peer, addr) in list {
                                swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer);
                                // Try dialing discovered address to ensure a connection exists
                                let _ = swarm.dial(addr.clone());
                                let _ = swarm.dial(DialOpts::peer_id(peer).build());
                                peer_set.insert(peer);
                            }
                            let _ = peers_tx.unbounded_send(peer_set.len());
                        }
                        SwarmEvent::Behaviour(AppBehaviourEvent::Mdns(MdnsEvent::Expired(list))) => {
                            for (peer, _addr) in list {
                                swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer);
                                peer_set.remove(&peer);
                            }
                            let _ = peers_tx.unbounded_send(peer_set.len());
                        }
                        _ => {}
                    }
                }
            }
        }
    });

    Ok((task, outbound_tx, inbound_rx, peer_id, peers_rx))
}

pub fn serialize_transform(msg: &TransformMsg) -> Result<Vec<u8>> {
    serde_json::to_vec(msg).context("serialize transform".to_string())
}

pub fn deserialize_transform(bytes: &[u8]) -> Result<TransformMsg> {
    serde_json::from_slice(bytes).context("deserialize transform".to_string())
}

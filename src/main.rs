use futures::prelude::*;
use libp2p::{identity, Multiaddr, PeerId};
use libp2p::multiaddr::Protocol;
use libp2p::ping::{Ping, PingConfig};
use libp2p::swarm::{Swarm, SwarmEvent};
use std::error::Error;
use std::net::{SocketAddr, ToSocketAddrs};
use chrono::Local;

fn log(s: String) {
    println!("{}: {}", Local::now(), s);
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);

    let transport = libp2p::development_transport(local_key).await?;

    // Create a ping network behaviour.
    //
    // For illustrative purposes, the ping protocol is configured to
    // keep the connection alive, so a continuous sequence of pings
    // can be observed.
    let behaviour = Ping::new(PingConfig::new().with_keep_alive(true));

    let mut swarm = Swarm::new(transport, behaviour, local_peer_id);

    // Tell the swarm to listen on all interfaces and a random, OS-assigned
    // port.
    swarm.listen_on("/ip4/0.0.0.0/tcp/8080".parse()?)?;

    // Dial the peer identified by the multi-address given as the second
    // command-line argument, if any.
    if let Some(addr) = std::env::args().nth(1) {
        let SocketAddr::V4(sock_addr) = (&addr[..], 8080).to_socket_addrs().unwrap().next().unwrap() else { todo!() };

        let mut remote: Multiaddr = Multiaddr::empty();
        remote.push(Protocol::Ip4(*sock_addr.ip()));
        remote.push(Protocol::Tcp(sock_addr.port()));
        // let remote: Multiaddr = addr.parse()?;
        swarm.dial(remote)?;
        log(format!("Dialed {}", addr))
    }

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => log(format!("Listening on {:?}", address)),
            SwarmEvent::Behaviour(event) => log(format!("{:?}", event)),
            _ => {}
        }
    }
}

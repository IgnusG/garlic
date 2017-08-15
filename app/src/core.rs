use mio::tcp::{TcpStream, TcpListener};
use mio::{Poll, Token, Ready, PollOpt, Events};

use std::net;
use std::net::SocketAddr;
use std::sync::mpsc;
use std::thread;
use std::thread::{JoinHandle};
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};

use errors::*;
use brunch::{send_message, create_connection, create_udp_connection,
    send_udp_message, receive_udp_message, receive_message};
use messages::Message;
use messages::Message::*;
use messages::onion::*;
use messages::onion::Onion::*;
use messages::auth::*;
use messages::auth::Auth::*;
use messages::rps::*;
use messages::rps::Rps::*;
use messages::p2p;
use messages::p2p::P2PMessage;
use config;

// The assumption here being once this counter wraps around previous tunnels/requests should be already dead
static NEXT_TUNNEL_ID: AtomicUsize = ATOMIC_USIZE_INIT;
static NEXT_REQUEST_ID: AtomicUsize = ATOMIC_USIZE_INIT;

struct Communication {
    receiver: mpsc::Receiver<Message>,
    sender: mpsc::Sender<StreamType>,
}
impl Communication {
    fn send(&self, message: Message) {
        self.sender.send(StreamType::API(message));
    }

    fn receive(&self) -> Result<Message> {
        Ok(self.receiver.recv().chain_err(|| "sender diconnected")?)
    }
}

struct AuthSession {
    session_id: u16,
    rps_peer: RpsPeer
}

pub enum StreamType {
    API(Message),
    P2P(Message)
}

fn request_peer(comm: &Communication) -> Result<RpsPeer> {
    comm.send(Rps(Query(RpsQuery {})));
    if let Rps(Peer(rps_peer)) = comm.receive()? {
        Ok(rps_peer)
    } else {
        bail!("protocol breach - expected RpsPeer")
    }
}

fn encrypt_for_all_peers(peers: &Vec<AuthSession>, data: Vec<u8>, comm: &Communication) -> Result<Vec<u8>> {
    let request_id = NEXT_TUNNEL_ID.fetch_add(1, Ordering::SeqCst) as u32;
    comm.send(Auth(CipherEncrypt(AuthCipherCrypt {
        session_id: peers.first().unwrap().session_id,
        request_id: request_id,
        cleartext: true,
        payload: data
    })));

    let data = if let Auth(CipherEncryptResp(message)) = comm.receive()? {
        message.payload
    } else {
        bail!("protocol breach - expected CipherEncryptResp")
    };

    if peers.len() < 2 {
        return Ok(data)
    };

    for peer in &peers[1..] {
        let request_id = NEXT_TUNNEL_ID.fetch_add(1, Ordering::SeqCst) as u32;
        comm.send(Auth(CipherEncrypt(AuthCipherCrypt {
            session_id: peers[0].session_id,
            request_id: request_id,
            cleartext: false,
            payload: data.clone()
        })));

        let data = if let Auth(CipherEncryptResp(message)) = comm.receive()? {
            message.payload
        } else {
            bail!("protocol breach - expected CipherEncryptResp")
        };
    };

    Ok(data)
}

struct Connection {
    udp: Option<net::UdpSocket>,
    tcp: Option<net::TcpStream>
}
impl Connection {
    fn send(&mut self, message: Message) -> Result<()> {
        if let Some(ref mut conn) = self.tcp {
            send_message(conn, message);
        } else if let Some(ref conn) = self.udp {
            send_udp_message(conn, message);
        } else {
            bail!("at least one connection needs to be specified");
        }
        Ok(())
    }

    fn receive(&mut self) -> Result<Message> {
        if let Some(ref mut conn) = self.tcp {
            Ok(receive_message(conn)?)
        } else if let Some(ref conn) = self.udp {
            Ok(receive_udp_message(conn)?)
        } else {
            bail!("at least one connection needs to be specified");
        }
    }
}

fn connect_to_peer(peer: RpsPeer, peers: &Vec<AuthSession>, conf: &config::Config, comm: &Communication) -> Result<AuthSession> {
    let request_id = NEXT_TUNNEL_ID.fetch_add(1, Ordering::SeqCst) as u32;
    comm.send(Auth(SessionStart(AuthSessionStart {
        request_id: request_id,
        hostkey: peer.hostkey.clone()
    })));

    let conn = if peers.len() == 0 {
        let socket = SocketAddr::new(peer.ip_addr, peer.port);
        Connection {
            tcp: Some(create_connection(socket)?),
            udp: None
        }
    } else {
        let peer = &peers.first().unwrap().rps_peer;
        let socket = SocketAddr::new(peer.ip_addr, peer.port);
        Connection {
            udp: Some(create_udp_connection(socket)?),
            tcp: None
        }
    };

    if let Auth(SessionHS1(message)) = comm.receive()? {

        // Send data to other peer

    } else {
        bail!("protocol breach - expected AuthSessionHS1")
    };

    let request_id = NEXT_TUNNEL_ID.fetch_add(1, Ordering::SeqCst) as u32;

    // Receive HS2

    Ok(AuthSession {
        session_id: 0,
        rps_peer: peer
    })
}

fn send_over_data(data: OnionTunnelPayload) -> Result<()> {
    unimplemented!();
}

fn start_dialogue(message: &OnionTunnelBuild, conf: &config::Config, comm: &Communication) {
    trace_labeled_error!( "dialogue encountered a problem", {
        let mut peers = vec![];
        for _ in 0..conf.min_hop_count {
            let peer = request_peer(comm)?;
            let auth_session = connect_to_peer(peer, &peers, conf, comm)?;
            peers.push(auth_session);
        }

        let tunnel_id = NEXT_TUNNEL_ID.fetch_add(1, Ordering::SeqCst) as u32;
        comm.send(Onion(TunnelReady(OnionTunnelPayload {
            tunnel_id: tunnel_id,
            payload: message.hostkey.clone()
        })));

        loop {
            match comm.receive()? {
                Onion(TunnelData(message)) => {
                    send_over_data(message);
                },
                Onion(TunnelDestroy(message)) => {
                    break;
                },
                _ => bail!("protocol breach - expected OnionTunnelData or OnionTunnelDestroy")
            }
        }
    });
}

fn answer_dialogue(message: &P2PMessage, conf: &config::Config, comm: &Communication) {
    unimplemented!();
}

fn spinup_state_machine(message: Message, conf: config::Config, ty: mpsc::Sender<StreamType>)
    -> (mpsc::Sender<Message>, JoinHandle<()>)
{
    let (tx, rx) = mpsc::channel();

    let handle = thread::spawn(move || {
        let message = &message;
        let comm = &Communication {
            receiver: rx,
            sender: ty,
        };

        trace_labeled_error!("failed to create state machine", {
            match *message {
                Onion(TunnelBuild(ref message)) => start_dialogue(message, &conf, &comm),
                P2P(ref message) => {
                    match message.message_type {
                        p2p::P2P::Knock => answer_dialogue(message, &conf, &comm),
                        _ => note!("message {} not part of protocol - discarding")
                    }
                }

                _ => note!("message {} not part of protocol - discarding")
            };
        });
    });

    (tx, handle)
}

pub fn start(rx: &mpsc::Receiver<StreamType>, ty: mpsc::Sender<StreamType>, conf: config::Config)
    -> Result<()> {

    // A loop represents one app round
    loop {
        status!("Waiting for stream");

        // Spinup state machines for received communication
    };

    Ok(())
}

use mio::tcp::{TcpListener, TcpStream};
use mio::{Poll, PollOpt, Token, Events, Ready, Event};
use stoppable_thread;
use stoppable_thread::StoppableHandle;

use std::net;
use std::net::{SocketAddr};
use std::sync::{mpsc};
use std::time::Duration;
use std::io::{Read, Write};

use errors::*;
use messages::{Message, decode_message, encode_message};
use config;
use core;
use core::StreamType;

const LISTENER: Token = Token(0);
const STREAM: Token = Token(1);

pub fn create_async_channel<'a>(listener: &'a TcpListener, stream: Option<&'a TcpStream>) ->
    Result<impl Fn(&'a TcpListener) -> Result<Box<impl Iterator<Item=Result<TcpStream>> + 'a>>>
{
    let poll = Poll::new().chain_err(|| "couln't create poll")?;

    poll.register(listener, LISTENER, Ready::readable(), PollOpt::edge())
        .chain_err(|| "couldn't register listener on poll")?;
    if let Some(stream) = stream {
        poll.register(stream, STREAM, Ready::writable(), PollOpt::edge())
            .chain_err(|| "couldn't register stream on poll")?;
    }

    Ok(move |listener: &'a TcpListener| {

        let mut events = Events::with_capacity(1024);

        poll.poll(&mut events, Some(Duration::from_millis(100)))
            .chain_err(|| "polling failed")?;

        Ok(Box::new(
            vec![0; events.into_iter().filter(|e: &Event| e.token() == LISTENER).count()]
                .into_iter().map(move |_: u16|
                    Ok(listener.accept().chain_err(|| "connection failed")?.0))
        ))
    })
}

// BUG: Due to rust's borrowing system and mio's Polling it is impossible to extract writing the
// stream into a separate thread - reading is therefore done before and only after that is writing done
/** Creates a tcp listener & tcp stream **/
fn create_api_channel(socket: SocketAddr, tx: mpsc::Sender<StreamType>, ry: mpsc::Receiver<StreamType>)
        -> StoppableHandle<()> {
    stoppable_thread::spawn(move |should_die| {
        trace_labeled_panic!("failed to create API tcp channel", {
            let listener = &TcpListener::bind(&socket).chain_err(|| "couldn't create tcp listener")?;
            let mut stream = &TcpStream::connect(&socket).chain_err(|| "couldn't create tcp listener")?;

            let async_incomming = create_async_channel(listener, Some(stream))?;
            note!(format!("successfully connected to API socket at {}", socket));

            while !should_die.get() {
                trace_labeled_error!( "API listener encountered a problem", {
                    for stream in async_incomming(listener)? {
                        let mut buffer = Vec::new();
                        stream?.read_to_end(&mut buffer).chain_err(|| "reading stream failed")?;
                        let message = decode_message(&buffer)?;

                        tx.send(StreamType::API(message))
                            .chain_err(|| "sending stream to core channel failed")?;
                    };
                });

                trace_labeled_error!( "API stream encountered a problem", {
                    if let Ok(packed_message) = ry.try_recv() {
                        let message = match packed_message {
                            StreamType::API(message) => message,
                            _ => bail!("only API messages are allowed here")
                        };

                        stream.write_all(&encode_message(message)?)
                            .chain_err(|| "writing stream failed")?;
                    }
                });
            };
        });
    })
}

fn create_p2p_listener(socket: SocketAddr, tx: mpsc::Sender<StreamType>) -> StoppableHandle<()> {
    stoppable_thread::spawn(move |should_die| {
        trace_labeled_panic!("failed to create P2P tcp listener", {
            let listener = &TcpListener::bind(&socket).chain_err(|| "couldn't create tcp listener")?;
            let async_incomming = create_async_channel(listener, None)?;

            while !should_die.get() {
                trace_labeled_error!( "P2P listener encountered a problem", {
                    for stream in async_incomming(listener)? {
                        let mut buffer = Vec::new();
                        stream?.read_to_end(&mut buffer).chain_err(|| "reading stream failed")?;
                        let message = decode_message(&buffer)?;

                        tx.send(StreamType::API(message))
                            .chain_err(|| "sending stream to core channel failed")?;
                    };
                });
            }
        })
    })
}

pub fn create_connection(socket: SocketAddr) -> Result<net::TcpStream> {
    Ok(net::TcpStream::connect(&socket).chain_err(|| "couldn't create tcp listener")?)
}

pub fn send_message(stream: &mut net::TcpStream, message: Message) -> Result<()> {
    stream.write_all(&encode_message(message)?)
        .chain_err(|| "writing stream failed")?;
    Ok(())
}

pub fn receive_message(stream: &mut net::TcpStream) -> Result<Message> {
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).chain_err(|| "reading stream failed")?;;
    Ok(decode_message(&buffer)?)
}

pub fn create_udp_connection(socket: SocketAddr) -> Result<net::UdpSocket> {
    Ok(net::UdpSocket::bind(&socket).chain_err(|| "failed to create udp connection")?)
}

pub fn send_udp_message(udp_socket: &net::UdpSocket, message: Message) -> Result<()> {
    udp_socket.send(&encode_message(message)?).chain_err(|| "failed to send data on connection")?;
    Ok(())
}

pub fn receive_udp_message(udp_socket: &net::UdpSocket) -> Result<Message> {
    let mut buffer = Vec::new();
    udp_socket.recv(&mut buffer).chain_err(|| "reading socket failed")?;
    Ok(decode_message(&buffer)?)
}

/**
    Brunch: Because nothing beats breakfast & lunch like good ol' garlic bread
    Connects tcp channels to the core module via the core channel
**/
pub fn start (conf: config::Config) -> Result<()> {
    status!("Brunch is served!");

    let (tx, rx) = mpsc::channel();
    let (ty, ry) = mpsc::channel();

    let api_thread_handle = {
        let conf = conf.clone();
        let tx = tx.clone();

        create_api_channel(conf.api_socket, tx, ry)
    };

    let p2p_thread_handle = {
        let conf = conf.clone();

        create_p2p_listener(conf.p2p_socket, tx)
    };

    let core_result = core::start(&rx, ty, conf).chain_err(|| "core routine exited too early");

    api_thread_handle.stop();
    p2p_thread_handle.stop();

    core_result
}

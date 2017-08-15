use errors::*;
use mio::tcp::{TcpListener, TcpStream};
use mio::{Poll, PollOpt, Token, Events, Ready};

use std::time::Duration;

pub fn create_async_listener<'a>(listener: &'a TcpListener) ->
    Result<impl Fn(&'a TcpListener) -> Result<Box<impl Iterator<Item=Result<TcpStream>> + 'a>>>
{
    let poll = Poll::new()
        .chain_err(|| "couldn't create poll")?;

    poll.register(listener, Token(0), Ready::readable() | Ready::writable(), PollOpt::edge())
        .chain_err(|| "couldn't register listener on poll")?;

    Ok(move |listener: &'a TcpListener| {
        let mut events = Events::with_capacity(1024);

        poll.poll(&mut events, Some(Duration::from_millis(100)))
            .chain_err(|| "polling failed")?;

        Ok(Box::new(
            vec![0; events.len() as usize].into_iter().map(move |_: u16|
                Ok(listener.accept().chain_err(|| "connection failed")?.0))
        ))
    })
}

#[macro_use]
mod utilities;
pub mod onion;
pub mod auth;
pub mod rps;
pub mod p2p;

use errors::*;
use messages::auth::*;
use messages::onion::*;
use messages::rps::*;
use messages::p2p::P2PMessage;

use num::FromPrimitive;
use serde::{Deserialize, Serialize};
use rmps::{Deserializer, Serializer};

pub enum Message {
    Onion(Onion),
    Auth(Auth),
    Rps(Rps),
    P2P(P2PMessage)
}

// Ref: 28028854
enum_from_primitive! {
    #[repr(u16)]
    pub enum MessageId {
        RpsQuery = 540,
        RpsPeer = 541,
        OnionTunnelBuild = 560,
        OnionTunnelReady = 561,
        OnionTunnelIncomming = 562,
        OnionTunnelDestroy = 563,
        OnionTunnelData = 564,
        OnionCover = 566,
        OnionError = 565,
        AuthSessionStart = 600,
        AuthSessionHS1 = 601,
        AuthSessionIncommingHS1 = 602,
        AuthSessionHS2 = 603,
        AuthSessionIncommingHS2 = 604,
        AuthCipherEncrypt = 611,
        AuthCipherEncryptResp = 612,
        AuthCipherDecrypt = 613,
        AuthCipherDecryptResp = 614,
        AuthSessionClose = 609,
        AuthSessionError = 610
    }
}

#[allow(or_fun_call)]
pub fn decode_message(bytes: &[u8]) -> Result<Message> {
    use self::Message::*;
    use messages::auth::Auth::*;
    use messages::onion::Onion::*;
    use messages::rps::Rps::*;

    // Quick and dirty hack for current message system
    let mut deserializer = Deserializer::new(bytes);
    let p2p_message = Deserialize::deserialize(&mut deserializer);

    if let Ok(p2p_message) = p2p_message {
        return Ok(Message::P2P(p2p_message));
    }

    let (length, message_type) = unpack_structure!("2H", &bytes[0..4]);
    let length = length as usize;

    if bytes.len() < length {
        bail!("message length is supposed to be {}, but was {}", length, bytes.len());
    }

    let bytes = bytes[4..length].to_vec();

    // TODO: This could use some dedupe refactoring (maybe a procedural macro - but that stuff is difficult to write)
    Ok(match MessageId::from_u16(message_type)
        .ok_or(::errors::Error::from("conversion of message type failed"))? {

        MessageId::OnionTunnelBuild => Onion(TunnelBuild(OnionTunnelBuild::decode(bytes)?)),
        MessageId::OnionTunnelReady => Onion(TunnelReady(OnionTunnelPayload::decode(bytes)?)),
        MessageId::OnionTunnelData => Onion(TunnelData(OnionTunnelPayload::decode(bytes)?)),
        MessageId::OnionTunnelIncomming => Onion(TunnelIncomming(OnionTunnelID::decode(bytes)?)),
        MessageId::OnionTunnelDestroy => Onion(TunnelDestroy(OnionTunnelID::decode(bytes)?)),
        MessageId::OnionCover => Onion(Cover(OnionCover::decode(bytes)?)),

        MessageId::AuthSessionHS1 => Auth(SessionHS1(AuthSessionHS::decode(bytes)?)),
        MessageId::AuthSessionHS2 => Auth(SessionHS2(AuthSessionHS::decode(bytes)?)),
        MessageId::AuthSessionIncommingHS2 => Auth(SessionIncommingHS2(AuthSessionHS::decode(bytes)?)),
        MessageId::AuthCipherEncryptResp => Auth(CipherEncryptResp(AuthCipherCryptResp::decode(bytes)?)),
        MessageId::AuthCipherDecryptResp => Auth(CipherDecryptResp(AuthCipherCryptResp::decode(bytes)?)),
        MessageId::AuthSessionError => Auth(SessionError(AuthSessionError::decode(bytes)?)),

        MessageId::RpsPeer => Message::Rps(Peer(RpsPeer::decode(bytes)?)),

        _ => bail!("message type {} unknown", message_type)
    })
}

pub fn encode_message(message: Message) -> Result<Vec<u8>> {
    use self::Message::*;
    use self::MessageId::*;
    use messages::auth::Auth::*;
    use messages::onion::Onion::*;
    use messages::rps::Rps::*;

    // TODO: ↑ + compiler cannot check a correct call to encode which can result in a bail out
    let (message_id, message) = match message {
        Onion(TunnelReady(message)) => (OnionTunnelReady, message.encode()?),
        Onion(TunnelIncomming(message)) => (OnionTunnelIncomming, message.encode()?),
        Onion(TunnelDestroy(message)) => (OnionTunnelDestroy, message.encode()?),
        Onion(TunnelData(message)) => (OnionTunnelData, message.encode()?),
        Onion(Error(message)) => (OnionError, message.encode()?),

        Auth(SessionStart(message)) => (AuthSessionStart, message.encode()?),
        Auth(SessionHS1(message)) => (AuthSessionHS1, message.encode()?),
        Auth(SessionIncommingHS1(message)) => (AuthSessionIncommingHS1, message.encode()?),
        Auth(SessionHS2(message)) => (AuthSessionHS2, message.encode()?),
        Auth(SessionIncommingHS2(message)) => (AuthSessionIncommingHS2, message.encode()?),
        Auth(CipherEncrypt(message)) => (AuthCipherEncrypt, message.encode()?),
        Auth(CipherDecrypt(message)) => (AuthCipherDecrypt, message.encode()?),
        Auth(SessionClose(message)) => (AuthSessionClose, message.encode()?),

        Rps(Query(message)) => (RpsQuery, message.encode()?),
        
        P2P(message) => {
            let mut bytes = Vec::new();
            message.serialize(&mut Serializer::new(&mut bytes)).chain_err(|| "couldn't serialize P2P message")?;
            return Ok(bytes);
        },

        // BUG: This has to be fixed with a better message system
        _ => panic!("a call to 'encode' that does not exist on this message type was requested")
    };

    let mut bytes = pack_structure!("2H", message.len() as u16 + 4, message_id as u16);
    bytes.extend_from_slice(&message);
    Ok(bytes)
}

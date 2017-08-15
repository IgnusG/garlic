use errors::*;

use bit_field::BitField;

use std::net::{Ipv4Addr, Ipv6Addr, IpAddr};

pub struct OnionTunnelBuild {
    pub onion_tunnel: u16,
    pub ip_addr: IpAddr,
    pub hostkey: Vec<u8>
}
/* 1B Reserved | 7b1b IPv | 2B OnionTunnel | 16B/4B IP | Rest Hostkey */
impl OnionTunnelBuild {
    pub fn decode(bytes: Vec<u8>) -> Result<OnionTunnelBuild> {
        let (ipv, onion_tunnel) = unpack_structure!("xBH", &bytes);

        let (next_field_offset, ip_addr) = if ipv.get_bit(0) {
            let (i0, i1, i2, i3, i4, i5, i6, i7) = unpack_structure!("8H", &bytes[4..18]);
            (18, IpAddr::V6(Ipv6Addr::new(i0, i1, i2, i3, i4, i5, i6, i7)))
        } else {
            let (i0, i1, i2, i3) = unpack_structure!("4B", &bytes[4..8]);
            (8, IpAddr::V4(Ipv4Addr::new(i0, i1, i2, i3)))
        };

        Ok(OnionTunnelBuild {
            onion_tunnel: onion_tunnel,
            ip_addr: ip_addr,
            hostkey: bytes[next_field_offset..].to_vec()
        })
    }
}

pub struct OnionTunnelPayload {
    pub tunnel_id: u32,
    pub payload: Vec<u8>
}
/* 4B TunnelId | Rest Payload */
impl OnionTunnelPayload {
    pub fn decode(bytes: Vec<u8>) -> Result<OnionTunnelPayload> {
        let (tunnel_id,) = unpack_structure!("I", &bytes);
        Ok(OnionTunnelPayload {
            tunnel_id: tunnel_id,
            payload: bytes[4..].to_vec()
        })
    }
    pub fn encode(self) -> Result<Vec<u8>> {
        let mut bytes = pack_structure!("I", self.tunnel_id);
        bytes.extend_from_slice(&self.payload);
        Ok(bytes)
    }
}

pub struct OnionTunnelID {
    pub tunnel_id: u32
}
/* 4B TunnelId */
impl OnionTunnelID {
    pub fn decode(bytes: Vec<u8>) -> Result<OnionTunnelID> {
        let (tunnel_id,) = unpack_structure!("I", &bytes);
        Ok(OnionTunnelID {
            tunnel_id: tunnel_id
        })
    }
    pub fn encode(self) -> Result<Vec<u8>> {
        Ok(pack_structure!("I", self.tunnel_id))
    }
}

pub struct OnionError {
    pub tunnel_id: u32,
    pub request_type: u16
}
/* 2B RequestType | 2B Reserved | 4B TunnelId */
impl OnionError {
    pub fn encode(self) -> Result<Vec<u8>> {
        Ok(pack_structure!("H2xI", self.request_type, self.tunnel_id))
    }
}

pub struct OnionCover {
    pub cover_size: u16,
}
/* 2B CoverSize | 2B Reserved */
impl OnionCover {
    pub fn decode(bytes: Vec<u8>) -> Result<OnionCover> {
        let (cover_size,) = unpack_structure!("H2x", &bytes);
        Ok(OnionCover {
            cover_size: cover_size,
        })
    }
}

pub enum Onion {
    TunnelBuild(OnionTunnelBuild),
    TunnelReady(OnionTunnelPayload),
    TunnelIncomming(OnionTunnelID),
    TunnelDestroy(OnionTunnelID),
    TunnelData(OnionTunnelPayload),
    Cover(OnionCover),
    Error(OnionError)
}

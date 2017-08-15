use errors::*;

use bit_field::BitField;

use std::net::{Ipv4Addr, Ipv6Addr, IpAddr};

pub struct RpsQuery {}
impl RpsQuery {
    pub fn encode(self) -> Result<Vec<u8>> {
        Ok(vec![])
    }
}

pub struct RpsPeer {
    pub port: u16,
    pub ip_addr: IpAddr,
    pub hostkey: Vec<u8>
}
/* 2B Port | 1B Reserved | 7b1b IPv | Rest Hostkey */
impl RpsPeer {
    pub fn decode(bytes: Vec<u8>) -> Result<RpsPeer> {
        let (port, ipv) = unpack_structure!("HxB", &bytes);

        let (next_field_offset, ip_addr) = if ipv.get_bit(0) {
            let (i0, i1, i2, i3, i4, i5, i6, i7) = unpack_structure!("8H", &bytes[4..18]);
            (18, IpAddr::V6(Ipv6Addr::new(i0, i1, i2, i3, i4, i5, i6, i7)))
        } else {
            let (i0, i1, i2, i3) = unpack_structure!("4B", &bytes[4..8]);
            (8, IpAddr::V4(Ipv4Addr::new(i0, i1, i2, i3)))
        };

        Ok(RpsPeer {
            port: port,
            ip_addr: ip_addr,
            hostkey: bytes[next_field_offset..].to_vec()
        })
    }
}

pub enum Rps {
    Query(RpsQuery),
    Peer(RpsPeer)
}

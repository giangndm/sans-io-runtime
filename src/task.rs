use std::{net::SocketAddr, time::Instant};

use crate::BusEvent;

pub enum NetIncoming<'a> {
    UdpListenResult {
        bind: SocketAddr,
        result: Result<SocketAddr, std::io::Error>,
    },
    UdpPacket {
        from: SocketAddr,
        to: SocketAddr,
        data: &'a [u8],
    },
}

pub enum NetOutgoing<'a> {
    UdpListen(SocketAddr),
    UdpPacket {
        from: SocketAddr,
        to: SocketAddr,
        data: &'a [u8],
    },
}

pub enum Input<'a, Ext, MSG> {
    Net(NetIncoming<'a>),
    Bus(MSG),
    External(Ext),
}

pub enum Output<'a, Ext, ChannelId, MSG> {
    Net(NetOutgoing<'a>),
    Bus(BusEvent<ChannelId, MSG>),
    External(Ext),
    Destroy,
}

pub trait Task<ExtIn, ExtOut, ChannelId, MSG, Cfg> {
    fn build(cfg: Cfg) -> Self;
    fn on_tick(&mut self, now: Instant);
    fn on_input<'a>(&mut self, now: Instant, input: Input<'a, ExtIn, MSG>);
    fn pop_output(&mut self, now: Instant) -> Option<Output<'_, ExtOut, ChannelId, MSG>>;
}

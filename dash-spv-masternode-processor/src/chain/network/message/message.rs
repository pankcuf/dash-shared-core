use std::io;
use std::io::{Error, Write};
use byte::ctx::{Endian, NULL, Str};
use byte::{BytesExt, TryRead};
use hashes::hex::ToHex;
use crate::chain::common::ChainType;
use crate::chain::network::message::addr::Addr;
use crate::chain::network::message::inventory::Inventory;
use crate::chain::network::message::response::Response;
use crate::chain::network::message::version::Version;
use crate::chain::network::Request;
use crate::chain::tx;
use crate::consensus::{encode, ReadExt};
use crate::consensus::encode::{CheckedData, Decodable, Encodable};
use crate::manager::peer_manager;
use crate::network::p2p::buffer::{Buffer, PassThroughBufferReader};
use crate::network::p2p::state::PeerState;
use crate::network::p2p::state_flags::PeerStateFlags;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum MessageType {
    #[default]
    WrongType,
    Version,
    Verack,
    Addr,
    AddrV2,
    Inv,
    Getdata,
    NotFound,
    Getblocks,
    Getheaders,
    Tx,
    // #[deprecated] // deprecated in version 14
    Ix,
    // #[deprecated] // deprecated in version 14
    Txlvote,
    Islock, // version 14
    Isdlock, // version 14
    Block,
    Chainlock,
    Headers,
    Getaddr,
    Mempool,
    Ping,
    Pong,
    Filterload,
    Filteradd,
    Filterclear,
    Merkleblock,
    Alert,
    // BIP61: https://github.com/bitcoin/bips/blob/master/bip-0061.mediawiki
    Reject,
    // BIP130: https://github.com/bitcoin/bips/blob/master/bip-0130.mediawiki
    Sendheaders,
    // BIP133: https://github.com/bitcoin/bips/blob/master/bip-0133.mediawiki
    Feefilter,
    Senddsq, // version 14
    Sendcmpct, // version 12.3
    Sendaddrv2, // version 12.3

    //Dash specific
    //Control
    Spork,
    GetSporks,

    //Masternode
    Dseg,
    // #[deprecated] // deprecated since version 70211
    Mnb,
    Mnget,
    Mnp,
    Mnv,
    Mnw,
    Mnwb,
    Ssc,
    Getmnlistd,
    Mnlistdiff,
    Qrinfo,
    Getqrinfo,
    //Governance
    Govobj,
    Govobjvote,
    Govsync,
    //Private send
    DarkSendAnnounce,
    DarkSendControl,
    DarkSendFinish,
    DarkSendInitiate,
    DarkSendQuorum,
    DarkSendSession,
    DarkSendSessionUpdate,
    DarkSendTX,
}

// Regex
// (Type::[A-Za-z]+) => (\"[A-Za-z]+\")
// $2 => $1
impl From<&str> for MessageType {
    fn from(orig: &str) -> Self {
        match orig {
            "version" => MessageType::Version,
            "verack" => MessageType::Verack,
            "addr" => MessageType::Addr,
            "inv" => MessageType::Inv,
            "getdata" => MessageType::Getdata,
            "notfound" => MessageType::NotFound,
            "getblocks" => MessageType::Getblocks,
            "getheaders" => MessageType::Getheaders,
            "tx" => MessageType::Tx,
            "ix" => MessageType::Ix,
            "txlvote" => MessageType::Txlvote,
            "islock" => MessageType::Islock,
            "isdlock" => MessageType::Isdlock,
            "block" => MessageType::Block,
            "clsig" => MessageType::Chainlock,
            "headers" => MessageType::Headers,
            "getaddr" => MessageType::Getaddr,
            "mempool" => MessageType::Mempool,
            "ping" => MessageType::Ping,
            "pong" => MessageType::Pong,
            "filterload" => MessageType::Filterload,
            "filteradd" => MessageType::Filteradd,
            "filterclear" => MessageType::Filterclear,
            "merkleblock" => MessageType::Merkleblock,
            "alert" => MessageType::Alert,
            "reject" => MessageType::Reject,
            "sendheaders" => MessageType::Sendheaders,
            "feefilter" => MessageType::Feefilter,
            "senddsq" => MessageType::Senddsq,
            "sendcmpct" => MessageType::Sendcmpct,
            "sendaddrv2" => MessageType::Sendaddrv2,
            "spork" => MessageType::Spork,
            "getsporks" => MessageType::GetSporks,
            "dseg" => MessageType::Dseg,
            "mnb" => MessageType::Mnb,
            "mnget" => MessageType::Mnget,
            "mnp" => MessageType::Mnp,
            "mnv" => MessageType::Mnv,
            "mnw" => MessageType::Mnw,
            "mnwb" => MessageType::Mnwb,
            "ssc" => MessageType::Ssc,
            "getmnlistd" => MessageType::Getmnlistd,
            "mnlistdiff" => MessageType::Mnlistdiff,
            "qrinfo" => MessageType::Qrinfo,
            "getqrinfo" => MessageType::Getqrinfo,
            "govobj" => MessageType::Govobj,
            "govobjvote" => MessageType::Govobjvote,
            "govsync" => MessageType::Govsync,
            "dsa" => MessageType::DarkSendAnnounce,
            "dsc" => MessageType::DarkSendControl,
            "dsf" => MessageType::DarkSendFinish,
            "dsi" => MessageType::DarkSendInitiate,
            "dsq" => MessageType::DarkSendQuorum,
            "dss" => MessageType::DarkSendSession,
            "dssu" => MessageType::DarkSendSessionUpdate,
            "dstx" => MessageType::DarkSendTX,

            _ => MessageType::WrongType
        }
    }
}

impl From<MessageType> for &str {
    fn from(value: MessageType) -> Self {
        match value {
            MessageType::WrongType => "",
            MessageType::Version => "version",
            MessageType::Verack => "verack",
            MessageType::Addr => "addr",
            MessageType::AddrV2 => "addrv2",
            MessageType::Inv => "inv",
            MessageType::Getdata => "getdata",
            MessageType::NotFound => "notfound",
            MessageType::Getblocks => "getblocks",
            MessageType::Getheaders => "getheaders",
            MessageType::Tx => "tx",
            MessageType::Ix => "ix",
            MessageType::Txlvote => "txlvote",
            MessageType::Islock => "islock",
            MessageType::Isdlock => "isdlock",
            MessageType::Block => "block",
            MessageType::Chainlock => "clsig",
            MessageType::Headers => "headers",
            MessageType::Getaddr => "getaddr",
            MessageType::Mempool => "mempool",
            MessageType::Ping => "ping",
            MessageType::Pong => "pong",
            MessageType::Filterload => "filterload",
            MessageType::Filteradd => "filteradd",
            MessageType::Filterclear => "filterclear",
            MessageType::Merkleblock => "merkleblock",
            MessageType::Alert => "alert",
            MessageType::Reject => "reject",
            MessageType::Sendheaders => "sendheaders",
            MessageType::Feefilter => "feefilter",
            MessageType::Senddsq => "senddsq",
            MessageType::Sendcmpct => "sendcmpct",
            MessageType::Sendaddrv2 => "sendaddrv2",
            MessageType::Spork => "spork",
            MessageType::GetSporks => "getsporks",
            MessageType::Dseg => "dseg",
            MessageType::Mnb => "mnb",
            MessageType::Mnget => "mnget",
            MessageType::Mnp => "mnp",
            MessageType::Mnv => "mnv",
            MessageType::Mnw => "mnw",
            MessageType::Mnwb => "mnwb",
            MessageType::Ssc => "ssc",
            MessageType::Getmnlistd => "getmnlistd",
            MessageType::Mnlistdiff => "mnlistdiff",
            MessageType::Qrinfo => "qrinfo",
            MessageType::Getqrinfo => "getqrinfo",
            MessageType::Govobj => "govobj",
            MessageType::Govobjvote => "govobjvote",
            MessageType::Govsync => "govsync",
            MessageType::DarkSendAnnounce => "dsa",
            MessageType::DarkSendControl => "dsc",
            MessageType::DarkSendFinish => "dsf",
            MessageType::DarkSendInitiate => "dsi",
            MessageType::DarkSendQuorum=> "dsq",
            MessageType::DarkSendSession => "dss",
            MessageType::DarkSendSessionUpdate => "dssu",
            MessageType::DarkSendTX => "dstx",
        }
    }
}
impl From<MessageType> for String {
    fn from(value: MessageType) -> Self {
        let str: &str = value.into();
        str.to_string()
    }
}

impl Encodable for MessageType {
    fn consensus_encode<W: Write>(&self, mut writer: W) -> Result<usize, Error> {
        let message_type = <MessageType as Into<[u8; 12]>>::into(*self);
        message_type.enc(&mut writer);
        Ok(12)
    }
}

impl Decodable for MessageType {
    fn consensus_decode<D: io::Read>(mut d: D) -> Result<Self, encode::Error> {
        // d.read_b
        let mut ret = vec![0u8; 12];
        d.read_slice(&mut ret)?;
        // Ok(ret).expect("TODO: panic message")
        let pos = ret.iter().position(|&c| c == 0).unwrap_or(ret.len());
        let str = String::from_utf8_lossy(&ret[..pos]);
        debug!("MessageType.decode: {}", str);
        Ok(MessageType::from(str.trim()))

        // Here, you would read into the buffer. I'll just demonstrate with hardcoded values.

        // Find the position of the first null terminator

        // Convert the sub-slice up to the found position to a string
        // let result = str::from_utf8(&buffer[..pos]).expect("Found invalid UTF-8");

        // String::from_utf8_lossy(&ret[..pos])
        //     .map_err(|err| err.into())
        //     .map(|str| {
        //         debug!("MessageType.decode: {}", str);
        //         MessageType::from(str.as_str())
        //     })
        // match String::from_utf8(ret) {
        //     Ok(str) => Ok(MessageType::from(str.as_str())),
        //     Err(err) => encode::Error::ParseFailed(&*err.to_string()),
        // }

        // String::consensus_decode(d)
        //     .map(|s| MessageType::from(s.as_str()))
    }
}

impl<'a> TryRead<'a, Endian> for MessageType {
    fn try_read(bytes: &'a [u8], _endian: Endian) -> byte::Result<(Self, usize)> {
        let offset: &mut usize = &mut 0;
        let str = bytes.read_with::<&str>(offset, Str::Delimiter(NULL)).unwrap();
        Ok((str.into(), *offset))
    }
}

impl From<MessageType> for [u8; 12] {
    fn from(value: MessageType) -> Self {
        let mut command = [0u8; 12];
        let s: &str = value.into();
        let bytes = s.as_bytes();
        command[..bytes.len()].copy_from_slice(bytes);
        command
    }
}

pub trait Payload: Sized + Clone {
    fn r#type(&self) -> MessageType;
}

#[derive(Clone, Debug)]
pub enum Direction {
    Incoming,
    Outgoing
}

impl Direction {
    pub fn message_with(self, r#type: MessageType, payload: MessagePayload) -> Message {
        Message {
            r#type,
            payload,
            direction: self,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CheckedPayload {
    pub(crate) message_type: MessageType,
    pub(crate) payload: CheckedData,
}


impl Payload for CheckedPayload {
    fn r#type(&self) -> MessageType {
        self.message_type
    }
}

impl io::Read for CheckedPayload {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        todo!()
        // self.read(buf)
    }
}

#[derive(Clone, Debug)]
pub enum MessagePayload {
    Request(Request),
    Response(Response),
    Checked(CheckedPayload),
}

impl Payload for MessagePayload {
    fn r#type(&self) -> MessageType {
        match self {
            Self::Request(request) => request.r#type(),
            Self::Response(response) => response.r#type(),
            Self::Checked(payload) => payload.r#type(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Message {
    // pub magic: u32,
    pub r#type: MessageType,
    pub direction: Direction,
    pub payload: MessagePayload
}

impl From<MessagePayload> for Message {
    fn from(value: MessagePayload) -> Self {
        Message {
            r#type: value.r#type(),
            direction: Direction::Incoming,
            payload: value
        }
    }
}

impl Message {

    pub fn compile(&self, magic: u32) -> Vec<u8> {
        if let MessagePayload::Request(ref request) = self.payload {
            // [self appendUInt32:chain.magicNumber];
            // [self appendNullPaddedString:type length:12];
            // [self appendUInt32:(uint32_t)message.length];
            // [self appendBytes:message.SHA256_2.u32 length:4];
            // [self appendBytes:message.bytes length:message.length];
            // let payload = request.compile();
            // let len = payload.len() as u32;
            // let mut writer = Vec::<u8>::new();
            // magic.enc(&mut writer);
            // self.r#type.enc(&mut writer);
            // len.enc(&mut writer);
            // sha256d::Hash::hash(&payload).enc(&mut writer);
            // writer.extend_from_slice(&payload);
            // writer.copy_from_slice(&payload);
            // writer
            return request.compile();
        } else {
            panic!("Can compile only request with payload")
        }
    }

    pub fn decompile<P: PeerState>(&self, context: &P) -> Result<Response, peer_manager::Error> {
        match self.r#type {
            MessageType::Verack => Ok(Response::Verack),
            MessageType::Version =>
                self.unpack::<Version, ChainType>(context.chain_type())
                    .map(Response::Version),
            MessageType::Addr =>
                self.unpack::<Addr, bool>(context.flags().contains(PeerStateFlags::SENT_GETADDR))
                    .map(Response::Addr),
            MessageType::Inv =>
                self.unpack::<Inventory, &P>(context)
                    .map(Response::Inventory),
            MessageType::Tx =>
                self.unpack::<tx::Kind, &P>(context)
                    .map(Response::Tx),
            _ => Err(peer_manager::Error::Default(format!("Unknown message type")))
        }
    }

    pub fn unpack<'a, T: TryRead<'a, Ctx>, Ctx>(&self, context: Ctx) -> Result<T, peer_manager::Error> {

        // data.read_with::<T>(&mut 0, context)
        //     .map_err(|err| peer_manager::Error::from(err))
        todo!()
        // self.payload.read_with::<T>(&mut 0, context)
        //     .map_err(|err| peer_manager::Error::from(err))
    }

    pub fn from_buffer(buffer: &mut Buffer) -> Result<Message, encode::Error> {
        let reader = PassThroughBufferReader::new(buffer);
        let mut finite_reader = reader.take_max();
        let magic = u32::consensus_decode(&mut finite_reader)?;
        let message_type = MessageType::consensus_decode(&mut finite_reader)?;
        let payload = CheckedData::consensus_decode(&mut finite_reader)?;
        let checked_payload = CheckedPayload { message_type, payload };
        println!("from_buffer: {:?} {:?}", checked_payload.message_type, checked_payload.payload.0.to_hex());
        Ok(Direction::Incoming.message_with(message_type, MessagePayload::Checked(checked_payload)))
    }

}

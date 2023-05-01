use crate::chain::network::message::version::Version;
use crate::chain::network::message::addr::Addr;
use crate::chain::network::message::inventory::Inventory;
use crate::chain::network::message::message::Payload;
use crate::chain::network::MessageType;
use crate::chain::tx;

// pub type Res = Result<Response, peer_manager::Error>;

#[derive(Clone)]
pub enum Response {
    Unknown,
    Version(Version),
    Verack,
    Addr(Addr),
    Inventory(Inventory),
    Tx(tx::Kind),
    // AddrV2(Addr),

    // GetBlocks(Vec<UInt256>, UInt256, u32),
    // GetHeaders(Vec<UInt256>, UInt256, u32),
}

impl Payload for Response {

    fn r#type(&self) -> MessageType {
        match self {
            Response::Unknown => MessageType::WrongType,
            Response::Version(_) => MessageType::Version,
            Response::Verack => MessageType::Verack,
            Response::Inventory(_) => MessageType::Inv,
            Response::Addr(_) => MessageType::Addr,
            Response::Tx(_) => MessageType::Tx,
        }
    }
}

// pub enum Response {
//     Version,
//     Inventory,
// }
//
// impl<Ctx> TryFrom<(&[u8], Ctx)> for Response {
//     type Error = byte::Error;
//
//     fn try_from(value: (&[u8], Ctx)) -> Result<Self, Self::Error> {
//         value.0.read_with::<>(&mut 0, value.1)
//         match  { }
//     }
// }

// impl<'a, Ctx, T: TryRead<'a, Ctx>, CtxSolve: Fn() -> Ctx> TryRead<'a, Ctx> for Response<T> {
//     fn try_read(bytes: &'a [u8], context: CtxSolve) -> byte::Result<(Self, usize)> {
//         // let offset = &mut 0;
//         // let r#type = bytes.read_with::<InvType>(offset, endian)?;
//         // let hash = bytes.read_with::<UInt256>(offset, endian)?;
//         // Ok((Self { r#type, hash }, *offset))
//         bytes.read_with::<T>(&mut 0, context())
//         // match Self {
//         //
//         //     Response::Version(chain_type) => bytes.read_with::<Version>(&mut 0, context())
//         // }
//         // message.read_with::<T>(&mut 0, context)
//     }
// }

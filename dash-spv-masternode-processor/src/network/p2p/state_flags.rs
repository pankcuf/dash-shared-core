bitflags! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct PeerStateFlags: u32 {
        const EMPTY = 0;
        const SENT_GETADDR = 1 << 0;
        const SENT_GETDATATXBLOCKS = 1 << 1;
        const SENT_GETDATAMASTERNODE = 1 << 2;
        const SENT_FILTER = 1 << 3;
        const SENT_GETBLOCKS = 1 << 4;
        const SENT_GETHEADERS = 1 << 5;
        const SENT_MEMPOOL = 1 << 6;
        const SENT_VERACK = 1 << 7;
        const GOT_VERACK = 1 << 8;
    }
}

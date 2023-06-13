use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr::null_mut;
use crate::ffi::{FromFFI, ToFFI};
use crate::ffi::boxer::boxed;
use rs_dapi_client::rs_dapi_client;

#[repr(C)]
pub struct Version {
    pub protocol: u32,
    pub software: u32,
    pub agent: *mut c_char,
}

impl ToFFI for rs_dapi_client::get_status_response::Version {
    type Item = Version;

    fn encode(&self) -> Self::Item {
        Self::Item {
            protocol: self.protocol,
            software: self.software,
            agent: CString::new(self.agent.clone()).unwrap().into_raw()
        }
    }
}

impl FromFFI for Version {
    type Item = rs_dapi_client::get_status_response::Version;

    unsafe fn decode(&self) -> Self::Item {
        Self::Item {
            protocol: self.protocol,
            software: self.software,
            agent: unsafe { CStr::from_ptr(self.agent) }.to_str().unwrap().to_string(),
        }
    }
}

#[repr(C)]
pub struct Time {
    pub now: u32,
    pub offset: i32,
    pub median: u32,
}

impl ToFFI for rs_dapi_client::get_status_response::Time {
    type Item = Time;

    fn encode(&self) -> Self::Item {
        Self::Item {
            now: self.now,
            offset: self.offset,
            median: self.median,
        }
    }
}

impl FromFFI for Time {
    type Item = rs_dapi_client::get_status_response::Time;

    unsafe fn decode(&self) -> Self::Item {
        Self::Item {
            now: self.now,
            offset: self.offset,
            median: self.median,
        }
    }
}


#[repr(C)]
pub struct Chain {
    pub name: *mut c_char,
    pub headers_count: u32,
    pub blocks_count: u32,
    pub best_block_hash: *mut [u8; 32],
    pub difficulty: f64,
    pub chain_work: *mut [u8; 32],
    pub is_synced: bool,
    pub sync_progress: f64,
}

impl ToFFI for rs_dapi_client::get_status_response::Chain {
    type Item = Chain;

    fn encode(&self) -> Self::Item {
        let best_block_hash: [u8; 32] = self.best_block_hash.clone().try_into().unwrap();
        let chain_work: [u8; 32] = self.chain_work.clone().try_into().unwrap();
        Self::Item {
            name: CString::new(self.name.clone()).unwrap().into_raw(),
            headers_count: self.headers_count,
            blocks_count: self.blocks_count,
            best_block_hash: boxed(best_block_hash),
            difficulty: self.difficulty,
            chain_work: boxed(chain_work),
            is_synced: self.is_synced,
            sync_progress: self.sync_progress,
        }
    }
}

impl FromFFI for Chain {
    type Item = rs_dapi_client::get_status_response::Chain;

    unsafe fn decode(&self) -> Self::Item {
        Self::Item {
            name: unsafe { CStr::from_ptr(self.name) }.to_str().unwrap().to_string(),
            headers_count: self.headers_count,
            blocks_count: self.blocks_count,
            best_block_hash: Vec::from(unsafe { *self.best_block_hash }),
            difficulty: self.difficulty,
            chain_work: Vec::from(unsafe { *self.chain_work }),
            is_synced: self.is_synced,
            sync_progress: self.sync_progress,
        }
    }
}

#[repr(C)]
pub struct Masternode {
    pub status: i32,
    pub pro_tx_hash: *mut [u8; 32],
    pub pose_penalty: u32,
    pub is_synced: bool,
    pub sync_progress: f64,
}

impl ToFFI for rs_dapi_client::get_status_response::Masternode {
    type Item = Masternode;

    fn encode(&self) -> Self::Item {
        let pro_tx_hash: [u8; 32] = self.pro_tx_hash.clone().try_into().unwrap();
        Self::Item {
            status: self.status,
            pro_tx_hash: boxed(pro_tx_hash),
            pose_penalty: self.pose_penalty,
            is_synced: self.is_synced,
            sync_progress: self.sync_progress,
        }
    }
}

impl FromFFI for Masternode {
    type Item = rs_dapi_client::get_status_response::Masternode;

    unsafe fn decode(&self) -> Self::Item {
        Self::Item {
            status: self.status,
            pro_tx_hash: Vec::from(unsafe { *self.pro_tx_hash }),
            pose_penalty: self.pose_penalty,
            is_synced: self.is_synced,
            sync_progress: self.sync_progress,
        }
    }
}

#[repr(C)]
pub struct Network {
    pub peers_count: u32,
    pub fee: *mut NetworkFee,
}

impl ToFFI for rs_dapi_client::get_status_response::Network {
    type Item = Network;

    fn encode(&self) -> Self::Item {
        Self::Item {
            peers_count: self.peers_count,
            fee: self.fee.as_ref().map_or(null_mut(), |v| boxed(v.encode())),
        }
    }
}

impl FromFFI for Network {
    type Item = rs_dapi_client::get_status_response::Network;

    unsafe fn decode(&self) -> Self::Item {
        Self::Item {
            peers_count: self.peers_count,
            fee: (!self.fee.is_null()).then_some((*self.fee).decode()),
        }
    }
}

#[repr(C)]
pub struct NetworkFee {
    pub relay: f64,
    pub incremental: f64,
}

impl FromFFI for NetworkFee {
    type Item = rs_dapi_client::get_status_response::NetworkFee;

    unsafe fn decode(&self) -> Self::Item {
        Self::Item {
            relay: self.relay,
            incremental: self.incremental,
        }
    }
}

impl ToFFI for rs_dapi_client::get_status_response::NetworkFee {
    type Item = NetworkFee;

    fn encode(&self) -> Self::Item {
        Self::Item {
            relay: self.relay,
            incremental: self.incremental,
        }
    }
}

#[repr(C)]
pub struct GetStatusResponse {
    pub version: *mut Version,
    pub time: *mut Time,
    pub status: i32,
    pub sync_progress: f64,
    pub chain: *mut Chain,
    pub masternode: *mut Masternode,
    pub network: *mut Network,
}
unsafe impl Send for GetStatusResponse {}


impl FromFFI for GetStatusResponse {
    type Item = rs_dapi_client::GetStatusResponse;

    unsafe fn decode(&self) -> Self::Item {
        Self::Item {
            version: (!self.version.is_null()).then_some((*self.version).decode()),
            time: (!self.time.is_null()).then_some((*self.time).decode()),
            status: self.status,
            sync_progress: self.sync_progress,
            chain: (!self.chain.is_null()).then_some((*self.chain).decode()),
            masternode: (!self.masternode.is_null()).then_some((*self.masternode).decode()),
            network: (!self.network.is_null()).then_some((*self.network).decode()),
        }
    }
}

impl ToFFI for rs_dapi_client::GetStatusResponse {
    type Item = GetStatusResponse;

    fn encode(&self) -> Self::Item {
        Self::Item {
            version: self.version.as_ref().map_or(null_mut(), |v| boxed(v.encode())),
            time: self.time.as_ref().map_or(null_mut(), |v| boxed(v.encode())),
            status: self.status,
            sync_progress: self.sync_progress,
            chain: self.chain.as_ref().map_or(null_mut(), |v| boxed(v.encode())),
            masternode: self.masternode.as_ref().map_or(null_mut(), |v| boxed(v.encode())),
            network: self.network.as_ref().map_or(null_mut(), |v| boxed(v.encode())),
        }
    }
}

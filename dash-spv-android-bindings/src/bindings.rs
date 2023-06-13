use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
use std::thread;
use rs_dapi_client::rs_dapi_client::GetStatusResponse;
use tokio::runtime::Runtime;
use tonic::Response;
use dash_spv_masternode_processor::ffi::boxer::boxed;
use dash_spv_masternode_processor::ffi::unboxer::unbox_any;
use crate::core::core;



#![allow(dead_code)]
#![allow(unused_variables)]

pub mod bindings;
pub mod core;

pub extern crate dash_spv_masternode_processor;

use std::{os::raw::{c_char, c_void}, thread};
use std::ffi::CStr;
use tokio::{runtime::Runtime, time::{sleep, Duration}};
use rs_dapi_client::{CoreClient, DAPIClient, GetStatusRequest};
use rs_dapi_client::rs_dapi_client::GetStatusResponse;





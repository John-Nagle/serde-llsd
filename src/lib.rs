//
//  lib.rs
//
//  Rust library for serializing and de-serializing data in
//  Linden Lab Structured Data format.
//
//  Serde version.
//
//  Format documentation is at http://wiki.secondlife.com/wiki/LLSD
//
//  Animats
//  October, 2021.
//  License: LGPL.
//
//
//  Modules
//
pub mod ser;
pub mod de;
pub mod error;  

pub use crate::{
    ser::{
        xml::to_string,
        binary::{ to_bytes, to_writer, to_writer_buffered },
    },
    de::{
        xml::from_str,
        binary::{ from_bytes, from_reader, from_reader_buffered },
    },
};

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
        xml::to_writer,
        ////	binary,////::{ to_bytes, to_writer, to_writer_buffered },
    },
    de::{
        xml::from_str,
        xml::from_reader,
        ////binary,////	{ from_bytes, from_reader, from_reader_buffered },
    },
};

use std::collections::HashMap;
use uuid::Uuid;
use enum_as_inner::{EnumAsInner};

///  The primitive LLSD data item.
#[derive(Debug, Clone, PartialEq, EnumAsInner)]
pub enum LLSDValue {
    Undefined,
    Boolean(bool),
    Real(f64),
    Integer(i32),
    UUID(Uuid),
    String(String),
    Date(i64),
    URI(String),
    Binary(Vec<u8>),
    Map(HashMap<String, LLSDValue>),
    Array(Vec<LLSDValue>),
}

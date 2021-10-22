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
pub mod de;
pub mod error;
pub mod ser;

pub use crate::{
    de::{
        binary::from_bytes,
        ////binary::from_reader,    // Name clash
        xml::from_reader,
        xml::from_str,
    },
    ser::{
        binary::to_bytes,
        ////binary::to_writer,  // Name clash
        xml::to_string,
        xml::to_writer,
    },
};

use enum_as_inner::EnumAsInner;
use std::collections::HashMap;
use uuid::Uuid;

/// The primitive LLSD data item.
/// Serialization takes a tree of these.
/// Deserialization returns a tree of these.
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

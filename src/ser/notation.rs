//! # ser/notation -- serialize LLSD, notation form.
//!
//!  Library for serializing and de-serializing data in
//!  Linden Lab Structured Data format.
//!
//!  Format documentation is at http://wiki.secondlife.com/wiki/LLSD
//!
//!  Notation format, serialization.
//
//  Animats
//  March, 2021.
//  License: LGPL.
//
use crate::LLSDValue;
use anyhow::Error;
use std::io::Write;
//
//  Constants
//
/// Notation LLSD prefix
pub const LLSDNOTATIONPREFIX: &str = "<? llsd/notation ?>\n"; 
/// Sentinel, must match exactly.
pub const LLSDNOTATIONSENTINEL: &str = LLSDNOTATIONPREFIX; 

/// Outputs an LLSDValue as a string of bytes, in LLSD "binary" format.
pub fn to_string(val: &LLSDValue) -> Result<String, Error> {
    let mut writer = String::new();
    writer.push_str(LLSDNOTATIONPREFIX); // prefix
    generate_value(&mut writer, val)?;
    Ok(writer)
}

/*
/// Outputs an LLSD value to an output stream
pub fn to_writer<W: Write>(writer: &mut W, val: &LLSDValue) -> Result<(), Error> {
    writer.write_all(LLSDNOTATIONPREFIX)?; // prefix
    generate_value(writer, val)?;
    writer.flush()?;
    Ok(())
}
*/
/// Generate one <TYPE> VALUE </TYPE> output. VALUE is recursive.
fn generate_value(writer: &mut String, val: &LLSDValue) -> Result<(), Error> {
    //  Emit notation form for all possible types.
    match val {
        LLSDValue::Undefined => writer.push('!'),
        LLSDValue::Boolean(v) => writer.push(if *v { 'T' } else { 'F' }),
        LLSDValue::String(v) => {
            writer.push('"');
            writer.push_str(&escape_quotes(v));
            writer.push('"');
        }
        LLSDValue::URI(v) => {
            writer.push('l');
            writer.push('"');
            writer.push_str(&escape_url(v));
            writer.push('"');
        }
        LLSDValue::Integer(v) => {
            writer.push('i');
            writer.push_str(&format!("{}",v));
        }
        LLSDValue::Real(v) => {
            writer.push('r');
            writer.push_str(&format!("{}",v));
        }
        LLSDValue::UUID(v) => {
            writer.push('u');
            writer.push_str(&v.to_string());
        }
        LLSDValue::Binary(v) => {
            writer.push('b');
            writer.push('1');
            writer.push('6');
            writer.push('"');
            writer.push_str(&hex::encode(v));
            writer.push('"');
        }
        LLSDValue::Date(v) => {
            writer.push('d');
            todo!();    // ***NEED DATE PARSER***
        }

        //  Map is {  key : value, key : value ... }
        LLSDValue::Map(v) => {
            //  Curly bracketed list
            writer.push('{');
            //  Output key/value pairs
            let mut first: bool = true;
            for (key, value) in v {
                if !first {
                    writer.push(',');
                    first = false;
                }
                writer.push('\'');
                writer.push_str(key);
                writer.push('\'');
                writer.push(':');
                generate_value(writer, value)?;
            }
            writer.push('}');
        }
        //  Array is [ child, child ... ]
        LLSDValue::Array(v) => {
            //  Square bracketed list
            writer.push('[');
            //  Output array entries
            let mut first: bool = true;
            for value in v {
                if !first {
                    writer.push(',');
                    first = false;
                }
                generate_value(writer, value)?;
            }
            writer.push(']');
        }
    };
    Ok(())
}

/// Escape double quote as \"
fn escape_quotes(s: &str) -> String {
    todo!()
}

/// Escape URL per RFC1738
fn escape_url(s: &str) -> String {
    todo!()
}

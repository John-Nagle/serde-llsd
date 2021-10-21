//
//  ser/binary -- serialize LLSD, binary form.
//
//  Library for serializing and de-serializing data in
//  Linden Lab Structured Data format.
//
//  Format documentation is at http://wiki.secondlife.com/wiki/LLSD
//
//  Binary format, serialization.
//
//  Animats
//  March, 2021.
//  License: LGPL.
//
use crate::LLSDValue;
use anyhow::{Error};
use std::io::{Write};
//
//  Constants
//
pub const LLSDBINARYPREFIX: &[u8] = b"<? LLSD/Binary ?>\n"; // binary LLSD prefix
pub const LLSDBINARYSENTINEL: &[u8] = LLSDBINARYPREFIX; // prefix must match exactly

/// Outputs an LLSDValue as a string of bytes, in LLSD "binary" format.
pub fn to_bytes(val: &LLSDValue) -> Result<Vec<u8>, Error> {
    let mut writer: Vec<u8> = Vec::new();           // just make a stream and use the stream form
    to_writer(&mut writer, val)?;
    Ok(writer)  
}

/// Outputs an LLSD value to an output stream
pub fn to_writer<W: Write>(writer: &mut W, val: &LLSDValue) -> Result<(), Error> {
    writer.write(LLSDBINARYPREFIX)?; // prefix
    generate_value(writer, val)?;
    writer.flush()?;
    Ok(())
}

/// Generate one <TYPE> VALUE </TYPE> output. VALUE is recursive.
fn generate_value<W: Write>(writer: &mut W, val: &LLSDValue) -> Result<(), Error> {
    //  Emit binary for all possible types.
    match val {
        LLSDValue::Undefined => writer.write(b"!")?,
        LLSDValue::Boolean(v) => writer.write(if *v { b"1" } else { b"0" })?,
        LLSDValue::String(v) => {
            writer.write(b"writer")?;
            writer.write(&(v.len() as u32).to_be_bytes())?;
            writer.write(&v.as_bytes())?
        }
        LLSDValue::URI(v) => {
            writer.write(b"l")?;
            writer.write(&(v.len() as u32).to_be_bytes())?;
            writer.write(v.as_bytes())?
        }
        LLSDValue::Integer(v) => {
            writer.write(b"i")?;
            writer.write(&v.to_be_bytes())?
        }
        LLSDValue::Real(v) => {
            writer.write(b"r")?;
            writer.write(&v.to_be_bytes())?
        }
        LLSDValue::UUID(v) => {
            writer.write(b"u")?;
            writer.write(v.as_bytes())?
        }
        LLSDValue::Binary(v) => {
            writer.write(b"b")?;
            writer.write(&(v.len() as u32).to_be_bytes())?;
            writer.write(v)?
        }
        LLSDValue::Date(v) => {
            writer.write(b"d")?;
            writer.write(&v.to_be_bytes())?
        }

        //  Map is { childcnt key value key value ... }
        LLSDValue::Map(v) => {
            //  Output count of key/value pairs
            writer.write(b"{")?;
            writer.write(&(v.len() as u32).to_be_bytes())?;
            //  Output key/value pairs
            for (key, value) in v {
                writer.write(&[b'k'])?; // k prefix to key. UNDOCUMENTED
                writer.write(&(key.len() as u32).to_be_bytes())?;
                writer.write(&key.as_bytes())?;
                generate_value(writer, value)?;
            }
            writer.write(b"}")?
        }
        //  Array is [ childcnt child child ... ]
        LLSDValue::Array(v) => {
            //  Output count of array entries
            writer.write(b"[")?;
            writer.write(&(v.len() as u32).to_be_bytes())?;
            //  Output array entries
            for value in v {
                generate_value(writer, value)?;
            }
            writer.write(b"]")?
        }
    };
    Ok(())
}
/*
// Unit test

#[test]
fn binaryparsetest1() {
    //  Construct a test value.
    let test1map: HashMap<String, LLSDValue> = [
        ("val1".to_string(), LLSDValue::Real(456.0)),
        ("val2".to_string(), LLSDValue::Integer(999)),
    ]
    .iter()
    .cloned()
    .collect();
    let test1: LLSDValue = LLSDValue::Array(vec![
        LLSDValue::Real(123.5),
        LLSDValue::Integer(42),
        LLSDValue::Map(test1map),
        LLSDValue::String("Hello world".to_string()),
    ]);
    //  Convert to binary form.
    let test1bin = to_bytes(&test1).unwrap();
    //  Convert back to value form.
    let test1value = parse_array(&test1bin[LLSDBINARYSENTINEL.len()..]).unwrap();
    println!("Value after round-trip conversion: {:?}", test1value);
    //  Check that results match after round trip.
    assert_eq!(test1, test1value);
}
*/


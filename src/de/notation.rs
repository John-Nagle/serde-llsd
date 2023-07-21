//! #  de/notation -- de-serialize LLSD, "notation" form.
//!
//!  Library for serializing and de-serializing data in
//!  Linden Lab Structured Data format.
//!
//!  Format documentation is at http://wiki.secondlife.com/wiki/LLSD
//!
//!  Notation format.
//!  Similar to JSON, but not compatible
//
//  Animats
//  March, 2021.
//  License: LGPL.
//
use crate::LLSDValue;
use anyhow::{anyhow, Error};
use std::collections::HashMap;
use std::io::{Cursor, Read};
use core::iter::{Peekable};
use core::str::Chars;
use uuid;
//
//  Constants
//
/// Notation LLSD prefix
pub const LLSDNOTATIONPREFIX: &[u8] = b"<? llsd/notation ?>\n"; 
/// Sentinel, must match exactly.
pub const LLSDNOTATIONSENTINEL: &[u8] = LLSDNOTATIONPREFIX; 

///    Parse LLSD string expressed in notation format into an LLSDObject tree. No header.
pub fn from_str(notation_str: &str) -> Result<LLSDValue, Error> {
    let mut cursor = notation_str.chars().peekable();
    parse_value(&mut cursor)
}
/*
///    Parse LLSD reader expressed in binary into an LLSDObject tree. No header.
pub fn from_reader(cursor: &mut dyn Read) -> Result<LLSDValue, Error> {
    todo!();
}
*/

/// Parse one value - real, integer, map, etc. Recursive.
fn parse_value(cursor: &mut Peekable<Chars>) -> Result<LLSDValue, Error> {
    /// Parse "iNNN"
    fn parse_integer(cursor: &mut Peekable<Chars>) -> Result<LLSDValue, Error> {
        let mut s = String::with_capacity(20);  // pre-allocate; can still grow
        //  Accumulate numeric chars.
        while let Some(ch) = cursor.peek() {
            match ch {
                '0'|'1'|'2'|'3'|'4'|'5'|'6'|'7'|'8'|'9'|'+'|'-' => s.push(cursor.next().unwrap()),
                 _ => break
            }
        }
        //  Digits accmulated, use standard conversion
        Ok(LLSDValue::Integer(s.parse::<i32>()?))
    }
    
    /// Parse "rNNN".
    //  Does "notation" allow exponents?
    fn parse_real(cursor: &mut Peekable<Chars>) -> Result<LLSDValue, Error> {
        let mut s = String::with_capacity(20);  // pre-allocate; can still grow
        //  Accumulate numeric chars.
        while let Some(ch) = cursor.peek() {
            match ch {
                '0'|'1'|'2'|'3'|'4'|'5'|'6'|'7'|'8'|'9'|'+'|'-'|'.' => s.push(cursor.next().unwrap()),
                 _ => break
            }
        }
        //  Digits accmulated, use standard conversion
        Ok(LLSDValue::Real(s.parse::<f64>()?))
    }
    /// Parse "{ 'key' : value, 'key' : value ... }
    fn parse_map(cursor: &mut Peekable<Chars>) -> Result<LLSDValue, Error> {
        todo!()
    }
    /// Parse "[ value, value ... ]"
    /// At this point, the '[' has been consumed.
    /// At successful return, the ending ']' has been consumed.
    fn parse_array(cursor: &mut Peekable<Chars>) -> Result<LLSDValue, Error> {
        let mut array_items = Vec::new();
        //  Accumulate array elements.
        loop {
            //  Check for end of items
            consume_whitespace(cursor);
            if let Some(ch) = cursor.peek() {
                match ch {
                    ']' => { let _ = cursor.next(); break } // end of array, may be empty.
                    _ => {}
                }
            }
            array_items.push(parse_value(cursor)?);          // parse next value
            //  Check for comma indicating more items.
            consume_whitespace(cursor);
            if let Some(ch) = cursor.peek() {
                match ch {
                    ',' => { let _ = cursor.next(); }   // continue with next field
                    _ => {}
                }
            }
            
        }
        Ok(LLSDValue::Array(array_items))               // return array
    }
    
    /// Consume whitespace. Next char will be non-whitespace.
    fn consume_whitespace(cursor: &mut Peekable<Chars>) {
        while let Some(ch) = cursor.peek() {
            match ch {
                ' ' | '\n' => { let _ = cursor.next(); },                 // ignore leading white space
                _ => break
            }
        }       
    }

    //
    consume_whitespace(cursor);                         // ignore leading white space
    if let Some(ch) = cursor.next() {
        match ch {
            '!' => { Ok(LLSDValue::Undefined) }         // "Undefined" as a value
            '0' => { Ok(LLSDValue::Boolean(false)) }    // false
            '1' => { Ok(LLSDValue::Boolean(true)) }     // true
            '{' => { parse_map(cursor) }                // map
            '[' => { parse_array(cursor) }              // array
            'i' => { parse_integer(cursor) }            // integer
            'r' => { parse_real(cursor) }               // real
            //  ***MORE*** add cases
            _ => { Err(anyhow!("Unexpected character: {:?}", ch)) } // error
        }
    } else {
        Err(anyhow!("Premature end of string in parse"))  // error
    }
}
/*
    //  These could be generic if generics with numeric parameters were in stable Rust.
    fn read_u8(cursor: &mut dyn Read) -> Result<u8, Error> {
        let mut b: [u8; 1] = [0; 1];
        cursor.read_exact(&mut b)?; // read one byte
        Ok(b[0])
    }
    fn read_u32(cursor: &mut dyn Read) -> Result<u32, Error> {
        let mut b: [u8; 4] = [0; 4];
        cursor.read_exact(&mut b)?; // read one byte
        Ok(u32::from_be_bytes(b))
    }
    fn read_i32(cursor: &mut dyn Read) -> Result<i32, Error> {
        let mut b: [u8; 4] = [0; 4];
        cursor.read_exact(&mut b)?; // read one byte
        Ok(i32::from_be_bytes(b))
    }
    fn read_i64(cursor: &mut dyn Read) -> Result<i64, Error> {
        let mut b: [u8; 8] = [0; 8];
        cursor.read_exact(&mut b)?; // read one byte
        Ok(i64::from_be_bytes(b))
    }
    fn read_f64(cursor: &mut dyn Read) -> Result<f64, Error> {
        let mut b: [u8; 8] = [0; 8];
        cursor.read_exact(&mut b)?; // read one byte
        Ok(f64::from_be_bytes(b))
    }
    fn read_variable(cursor: &mut dyn Read) -> Result<Vec<u8>, Error> {
        let length = read_u32(cursor)?; // read length in bytes
        let mut buf = vec![0u8; length as usize];
        cursor.read_exact(&mut buf)?;
        Ok(buf) // read bytes of string
    }

    let typecode = read_u8(cursor)?;
    match typecode {
        //  Undefined - the empty value
        b'!' => Ok(LLSDValue::Undefined),
        //  Boolean - 1 or 0
        b'0' => Ok(LLSDValue::Boolean(false)),
        b'1' => Ok(LLSDValue::Boolean(true)),
        //  String - length followed by data
        b's' => Ok(LLSDValue::String(
            std::str::from_utf8(&read_variable(cursor)?)?.to_string(),
        )),
        //  URI - length followed by data
        b'l' => Ok(LLSDValue::URI(
            std::str::from_utf8(&read_variable(cursor)?)?.to_string(),
        )),
        //  Integer - 4 bytes
        b'i' => Ok(LLSDValue::Integer(read_i32(cursor)?)),
        //  Real - 4 bytes
        b'r' => Ok(LLSDValue::Real(read_f64(cursor)?)),
        //  UUID - 16 bytes
        b'u' => {
            let mut buf: [u8; 16] = [0u8; 16];
            cursor.read_exact(&mut buf)?; // read bytes of string
            Ok(LLSDValue::UUID(uuid::Uuid::from_bytes(buf)))
        }
        //  Binary - length followed by data
        b'b' => Ok(LLSDValue::Binary(read_variable(cursor)?)),
        //  Date - 64 bits
        b'd' => Ok(LLSDValue::Date(read_i64(cursor)?)),
        //  Map -- keyed collection of items
        b'{' => {
            let mut dict: HashMap<String, LLSDValue> = HashMap::new(); // accumulate hash here
            let count = read_u32(cursor)?; // number of items
            for _ in 0..count {
                let keyprefix = &read_u8(cursor)?; // key should begin with b'k';
                match keyprefix {
                    b'k' => {
                        let key = std::str::from_utf8(&read_variable(cursor)?)?.to_string();
                        let _ = dict.insert(key, parse_value(cursor)?); // recurse and add, allowing dups
                    }
                    _ => {
                        return Err(anyhow!(
                            "Binary LLSD map key had {:?} instead of expected 'k'",
                            keyprefix
                        ))
                    }
                }
            }
            if read_u8(cursor)? != b'}' {
                return Err(anyhow!("Binary LLSD map did not end properly with }}"));
            }
            Ok(LLSDValue::Map(dict))
        }
        //  Array -- array of items
        b'[' => {
            let mut array: Vec<LLSDValue> = Vec::new(); // accumulate hash here
            let count = read_u32(cursor)?; // number of items
            for _ in 0..count {
                array.push(parse_value(cursor)?); // recurse and add, allowing dups
            }
            if read_u8(cursor)? != b']' {
                return Err(anyhow!("Binary LLSD array did not end properly with ] "));
            }
            Ok(LLSDValue::Array(array))
        }

        _ => Err(anyhow!("Binary LLSD, unexpected type code {:?}", typecode)),
    }
}

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
        LLSDValue::Map(test1map),
        LLSDValue::Integer(42),
        LLSDValue::String("Hello world".to_string()),
    ]);
    //  Convert to binary form.
    let test1bin = crate::to_bytes(&test1).unwrap();
    println!("Binary form: {:?}", test1bin);
    //  Convert back to value form.
    let test1value = from_bytes(&test1bin[LLSDBINARYSENTINEL.len()..]).unwrap();
    println!("Value after round-trip conversion: {:?}", test1value);
    //  Check that results match after round trip.
    assert_eq!(test1, test1value);
}
*/
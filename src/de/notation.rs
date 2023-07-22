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
use core::iter::{Peekable};
use core::str::Chars;
use uuid::{Uuid};
use chrono::DateTime;
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
    
    /// Parse Boolean
    fn parse_boolean(cursor: &mut Peekable<Chars>, first_char: char) -> Result<LLSDValue, Error> {
        //  Accumulate next word
        let mut s = String::with_capacity(4);
        s.push(first_char);     // we already had the first character.        
        loop {              
            if let Some(ch) = cursor.peek() {
                if ch.is_alphabetic() {
                    s.push(cursor.next().unwrap());
                    continue
                }
            }
            break;
        }
        //  Check for all the allowed Boolean forms.
        match s.as_str() {
            "f" | "F" | "false" | "FALSE" => Ok(LLSDValue::Boolean(false)),
            "t" | "T" | "true" | "TRUE" => Ok(LLSDValue::Boolean(true)),
            _ => Err(anyhow!("Parsing Boolean, got {}", s)) 
        }
    }
    
    /// Parse string. "ABC" or 'ABC', with '\' as escape.
    /// Does not currently parse the numeric count prefix form.
    fn parse_quoted_string(cursor: &mut Peekable<Chars>, delim: char) -> Result<String, Error> {
        consume_whitespace(cursor);
        let mut s = String::with_capacity(128);           // allocate reasonably large size
        loop {
            let ch_opt = cursor.next();
            if let Some(ch) = ch_opt { 
                if ch == delim { break };
            } else {
                return Err(anyhow!("String began with EOF instead of quote."))
            }
            match ch_opt {
                Some('\\') => { if let Some(ch) = cursor.next() {
                        s.push(ch)
                    } else {
                        return Err(anyhow!("String ended with EOF instead of quote."));
                    }
                }
                Some(_) => s.push(ch_opt.unwrap()), 
                None => { return Err(anyhow!("String ended with EOF instead of quote.")); }
            }
        }
        String::shrink_to_fit(&mut s);                      // release wasted space
        Ok(s)
    }
    
    /// Parse date string
    fn parse_date(cursor: &mut Peekable<Chars>) -> Result<LLSDValue, Error> {
        if let Some(delim) = cursor.next() {
            if delim == '"' || delim == '\'' {
                let s = parse_quoted_string(cursor, delim)?;
                let naive_date =  DateTime::parse_from_rfc3339(&s)?; // parse date per RFC 3339.
                Ok(LLSDValue::Date(naive_date.timestamp())) // seconds since UNIX epoch.
            } else {
                Err(anyhow!("URI did not begin with '\"'"))
            }
        } else {
            Err(anyhow!("URI at end of file."))
        }
    }
    
    /// Parse URI string per rfc 1738
    fn parse_uri(cursor: &mut Peekable<Chars>) -> Result<LLSDValue, Error> {
        if let Some(delim) = cursor.next() {
            if delim == '"' || delim == '\'' {
                let s = parse_quoted_string(cursor, delim)?;
                Ok(LLSDValue::URI(urlencoding::decode(&s)?.to_string()))
            } else {
                Err(anyhow!("URI did not begin with '\"'"))
            }
        } else {
            Err(anyhow!("URI at end of file."))
        }
    }
    
    /// Parse UUID. No quotes
    fn parse_uuid(cursor: &mut Peekable<Chars>) -> Result<LLSDValue, Error> {
        const UUID_LEN: usize = "c69b29b1-8944-58ae-a7c5-2ca7b23e22fb".len();
        let mut s = String::with_capacity(UUID_LEN);
        //  next_chunk, for getting N chars, doesn't work yet.
        for _ in 0..UUID_LEN {
            s.push(cursor.next().ok_or(anyhow!("EOF parsing UUID"))?);
        }
        Ok(LLSDValue::UUID(Uuid::parse_str(&s)?))
    }
    
    /// Parse binary value.
    fn parse_binary(cursor: &mut Peekable<Chars>) -> Result<LLSDValue, Error> {
        todo!()
    }

    
    /// Parse "{ 'key' : value, 'key' : value ... }
    fn parse_map(cursor: &mut Peekable<Chars>) -> Result<LLSDValue, Error> {
        let mut kvmap = HashMap::new();                         // building map
        loop {
            consume_whitespace(cursor);
            let key = if let Some(ch) = cursor.next() {
                match ch {
                    '}' => { let _ = cursor.next(); break } // end of map, may be empty.
                    '\'' | '"' => parse_quoted_string(cursor, ch)?, 
                    _ => { return Err(anyhow!("Map key began with {} instead of quote.", ch)); }
                }
            } else {
                return Err(anyhow!("Map key began with EOF instead of quote."));
            };
            consume_char(cursor, ':')?;
            let value = parse_value(cursor)?;           // value of key:value
            kvmap.insert(key, value);
            //  Check for comma indicating more items.
            consume_whitespace(cursor);
            if let Some(',') = cursor.peek() {
                let _ = cursor.next();    // continue with next field
            }
        }
        Ok(LLSDValue::Map(kvmap))
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
            if let Some(']') = cursor.peek() {
                let _ = cursor.next(); break;    // end of array, may be empty.
            }
            array_items.push(parse_value(cursor)?);          // parse next value
            //  Check for comma indicating more items.
            consume_whitespace(cursor);
            if let Some(',') = cursor.peek() {
                let _ = cursor.next();   // continue with next field
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
    
    /// Consume expected non-whitespace char
    fn consume_char(cursor: &mut Peekable<Chars>, expected_ch: char) -> Result<(), Error> {
        consume_whitespace(cursor);
        if let Some(ch) = cursor.next() {
            if ch != expected_ch {
                Err(anyhow!("Expected '{}', found '{}'.", expected_ch, ch))
            } else {
                Ok(())
            }
        } else {
            Err(anyhow!("Expected '{}', found end of string.", expected_ch))
        }
    }

    // Main function. This is called recursively.
    consume_whitespace(cursor);                         // ignore leading white space
    if let Some(ch) = cursor.next() {
        match ch {
            '!' => { Ok(LLSDValue::Undefined) }         // "Undefined" as a value
            '0' => { Ok(LLSDValue::Boolean(false)) }    // false
            '1' => { Ok(LLSDValue::Boolean(true)) }     // true
            'f' | 'F' => { parse_boolean(cursor, ch) }  // false, all alpha forms
            't' | 'T' => { parse_boolean(cursor, ch) }  // true, all alpha forms
            '{' => { parse_map(cursor) }                // map
            '[' => { parse_array(cursor) }              // array
            'i' => { parse_integer(cursor) }            // integer
            'r' => { parse_real(cursor) }               // real
            'd' => { parse_date(cursor) }               // date
            'u' => { parse_uuid(cursor) }               // UUID
            'l' => { parse_uri(cursor) }                // URI
            'b' => { parse_binary(cursor) }             // binary
            '"' => { Ok(LLSDValue::String(parse_quoted_string(cursor, ch)?)) }  // string, double quoted
            '\'' => { Ok(LLSDValue::String(parse_quoted_string(cursor, ch)?)) }  // string, double quoted
            //  ***MORE*** add cases for UUID, URL, date, and binary.
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
*/


#[test]
fn notationparsetest1() {
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
    //  Convert to notation
    let test1bin = crate::notation_to_string(&test1).unwrap();
    println!("Notation form: {:?}", test1bin);
    //  Convert back to value form.
    let test1value = from_str(&test1bin[LLSDNOTATIONPREFIX.len()..]).unwrap();
    println!("Value after round-trip conversion: {:?}", test1value);
    //  Check that results match after round trip.
    assert_eq!(test1, test1value);
}

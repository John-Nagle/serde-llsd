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
use core::str::{Chars, Bytes};
use uuid::{Uuid};
use chrono::DateTime;
use base64::Engine;

//
//  Constants
//
/// Notation LLSD prefix
pub const LLSDNOTATIONPREFIX: &[u8] = b"<? llsd/notation ?>\n"; 
/// Sentinel, must match exactly.
pub const LLSDNOTATIONSENTINEL: &[u8] = LLSDNOTATIONPREFIX; 

// ==================
/// An LLSD stream. May be either a UTF-8 stream or a byte stream
trait LLSDStream<C, S> {
    /// Get next char/byte
    fn next(&mut self) -> Option<C>;
    /// Peek at next char/byte
    fn peek(&mut self) -> Option<&C>;
}

/// Stream, composed of UTF-8 chars.
struct LLSDStreamChars<'a> {
    /// Stream is composed of peekable UTF-8 chars
    stream: Peekable<Chars<'a>>,
}

impl LLSDStream<char, Peekable<Chars<'_>>> for LLSDStreamChars<'_> {
    /// Get next UTF-8 char.
    fn next(&mut self) -> Option<char> {
        self.stream.next()
    }
    /// Peek at next UTF-8 char.
    fn peek(&mut self) -> Option<&char> {
        self.stream.peek()
    }
}

/// Stream, composed of raw bytes.
struct LLSDStreamBytes<'a> {
    /// Stream is composed of peekable bytes.
    stream: Peekable<Bytes<'a>>,
}

impl LLSDStream<u8, Peekable<Bytes<'_>>> for LLSDStreamBytes<'_> {
    /// Get next UTF-8 byte.
    fn next(&mut self) -> Option<u8> {
        self.stream.next()
    }
    /// Peek at next UTF-8 byte.
    fn peek(&mut self) -> Option<&u8> {
        self.stream.peek()
    }
}




// ==================

type InputType<'a> = Peekable<Chars<'a>>;
type AccumulateType = String;

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
fn parse_value(cursor: &mut InputType) -> Result<LLSDValue, Error> {
    /// Parse "iNNN"
    fn parse_integer(cursor: &mut InputType) -> Result<LLSDValue, Error> {
        let mut s = AccumulateType::with_capacity(20);  // pre-allocate; can still grow
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
    fn parse_real(cursor: &mut InputType) -> Result<LLSDValue, Error> {
        let mut s = AccumulateType::with_capacity(20);  // pre-allocate; can still grow
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
    fn parse_boolean(cursor: &mut InputType, first_char: char) -> Result<LLSDValue, Error> {
        //  Accumulate next word
        let mut s = AccumulateType::with_capacity(4);
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
    /// Unclear what a "raw string" means here. We are utf-8 and SL is - what, utf-16?
    fn parse_quoted_string(cursor: &mut InputType, delim: char) -> Result<String, Error> {
        consume_whitespace(cursor);
        let mut s = AccumulateType::with_capacity(128);           // allocate reasonably large size
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
    
    /// Parse date string per RFC 1339.
    fn parse_date(cursor: &mut InputType) -> Result<LLSDValue, Error> {
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
    fn parse_uri(cursor: &mut InputType) -> Result<LLSDValue, Error> {
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
    fn parse_uuid(cursor: &mut InputType) -> Result<LLSDValue, Error> {
        const UUID_LEN: usize = "c69b29b1-8944-58ae-a7c5-2ca7b23e22fb".len();   // just to get the length of a standard format UUID.
        let s = next_chunk(cursor, UUID_LEN)?;   // read fixed length
        Ok(LLSDValue::UUID(Uuid::parse_str(&s)?))
    }
    
    /// Parse binary value.
    /// Format is b16"value" or b64"value" or b(cnt)"value".
    /// Not sure about that last form. Input to this is UTF-8.
    /// Putting text in this format is just wrong, yet the LL example does it.
    /// This conversion may fail for non-UTF8 input.
    //
    //  The LL parser for this is at
    //  https://github.com/secondlife/viewer/blob/ec4135da63a3f3877222fba4ecb59b15650371fe/indra/llcommon/llsdserialize.cpp#L789
    //  That reads N bytes from the input as a byte stream. But we're working with UTF-8. This is a problem.
    //
    fn parse_binary(cursor: &mut InputType) -> Result<LLSDValue, Error> {
        if let Some(ch) = cursor.peek() {
            match ch {
                '(' => {
                    let cnt = parse_number_in_parentheses(cursor)?;
                    consume_char(cursor, '"')?;
                    let s = next_chunk(cursor, cnt)?;
                    consume_char(cursor, '"')?;  // count must be correct or this will fail.
                    Ok(LLSDValue::String(s))     // not sure about this
                }                 
                '1' => {
                    consume_char(cursor, '1')?;
                    consume_char(cursor, '6')?;          // base 16
                    consume_char(cursor, '"')?;          // begin quote
                    let mut s = parse_quoted_string(cursor,'"')?;
                    s.retain(|c| !c.is_whitespace());
                    Ok(LLSDValue::Binary(hex::decode(s)?))
                }
                '6' => {
                    consume_char(cursor, '6')?;
                    consume_char(cursor, '4')?;
                    consume_char(cursor, '"')?;          // begin quote
                    let mut s = parse_quoted_string(cursor,'"')?;
                    s.retain(|c| !c.is_whitespace());
                    println!("Base 64 decode input: \"{}\"", s);    // ***TEMP***
                    let bytes = base64::engine::general_purpose::STANDARD.decode(s)?;
                    Ok(LLSDValue::Binary(bytes))
                }
                _ => Err(anyhow!("Binary value started with {} instead of (, 1, or 6", ch))   
            } 
        } else {
            Err(anyhow!("Binary value started with EOF"))   
        }
    }
    
    /// Parse sized string.
    /// Format is s(NNN)"string"
    fn parse_sized_string(cursor: &mut InputType) -> Result<LLSDValue, Error> {
        let cnt = parse_number_in_parentheses(cursor)?;
        println!("String size is {}", cnt);
        //  At this point, we are supposed to have a quoted string with no escape chararacters. I think.
        //  This may have problems with non-UTF8 encoding.
        consume_char(cursor, '"')?;
        let s = next_chunk(cursor, cnt)?;
        consume_char(cursor, '"')?;
        Ok(LLSDValue::String(s))
    }
    
    fn parse_number_in_parentheses(cursor: &mut InputType) -> Result<usize, Error> {
        consume_char(cursor, '(')?;
        let val = parse_integer(cursor)?;
        consume_char(cursor, ')')?;   
        if let LLSDValue::Integer(v) = val {
            Ok(v as usize)
        } else {
            panic!("Integer parse did not return an integer.");
        }
    }
    
    /// Read chunk of N characters.
    //  This is a built-in feature of Chars in Nightly, but it hasn't shipped in stable Rust yet.
    fn next_chunk(cursor: &mut InputType, cnt: usize) -> Result<String, Error> {
        let mut s = AccumulateType::with_capacity(cnt);
        //  next_chunk, for getting N chars, doesn't work yet.
        for _ in 0..cnt {
            s.push(cursor.next().ok_or(anyhow!("EOF parsing UUID"))?);
        }
        Ok(s)
    }

    /// Parse "{ 'key' : value, 'key' : value ... }
    fn parse_map(cursor: &mut InputType) -> Result<LLSDValue, Error> {
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
    fn parse_array(cursor: &mut InputType) -> Result<LLSDValue, Error> {
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
    fn consume_whitespace(cursor: &mut InputType) {
        while let Some(ch) = cursor.peek() {
            match ch {
                ' ' | '\n' => { let _ = cursor.next(); },                 // ignore leading white space
                _ => break
            }
        }       
    }
    
    /// Consume expected non-whitespace char
    fn consume_char(cursor: &mut InputType, expected_ch: char) -> Result<(), Error> {
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
            's' => { parse_sized_string(cursor) }       // string with explicit size
            '"' => { Ok(LLSDValue::String(parse_quoted_string(cursor, ch)?)) }  // string, double quoted
            '\'' => { Ok(LLSDValue::String(parse_quoted_string(cursor, ch)?)) }  // string, double quoted
            //  ***MORE*** add cases for UUID, URL, date, and binary.
            _ => { Err(anyhow!("Unexpected character: {:?}", ch)) } // error
        }
    } else {
        Err(anyhow!("Premature end of string in parse"))  // error
    }
}

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

#[test]
fn notationparsetest2() {
    //  Linden Lab documented test data from wiki. Compatibility test use only.
    const TESTNOTATION2: &str = r#"
[
  {'destination':l"http://secondlife.com"}, 
  {'version':i1}, 
  {
    'agent_id':u3c115e51-04f4-523c-9fa6-98aff1034730, 
    'session_id':u2c585cec-038c-40b0-b42e-a25ebab4d132, 
    'circuit_code':i1075, 
    'first_name':'Phoenix', 
    'last_name':'Linden',
    'position':[r70.9247,r254.378,r38.7304], 
    'look_at':[r-0.043753,r-0.999042,r0], 
    'granters':[ua2e76fcd-9360-4f6d-a924-000000000003],
    'attachment_data':
    [
      {
        'attachment_point':i2,
        'item_id':ud6852c11-a74e-309a-0462-50533f1ef9b3,
        'asset_id':uc69b29b1-8944-58ae-a7c5-2ca7b23e22fb
      },
      {
        'attachment_point':i10, 
        'item_id':uff852c22-a74e-309a-0462-50533f1ef900,
        'asset_id':u5868dd20-c25a-47bd-8b4c-dedc99ef9479
      }
    ]
  }
]
"#;
    let parsed2 =  from_str(TESTNOTATION2).unwrap();
    println!("Parse of {}: \n{:#?}", TESTNOTATION2, parsed2);
}

#[test]
fn notationparsetest3() {
    //  Linden Lab documented test data from wiki. Compatibility test use only.
    const TESTNOTATION3: &str = r#"
[
  {
    'creation-date':d"2007-03-15T18:30:18Z", 
    'creator-id':u3c115e51-04f4-523c-9fa6-98aff1034730
  },
  s(10)"0123456789",
  "Where's the beef?",
  'Over here.',  
  b(158)"default
{
    state_entry()
    {
        llSay(0, "Hello, Avatar!");
    }

    touch_start(integer total_number)
    {
        llSay(0, "Touched.");
    }
}",
  b64"AABAAAAAAAAAAAIAAAA//wAAP/8AAADgAAAA5wAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
AABkAAAAZAAAAAAAAAAAAAAAZAAAAAAAAAABAAAAAAAAAAAAAAAAAAAABQAAAAEAAAAQAAAAAAAA
AAUAAAAFAAAAABAAAAAAAAAAPgAAAAQAAAAFAGNbXgAAAABgSGVsbG8sIEF2YXRhciEAZgAAAABc
XgAAAAhwEQjRABeVAAAABQBjW14AAAAAYFRvdWNoZWQuAGYAAAAAXF4AAAAIcBEI0QAXAZUAAEAA
AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA" 
]
"#;
    let parsed3 =  from_str(TESTNOTATION3).unwrap();
    println!("Parse of {}: \n{:#?}", TESTNOTATION3, parsed3);
}

//! #De-serialization. Converts an LLSD stream to tree of LLSDValue structs.
pub mod binary;
pub mod xml;

use anyhow::{anyhow, Error};

/// Parse LLSD, detecting format.
/// Recognizes binary and XML LLSD, with or without sentinel.
pub fn from_bytes(msg: &[u8]) -> Result<crate::LLSDValue, Error> {
    //  Try binary first
    if msg.len() >= binary::LLSDBINARYSENTINEL.len()
        && &msg[0..binary::LLSDBINARYSENTINEL.len()] == binary::LLSDBINARYSENTINEL
    {
        return binary::from_bytes(&msg[binary::LLSDBINARYSENTINEL.len()..]);
    }
    //  Check for binary without header. If array or map marker, parse.
    if msg.len() > 1 {
        match msg[0] {
            // check first char
            b'{' | b'[' => return binary::from_bytes(msg),
            _ => {}
        }
    }
    //  No binary sentinel, try text format.
    let msgstring = std::str::from_utf8(msg)?; // convert to UTF-8 string
    if msgstring.trim_start().starts_with(xml::LLSDXMLSENTINEL) {
        // try XML
        return xml::from_str(msgstring);
    }
    //  "Notation" syntax is not currently supported.
    //  Trim string to N chars for error msg.
    let snippet = msgstring
        .chars()
        .zip(0..60)
        .map(|(c, _)| c)
        .collect::<String>();
    Err(anyhow!("LLSD format not recognized: {:?}", snippet))
}

//! #De-serialization. Converts an LLSD stream to tree of LLSDValue structs.
pub mod binary;
pub mod xml;
pub mod notation;

use anyhow::{anyhow, Error};

/// Parse LLSD, detecting format.
/// Recognizes binary and XML LLSD, with or without sentinel.
pub fn from_bytes(msg: &[u8]) -> Result<crate::LLSDValue, Error> {
    //  Try sentinels first.
    //  Binary sentinel
    if msg.len() >= binary::LLSDBINARYSENTINEL.len()
        && &msg[0..binary::LLSDBINARYSENTINEL.len()] == binary::LLSDBINARYSENTINEL
    {
        return binary::from_bytes(&msg[binary::LLSDBINARYSENTINEL.len()..]);
    }
    //  Try Notation sentinel
    if msg.len() >= notation::LLSDNOTATIONSENTINEL.len()
        && &msg[0..notation::LLSDNOTATIONSENTINEL.len()] == notation::LLSDNOTATIONSENTINEL
    {
        return notation::from_bytes(&msg[notation::LLSDNOTATIONSENTINEL.len()..]);
    }
    
    //  Check for binary without header. If array or map marker, parse.
    if msg.len() > 1 {
        match msg[0] {
            // check first char
            b'{' | b'[' => return binary::from_bytes(msg),
            _ => {}
        }
    }
    
    //  Try XML sentinel.
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

#[test]
fn testpbrmaterialdecode() {
    use base64::Engine;
    // A sample PBR material item, in base64.
    const TESTPBRMATLLLSD: &str =
        "PD8gTExTRC9CaW5hcnkgPz4KewAAAANrAAAABGRhdGFzAAABc3siYXNzZXQiOnsidmVyc2lvbiI6
        IjIuMCJ9LCJpbWFnZXMiOlt7InVyaSI6ImQxZjkxYmI3LWY3ZDYtZDI2Zi1lMGQ3LTU2OGYwZmY3
        NDI3OSJ9LHsidXJpIjoiZDFmOTFiYjctZjdkNi1kMjZmLWUwZDctNTY4ZjBmZjc0Mjc5In0seyJ1
        cmkiOiI4YTQ1Yzk5YS1jZjg0LTc3YzUtOWQ5ZC01Yzk4NzUyMTNmZTkifV0sIm1hdGVyaWFscyI6
        W3sibm9ybWFsVGV4dHVyZSI6eyJpbmRleCI6Mn0sInBick1ldGFsbGljUm91Z2huZXNzIjp7ImJh
        c2VDb2xvclRleHR1cmUiOnsiaW5kZXgiOjB9LCJtZXRhbGxpY1JvdWdobmVzc1RleHR1cmUiOnsi
        aW5kZXgiOjF9fX1dLCJ0ZXh0dXJlcyI6W3sic291cmNlIjowfSx7InNvdXJjZSI6MX0seyJzb3Vy
        Y2UiOjJ9XX0KawAAAAR0eXBlcwAAAAhHTFRGIDIuMGsAAAAHdmVyc2lvbnMAAAADMS4wfQA=";

    let mut clean_base64 = TESTPBRMATLLLSD.to_string();
    clean_base64.retain(|c| !char::is_whitespace(c)); // without whitespace
    println!("Base 64: {}", clean_base64);
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(clean_base64)
        .expect("PBR example failed base64 decode"); // as bytes
    let llsd = from_bytes(&bytes).expect("LLSD decode failed");
    println!("PBR asset: {:?}", llsd);
    //  Display as XML
    println!(
        "As XML: \n{}",
        crate::ser::xml::to_string(&llsd, true).expect("Conversion to XML failed")
    );
}

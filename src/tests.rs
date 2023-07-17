//! # tests.rs -- tests for serialization and deserialization
//! Part of serde-llsd.
///
//  Animats
//  July, 2023.
//  License: LGPL.

#[test]
fn testpbrmaterialdecode() {
    use crate::from_bytes;
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
    let llsd_xml = crate::ser::xml::to_string(&llsd, true).expect("Conversion to XML failed");
    //  Display as XML
    println!(
        "As XML: \n{}",
        llsd_xml
    );
}

#[test]
fn teststructdecode() {
    //  Decode into a structure.
    use serde::{Deserialize, Serialize};
    #[derive(Serialize, Deserialize)]
    struct NamedPoint {
        name: String,
        x: f32,
        y: f32,
    }
    //  Automatic conversion from structure.
    //  ***NOT IMPLEMENTED YET***
/*
    let pt = NamedPoint { name: "Home".as_string(), x: 100.0, y: 200.0 };
    let llsd_xml = crate::to_string(pt).expect("Conversion to XML failed.");
    //  Display as XML
    println!(
        "As XML: \n{}",
        llsd_xml
    );
*/
}

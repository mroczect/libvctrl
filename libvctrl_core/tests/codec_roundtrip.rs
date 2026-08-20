use libvctrl_core::codec::{BinaryDecoder, BinaryEncoder, VERSION};
use libvctrl_core::object::BlobBuilder;
use std::io::Cursor;

mod common;

fn encode_to_vec<F, W>(encode_fn: F) -> Vec<u8>
where
    W: std::io::Write + Send,
    F: FnOnce(&mut W) -> Result<(), libvctrl_core::codec::binary_encoder::VctrlError>,
{
    let mut buf = Cursor::new(Vec::new());
    encode_fn(&mut buf).unwrap();
    buf.into_inner()
}

#[test]
fn test_blob_roundtrip() {
    let original_data = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let blob = BlobBuilder::new()
        .with_data(original_data.clone())
        .build()
        .expect("blob build should succeed");

    let encoded = encode_to_vec(|w| BinaryEncoder.encode_blob(&blob, w));
    assert_eq!(encoded[0], VERSION, "first byte should be version");

    let decoded = BinaryDecoder
        .decode_blob(Cursor::new(encoded))
        .expect("decode should succeed");
    assert_eq!(
        decoded.data(),
        original_data.as_slice(),
        "roundtrip blob data should match original"
    );
}

#[test]
fn test_blob_empty_roundtrip() {
    let blob = BlobBuilder::new()
        .with_data(vec![])
        .build()
        .expect("empty blob build should succeed");

    let encoded = encode_to_vec(|w| BinaryEncoder.encode_blob(&blob, w));
    let decoded = BinaryDecoder
        .decode_blob(Cursor::new(encoded))
        .expect("decode empty blob should succeed");
    assert!(
        decoded.data().is_empty(),
        "roundtrip empty blob should have empty data"
    );
}

#[test]
fn test_blob_large_roundtrip() {
    let original_data = vec![0x42u8; 8192];
    let blob = BlobBuilder::new()
        .with_data(original_data.clone())
        .build()
        .expect("large blob build should succeed");

    let encoded = encode_to_vec(|w| BinaryEncoder.encode_blob(&blob, w));
    let decoded = BinaryDecoder
        .decode_blob(Cursor::new(encoded))
        .expect("decode large blob should succeed");
    assert_eq!(decoded.data(), original_data.as_slice());
}

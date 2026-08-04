use libvctrl::Blob;

#[test]
fn blob_new_and_hash() {
    let data = b"hello world";
    let blob = Blob::new(data.to_vec());
    assert_eq!(blob.as_bytes(), data);
    let hash = blob.hash().expect("hash harus sukses");
    assert_eq!(hash.as_bytes().len(), 64);
}

#[test]
fn blob_into_bytes() {
    let data = vec![1, 2, 3];
    let blob = Blob::new(data.clone());
    let retrieved = blob.into_bytes();
    assert_eq!(retrieved, data);
}

#[test]
fn blob_deterministic_hash() {
    let blob1 = Blob::new(b"same data".to_vec());
    let blob2 = Blob::new(b"same data".to_vec());
    let h1 = blob1.hash().unwrap();
    let h2 = blob2.hash().unwrap();
    assert_eq!(h1, h2);
}

#[test]
fn blob_different_data_different_hash() {
    let blob1 = Blob::new(b"data1".to_vec());
    let blob2 = Blob::new(b"data2".to_vec());
    assert_ne!(blob1.hash().unwrap(), blob2.hash().unwrap());
}

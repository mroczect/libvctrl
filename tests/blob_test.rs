use libvctrl::Blob;

#[test]
fn blob_new_and_access() {
    let data = b"hello world";
    let blob = Blob::new(data.to_vec());
    assert_eq!(blob.as_bytes(), data);
}

#[test]
fn blob_into_bytes() {
    let data = vec![1, 2, 3];
    let blob = Blob::new(data.clone());
    assert_eq!(blob.into_bytes(), data);
}

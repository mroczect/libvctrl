use ed25519_dalek::{Verifier, VerifyingKey};
use libvctrl::crypto::{LibrageSigner, Signer};
use tempfile::tempdir;

#[test]
fn test_generate_and_sign() {
    let signer = LibrageSigner::generate();
    let hash = [42u8; 64];
    let signature = signer.sign(&hash).unwrap();
    let vk: VerifyingKey = signer.verifying_key();
    let sig =
        ed25519_dalek::Signature::try_from(signature.as_slice()).expect("valid signature bytes");
    assert!(vk.verify(&hash, &sig).is_ok());
}

#[test]
fn test_sign_deterministic() {
    let signer = LibrageSigner::generate();
    let hash = [42u8; 64];
    let sig1 = signer.sign(&hash).unwrap();
    let sig2 = signer.sign(&hash).unwrap();
    assert_eq!(sig1, sig2);
}

#[test]
fn test_different_hashes_different_signatures() {
    let signer = LibrageSigner::generate();
    let hash1 = [1u8; 64];
    let hash2 = [2u8; 64];
    let sig1 = signer.sign(&hash1).unwrap();
    let sig2 = signer.sign(&hash2).unwrap();
    assert_ne!(sig1, sig2);
}

#[test]
fn test_verify_wrong_hash_fails() {
    let signer = LibrageSigner::generate();
    let hash = [42u8; 64];
    let signature = signer.sign(&hash).unwrap();
    let vk = signer.verifying_key();
    let sig = ed25519_dalek::Signature::try_from(signature.as_slice()).unwrap();
    let wrong_hash = [99u8; 64];
    assert!(vk.verify(&wrong_hash, &sig).is_err());
}

#[test]
fn test_sign_empty_hash() {
    let signer = LibrageSigner::generate();
    let empty = [0u8; 64];
    let signature = signer.sign(&empty).unwrap();
    let vk = signer.verifying_key();
    let sig = ed25519_dalek::Signature::try_from(signature.as_slice()).unwrap();
    assert!(vk.verify(&empty, &sig).is_ok());
}

#[test]
fn test_from_seed_file() {
    let seed = [0xcd; 32];
    let dir = tempdir().unwrap();
    let seed_path = dir.path().join("seed.bin");
    std::fs::write(&seed_path, seed).unwrap();

    let loaded = LibrageSigner::from_seed_file(&seed_path).unwrap();
    let hash = [0xab; 64];
    let sig = loaded.sign(&hash).unwrap();
    let vk = loaded.verifying_key();
    let sig = ed25519_dalek::Signature::try_from(sig.as_slice()).unwrap();
    assert!(vk.verify(&hash, &sig).is_ok());
}

#[test]
fn test_seed_file_invalid_length() {
    let dir = tempdir().unwrap();
    let seed_path = dir.path().join("short.bin");
    std::fs::write(&seed_path, [0u8; 31]).unwrap();
    let err = LibrageSigner::from_seed_file(&seed_path).unwrap_err();
    match err {
        libvctrl::VctrlError::Other(msg) => assert!(msg.contains("32 bytes")),
        _ => panic!("expected Other error"),
    }
}

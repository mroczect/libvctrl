#![allow(missing_docs)]
#![cfg(feature = "sha384")]

use criterion::{Criterion, criterion_group, criterion_main};
use libvctrl_sha512::sha384;

fn bench_sha384(c: &mut Criterion) {
    let data = [0x42u8; 1024];
    c.bench_function("SHA384/hash_1kb", |b| {
        b.iter(|| sha384::Hash::hash(core::hint::black_box(&data)));
    });
}

fn bench_hmac_sha384(c: &mut Criterion) {
    let key = [0x01u8; 32];
    let data = [0x42u8; 1024];
    c.bench_function("HMAC-SHA384/mac_1kb", |b| {
        b.iter(|| sha384::HMAC::mac(core::hint::black_box(&data), core::hint::black_box(&key)));
    });
}

criterion_group!(benches_sha384, bench_sha384, bench_hmac_sha384);
criterion_main!(benches_sha384);

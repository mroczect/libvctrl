#![allow(missing_docs)]

use zeroize as _;

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use libvctrl_sha512::{HKDF, HMAC, Hash};

fn bench_sha512(c: &mut Criterion) {
    let data = [0x42_u8; 1024];
    let _ = c.bench_function("SHA512/hash_1kb", |b| {
        b.iter(|| Hash::hash(core::hint::black_box(&data)));
    });
}

fn bench_hmac(c: &mut Criterion) {
    let key = [0x01_u8; 32];
    let data = [0x42_u8; 1024];

    let _ = c.bench_function("HMAC-SHA512/mac_1kb", |b| {
        b.iter(|| HMAC::mac(core::hint::black_box(&data), core::hint::black_box(&key)));
    });

    let _ = c.bench_function("HMAC-SHA512/streaming_1kb_chunked", |b| {
        b.iter_batched(
            || (key, data),
            |(k, d)| {
                let mut hmac = HMAC::new(k);
                for chunk in d.chunks(64) {
                    hmac.update(chunk);
                }
                hmac.finalize()
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_hkdf(c: &mut Criterion) {
    let ikm = [0x0b_u8; 22];
    let salt = [0x00_u8; 13];
    let info = [0xf0_u8; 10];

    let _ = c.bench_function("HKDF-SHA512/extract", |b| {
        b.iter(|| HKDF::extract(core::hint::black_box(salt), core::hint::black_box(ikm)));
    });

    let prk = HKDF::extract(salt, ikm);
    let _ = c.bench_function("HKDF-SHA512/expand_64_bytes", |b| {
        b.iter(|| {
            let mut out = [0_u8; 64];
            HKDF::expand(
                &mut out,
                core::hint::black_box(prk),
                core::hint::black_box(info),
            );
            out
        });
    });
}

criterion_group!(benches, bench_sha512, bench_hmac, bench_hkdf);
criterion_main!(benches);

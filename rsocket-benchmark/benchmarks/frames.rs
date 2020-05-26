use bytes::{Bytes, BytesMut};
use criterion::{criterion_group, Criterion};
use rsocket_rust::frame::*;
use rsocket_rust::utils::Writeable;

fn bench_unmarshal_request_response(c: &mut Criterion) {
    c.bench_function("unmarshal request_response", |b| {
        b.iter(|| {
            let f = RequestResponse::builder(1234, 0)
                .set_data(Bytes::from("Hello World"))
                .set_metadata(Bytes::from("Foobar"))
                .build();
            let mut bf = BytesMut::with_capacity(f.len() as usize);
            f.write_to(&mut bf);
        })
    });
}

criterion_group!(benches, bench_unmarshal_request_response);

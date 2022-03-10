use amqp_core::methods::Method;
use amqp_transport::methods::{
    RandomMethod, {self},
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::SeedableRng;

fn random_method_from_seed(seed: u128) -> Method {
    let mut rng = rand::rngs::StdRng::from_seed(
        [seed.to_be_bytes(), seed.to_be_bytes()]
            .concat()
            .try_into()
            .unwrap(),
    );
    Method::random(&mut rng)
}

fn serialize_method(method: Method) -> Vec<u8> {
    let mut writer = Vec::new();
    methods::write::write_method(method, &mut writer).unwrap();
    writer
}

fn parse_method(c: &mut Criterion) {
    let methods = (0..10000)
        .map(random_method_from_seed)
        .map(serialize_method)
        .collect::<Vec<_>>();

    c.bench_function("parse random methods", |b| {
        b.iter(|| {
            for data in &methods {
                let result = methods::parse_method(black_box(data)).unwrap();
                black_box(result);
            }
        })
    });
}

criterion_group!(benches, parse_method);
criterion_main!(benches);

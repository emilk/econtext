use criterion::{criterion_group, criterion_main, Criterion};

use econtext::*;

pub fn criterion_benchmark(c: &mut Criterion) {
	c.bench_function("econtext!", |b| {
		b.iter(|| {
			econtext!("context");
			criterion::black_box(42)
		})
	});
	c.bench_function("econtext_data!", |b| {
		b.iter(|| {
			econtext_data!("context", 42);
			criterion::black_box(42)
		})
	});
	c.bench_function("econtext_data! + string alloc", |b| {
		b.iter(|| {
			econtext_data!("context", "file_name.txt".to_owned());
			criterion::black_box(42)
		})
	});
	c.bench_function("econtext_function!", |b| {
		b.iter(|| {
			econtext_function!();
			criterion::black_box(42)
		})
	});
	c.bench_function("econtext_function_data!", |b| {
		b.iter(|| {
			econtext_function_data!(42);
			criterion::black_box(42)
		})
	});
	c.bench_function("econtext_function_data! + string alloc", |b| {
		b.iter(|| {
			econtext_function_data!("file_name.txt".to_owned());
			criterion::black_box(42)
		})
	});
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use icarus_mcp::storage::{DefaultBuffer, StreamingResponse};
use rand::Rng;

#[cfg(any(feature = "simd", feature = "streaming"))]
use icarus_mcp::storage::SimdProcessor;

fn generate_test_data(size: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..size).map(|_| rng.gen::<u8>()).collect()
}

fn benchmark_data_copy(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_copy");

    for &size in &[1024, 4096, 16384, 65536] {
        let data = generate_test_data(size);
        group.throughput(Throughput::Bytes(size as u64));

        // Standard copy
        group.bench_with_input(BenchmarkId::new("standard_copy", size), &data, |b, data| {
            b.iter(|| {
                let mut response = black_box(StreamingResponse::<DefaultBuffer>::new());
                response.extend_from_slice(black_box(data));
            });
        });

        // SIMD-enhanced copy (if available)
        #[cfg(any(feature = "simd", feature = "streaming"))]
        group.bench_with_input(BenchmarkId::new("simd_copy", size), &data, |b, data| {
            b.iter(|| {
                let mut response = black_box(StreamingResponse::<DefaultBuffer>::new());
                response.extend_from_slice_simd(black_box(data));
            });
        });
    }
    group.finish();
}

#[cfg(any(feature = "simd", feature = "streaming"))]
fn benchmark_simd_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_operations");

    for &size in &[1024, 4096, 16384, 65536] {
        let data = generate_test_data(size);
        group.throughput(Throughput::Bytes(size as u64));

        // Checksum
        group.bench_with_input(BenchmarkId::new("checksum", size), &data, |b, data| {
            b.iter(|| {
                black_box(SimdProcessor::fast_checksum(black_box(data)));
            });
        });

        // Compare (using first half vs second half)
        if size >= 2 {
            let half_size = size / 2;
            let data1 = &data[..half_size];
            let data2 = &data[half_size..];

            group.bench_with_input(
                BenchmarkId::new("compare", half_size),
                &(data1, data2),
                |b, (data1, data2)| {
                    b.iter(|| {
                        black_box(SimdProcessor::fast_compare(
                            black_box(data1),
                            black_box(data2),
                        ));
                    });
                },
            );
        }

        // Pattern search (look for first 4 bytes)
        if size >= 8 {
            let pattern = &data[..4];
            group.bench_with_input(
                BenchmarkId::new("find_pattern", size),
                &(&data, pattern),
                |b, (data, pattern)| {
                    b.iter(|| {
                        black_box(SimdProcessor::fast_find(
                            black_box(data),
                            black_box(pattern),
                        ));
                    });
                },
            );
        }
    }
    group.finish();
}

fn benchmark_json_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_parsing");

    let json_data = r#"{"users":[{"id":1,"name":"Alice","email":"alice@example.com","active":true},{"id":2,"name":"Bob","email":"bob@example.com","active":false}],"total":2,"page":1}"#;
    let json_bytes = json_data.as_bytes();

    group.throughput(Throughput::Bytes(json_bytes.len() as u64));

    // Standard JSON parsing
    group.bench_function("standard_json", |b| {
        b.iter(|| {
            let mut response = black_box(StreamingResponse::<DefaultBuffer>::new());
            response.extend_from_slice(black_box(json_bytes));
            response.try_parse_json().unwrap()
        });
    });

    // SIMD-enhanced JSON parsing (if available)
    #[cfg(any(feature = "simd", feature = "streaming"))]
    group.bench_function("simd_json", |b| {
        b.iter(|| {
            let mut response = black_box(StreamingResponse::<DefaultBuffer>::new());
            response.extend_from_slice(black_box(json_bytes));
            response.try_parse_json_simd().unwrap()
        });
    });

    group.finish();
}

#[cfg(any(feature = "simd", feature = "streaming"))]
criterion_group!(
    benches,
    benchmark_data_copy,
    benchmark_simd_operations,
    benchmark_json_parsing
);

#[cfg(not(any(feature = "simd", feature = "streaming")))]
criterion_group!(benches, benchmark_data_copy, benchmark_json_parsing);

criterion_main!(benches);

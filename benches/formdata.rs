use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use layer8_interceptor_production::fetch::formdata::parse_form_data_to_array;

fn parse_form_data_benchmark(c: &mut Criterion) {
    let form_data = web_sys::FormData::new().unwrap();
    // Add some dummy data to the form_data for testing
    form_data.append_with_str("field1", "value1").unwrap();
    form_data.append_with_str("field2", "value2").unwrap();

    c.bench_function("parse_form_data_to_array", |b| {
        b.iter(|| {
            parse_form_data_to_array(
                black_box(form_data.clone()),
                black_box("boundary".to_string()),
            )
        })
    });
}

criterion_group!(benches, parse_form_data_benchmark);
criterion_main!(benches);

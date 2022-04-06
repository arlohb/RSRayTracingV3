use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn new_image(image: &mut eframe::epaint::ColorImage) {
  *image = eframe::epaint::ColorImage::new([500, 400], eframe::epaint::Color32::BLACK);
}

fn modify_image(image: &mut eframe::epaint::ColorImage) {
  image.size = [500, 400];
  image.pixels = vec![eframe::epaint::Color32::BLACK; 500 * 400];
}

fn criterion_benchmark(c: &mut Criterion) {
  let mut image = eframe::epaint::ColorImage::new([400, 300], eframe::epaint::Color32::BLACK);

  c.bench_function("new image", |b| b.iter(|| new_image(black_box(&mut image))));
  c.bench_function("modify image", |b| b.iter(|| modify_image(black_box(&mut image))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

// CONCLUSION
// there is no difference in performance between the two benchmarks
// I prefer modify image as I have to do this with the image mutex
// So I'll do this to be consistent

use criterion::{black_box, criterion_group, criterion_main, Criterion};

// in the real code these are struct members
const WIDTH: u32 = 400;
const HEIGHT: u32 = 300;

fn modify_vec(image: &mut eframe::epaint::ColorImage) {
  let mut pixels = [0u8; (WIDTH * HEIGHT * 4) as usize];

  (0..(WIDTH * HEIGHT) as usize).step_by(4).for_each(|index| {
    let pixel_index = index / 4;
    let y = (pixel_index as u32) / WIDTH;
    let x = pixel_index as u32 % WIDTH;

    let pixel = (x as f32 / WIDTH as f32, y as f32 / HEIGHT as f32, 1.);

    pixels[index] = (pixel.0 * 255.) as u8;
    pixels[index + 1] = (pixel.1 * 255.) as u8;
    pixels[index + 2] = (pixel.2 * 255.) as u8;
    pixels[index + 3] = 255;
  });

  *image = eframe::epaint::ColorImage::from_rgba_unmultiplied([WIDTH as usize, HEIGHT as usize], &pixels);
}

fn modify_pixels(image: &mut eframe::epaint::ColorImage) {
  image.pixels.iter_mut().enumerate().for_each(|(index, colour)| {
    let y = (index as u32) / WIDTH;
    let x = index as u32 % WIDTH;

    let pixel = (x as f32 / WIDTH as f32, y as f32 / HEIGHT as f32, 1.);

    *colour = eframe::epaint::Color32::from_rgb(
      (pixel.0 * 255.) as u8,
      (pixel.1 * 255.) as u8,
      (pixel.2 * 255.) as u8,
    );
  });
}

fn criterion_benchmark(c: &mut Criterion) {
  let mut image = eframe::epaint::ColorImage::new([WIDTH as usize, HEIGHT as usize], eframe::epaint::Color32::BLACK);

  c.bench_function("new image", |b| b.iter(|| modify_vec(black_box(&mut image))));
  c.bench_function("modify image", |b| b.iter(|| modify_pixels(black_box(&mut image))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

// CONCLUSION
// modify_pixels is much faster than modify_vec
// I was doing this anyway so no changes are needed
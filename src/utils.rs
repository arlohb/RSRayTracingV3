pub fn print_bytes(bytes: &Vec<u8>) {
  let mut i = 0;
  bytes.into_iter().for_each(|byte| {
    if i % 16 == 0 {
      print!("\n");
    }
    print!("{:02x} ", byte);
    i += 1
  });
  println!("\n\n");
}

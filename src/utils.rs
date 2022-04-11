pub fn print_bytes(bytes: &Vec<u8>) -> String {
  let mut s = "".to_string();

  let mut i = 0;

  bytes.into_iter().for_each(|byte| {
    if i % 4 == 0 {
      s.push(' ');
    }
    if i % 16 == 0 {
      s.push('\n');
    }
    s.push_str(format!("{:02x} ", byte).as_str());
    i += 1
  });
  
  s
}
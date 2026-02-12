use heapless::String;
use core::fmt::Write;

#[allow(unused)]
pub fn convert_to_lossy_utf8(input: &mut [u8]) {
  for byte in input {
      if !byte.is_ascii() {
          *byte = b'?';
      }
  }
}

pub fn to_hex_string(data: &[u8]) -> String<1024> {
  let mut hex_string = String::<1024>::new(); // Create a heapless String with a max capacity of 64
  for byte in data {
    // Write the hex representation of each byte
    write!(&mut hex_string, "{:02x}", byte).unwrap();
  }
  hex_string
}
fn pad_string(input: &str) -> String {
  let padding_needed = (4 - (input.len() % 4)) % 4;
  let mut padded = input.to_string();
  padded.extend(std::iter::repeat('\0').take(padding_needed));
  padded
}

/**
 * Returns a replacement byte array that's padded with a given byte (usually best to be 0)
 */
pub fn pad_block(input: &[u8], target_size: usize, pad_byte: u8) -> Vec<u8> {
  if input.len() == target_size {
    // If we are already the expected size, just return it
    return input.into();
  }
  // Clone the existing bytes
  let mut new_vec = Vec::from(input);

  // Pad
  let mut pad = vec![pad_byte; target_size-input.len()];
  new_vec.append(&mut pad);

  new_vec
}

pub fn split_into_blocks(input: &str) -> Vec<String> {
  let block_size = input.len() / 4;
  input
      .chars()
      .collect::<Vec<char>>()
      .chunks(block_size)
      .map(|chunk| chunk.iter().collect::<String>())
      .collect()
}

pub fn calculate_ec_block(blocks: &[&[u8]]) -> Vec<u8> {
  let block_size = blocks[0].len();
  let mut ec_block = vec![0u8; block_size];

  for block in blocks {
      for (i, c) in block.iter().enumerate() {
          ec_block[i] ^= c;
      }
  }

  ec_block
}

pub fn reconstruct_block(blocks: &[&[u8]], ec_block: &[u8]) -> Vec<u8> {
  let block_size = blocks[0].len();
  let mut reconstructed = vec![0u8; block_size];

  // XOR all available blocks and EC block
  for block in blocks {
      for (i, c) in block.iter().enumerate() {
          reconstructed[i] ^= c;
      }
  }

  // XOR with EC block to get the missing block
  for (i, c) in ec_block.iter().enumerate() {
      reconstructed[i] ^= c;
  }

  reconstructed
}

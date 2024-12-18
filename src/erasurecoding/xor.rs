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

pub fn split_into_blocks(input: &[u8], num_blocks: usize) -> Vec<&[u8]> {
  let block_size = input.len() / num_blocks;
  input
      .chunks(block_size)
      .map(|chunk| chunk)
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

pub fn reconstruct_block(base_blocks: &[&[u8]], ec_block: &[u8]) -> Vec<u8> {
  let block_size = base_blocks[0].len();
  let mut reconstructed = vec![0u8; block_size];

  // XOR all available blocks and EC block
  for block in base_blocks {
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

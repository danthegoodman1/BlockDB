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
    let mut pad = vec![pad_byte; target_size - input.len()];
    new_vec.append(&mut pad);

    new_vec
}

fn split_into_blocks(input: &[u8], num_blocks: usize) -> Vec<&[u8]> {
    let block_size = input.len() / num_blocks;
    input.chunks(block_size).collect()
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

pub fn create_blocks(input: &[u8], num_blocks: usize, pad_byte: u8) -> Vec<Vec<u8>> {
    // Calculate the block size needed (round up to next multiple of num_blocks)
    let padded_size = ((input.len() + num_blocks - 1) / num_blocks) * num_blocks;

    // Pad the input to make its length divisible by num_blocks
    let padded = pad_block(input, padded_size, pad_byte);

    // Split into equal blocks
    split_into_blocks(&padded, num_blocks)
        .into_iter()
        .map(Vec::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ec() {
        let input = "Hello, this is a test string!";

        // Create 4 equal blocks with zero padding
        let blocks: Vec<Vec<u8>> = create_blocks(input.as_bytes(), 4, 0);

        // Convert blocks to slice references for calculate_ec_block
        let block_refs: Vec<&[u8]> = blocks.iter().map(|b| b.as_ref()).collect();

        // Calculate erasure coding block (XOR of all blocks)
        let ec_block = calculate_ec_block(&block_refs);

        println!("Original blocks:");
        for (i, block) in blocks.iter().enumerate() {
            println!("Block {}: {}", i + 1, String::from_utf8_lossy(block));
        }
        println!("EC block: {}", String::from_utf8_lossy(&ec_block));

        // Simulate loss of block 4 and reconstruct it
        let reconstruction_refs: Vec<&[u8]> = blocks[0..3].iter().map(|b| b.as_ref()).collect();
        let reconstructed = reconstruct_block(&reconstruction_refs, &ec_block);
        println!(
            "\nReconstructed block 4: {}",
            String::from_utf8_lossy(&reconstructed)
        );

        // Verify reconstruction
        assert_eq!(blocks[3], reconstructed);

        // Combine blocks to get original string
        let mut final_string = String::new();
        for i in 0..3 {
            final_string.push_str(&String::from_utf8_lossy(&blocks[i]));
        }
        final_string.push_str(&String::from_utf8_lossy(&reconstructed));

        // Remove padding
        let final_string = final_string.trim_end_matches('\0');
        println!("\nFinal reconstructed string: {}", final_string);
    }
}

use crate::error::Result;

pub fn run_length_decode(input: &[u8]) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut i = 0;
    while i < input.len() {
        let length_byte = input[i];
        i += 1;
        match length_byte {
            0..=127 => {
                let count = (length_byte as usize) + 1;
                if i + count > input.len() {
                    break;
                }
                output.extend_from_slice(&input[i..i + count]);
                i += count;
            }
            129..=255 => {
                let count = 257 - (length_byte as usize);
                if i >= input.len() {
                    break;
                }
                output.extend(std::iter::repeat(input[i]).take(count));
                i += 1;
            }
            128 => {
                break;
            }
        }
    }

    Ok(output)
}

#[cfg(test)]
mod tests {}

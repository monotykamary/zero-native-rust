use crate::security;

/// Join an array of string values with newline separators, into a buffer.
/// Returns a slice of the written portion.
pub fn join<'a>(values: &[&str], buffer: &'a mut [u8]) -> Result<&'a str, JoinError> {
    let mut offset = 0usize;
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            if offset >= buffer.len() { return Err(JoinError::NoSpaceLeft); }
            buffer[offset] = b'\n';
            offset += 1;
        }
        if offset + value.len() > buffer.len() { return Err(JoinError::NoSpaceLeft); }
        buffer[offset..offset + value.len()].copy_from_slice(value.as_bytes());
        offset += value.len();
    }
    std::str::from_utf8(&buffer[..offset]).map_err(|_| JoinError::InvalidUtf8)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinError {
    NoSpaceLeft,
    InvalidUtf8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_values_with_newlines() {
        let mut buffer = [0u8; 256];
        let result = join(&["zero://app", "zero://inline"], &mut buffer).unwrap();
        assert_eq!("zero://app\nzero://inline", result);
    }

    #[test]
    fn join_single_value() {
        let mut buffer = [0u8; 256];
        let result = join(&["zero://app"], &mut buffer).unwrap();
        assert_eq!("zero://app", result);
    }

    #[test]
    fn join_buffer_too_small() {
        let mut buffer = [0u8; 4];
        assert!(join(&["hello", "world"], &mut buffer).is_err());
    }
}

use sha2::{Digest, Sha256};

pub fn sha256(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn hashes_text() {
        assert_eq!(super::sha256("hello").len(), 64);
    }
}

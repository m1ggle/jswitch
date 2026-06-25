use std::{fs::File, io::Read, path::Path};

use sha2::{Digest, Sha256};

use crate::error::jswitch_error::NetworkError;

pub fn verify_sha256(path: &Path, expected: &str) -> Result<(), NetworkError> {
    let mut file = File::open(path).map_err(|source| NetworkError::ReadFile {
        path: path.to_path_buf(),
        source,
    })?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 16 * 1024];

    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|source| NetworkError::ReadFile {
                path: path.to_path_buf(),
                source,
            })?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    let actual = format_sha256(hasher.finalize().as_slice());
    let expected = expected
        .split_whitespace()
        .next()
        .unwrap_or(expected)
        .to_ascii_lowercase();

    if actual == expected {
        Ok(())
    } else {
        Err(NetworkError::ChecksumMismatch { expected, actual })
    }
}

fn format_sha256(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verifies_matching_sha256() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("archive.txt");
        std::fs::write(&path, "jswitch").unwrap();

        verify_sha256(
            &path,
            "3c9e1458bf9b45180d512e5f41e8500b163b3e1166018f66cb55011828affe4a",
        )
        .unwrap();
    }

    #[test]
    fn rejects_mismatched_sha256() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("archive.txt");
        std::fs::write(&path, "jswitch").unwrap();

        let error = verify_sha256(&path, "0000").unwrap_err();

        assert!(matches!(error, NetworkError::ChecksumMismatch { .. }));
    }
}

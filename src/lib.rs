use data_encoding::BASE32;
use ed25519_dalek::{SecretKey, Signature, Signer, SigningKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use serde_cbor;
use std::fs;
use std::io::Result;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Represents the contents of a file
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct File {
    /// Suggested path for this file
    pub path: PathBuf,
    /// The raw file bytes
    pub content: Vec<u8>,
}

impl File {
    pub fn new(path: PathBuf, content: Vec<u8>) -> File {
        File { path, content }
    }

    pub fn read(dir: &Path, path: &Path) -> Result<File> {
        let file_path = dir.join(path);
        let content = std::fs::read(file_path)?;
        return Ok(File::new(path.to_path_buf(), content));
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct Archive {
    /// A Unix timestamp representing the archive creation time
    pub created_at: u64,
    pub files: Vec<File>,
}

impl Archive {
    pub fn new(files: Vec<File>) -> Archive {
        Archive {
            created_at: now_epoch_secs(),
            files,
        }
    }

    pub fn from_paths(dir: &Path, paths: &[PathBuf]) -> Result<Archive> {
        let mut files = Vec::new();

        for path in paths {
            let file = File::read(dir, path)?;
            files.push(file);
        }

        Ok(Archive {
            created_at: now_epoch_secs(),
            files,
        })
    }

    /// Create an archive from the file contents of a directory
    pub fn from_dir(dir: &Path) -> Result<Archive> {
        let mut paths: Vec<PathBuf> = Vec::new();
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_file() {
                paths.push(
                    path.strip_prefix(dir)
                        .map_err(|e| std::io::Error::other(e))?
                        .to_path_buf(),
                );
            }
        }
        return Archive::from_paths(dir, &paths);
    }

    /// Write CBOR archive file
    pub fn write_archive(&self, file: &Path) -> Result<()> {
        let cbor_file = fs::File::create(file)?;
        serde_cbor::to_writer(cbor_file, self).map_err(|e| std::io::Error::other(e))
    }

    /// Serialize the archive as CBOR and return the bytes
    pub fn to_cbor_bytes(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        serde_cbor::to_writer(&mut buffer, self).map_err(|e| std::io::Error::other(e))?;
        Ok(buffer)
    }

    /// Write the contents of the archive to individual files in a directory
    pub fn write_archive_contents(&self, dir: &Path) -> Result<()> {
        fs::create_dir_all(dir).expect("Directory should not exist");
        for file in &self.files {
            let mut file_path = dir.to_path_buf();
            file_path.push(&file.path);
            write_file_deep(&file_path, &file.content)?;
        }
        Ok(())
    }

    /// Read archive from CBOR
    pub fn read_archive(file: &Path) -> Result<Archive> {
        let cbor_file = fs::File::open(file)?;
        let archive: Archive =
            serde_cbor::from_reader(cbor_file).map_err(|e| std::io::Error::other(e))?;
        Ok(archive)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Envelope {
    pub content: Archive,
    /// Signature
    pub sig: Signature,
}

impl Envelope {
    pub fn new(archive: Archive, private_key: &SecretKey) -> Result<Envelope> {
        // Generate a keypair
        let keypair = SigningKey::from_bytes(private_key);

        // Convert archive to CBOR bytes for signing
        let archive_bytes = archive.to_cbor_bytes()?;

        // Sign the archive bytes
        let signature: Signature = keypair.sign(&archive_bytes);

        Ok(Envelope {
            content: archive,
            sig: signature,
        })
    }

    /// Write CBOR archive file
    pub fn write_cbor<W: std::io::Write>(&self, writer: &mut W) -> Result<()> {
        serde_cbor::to_writer(writer, self).map_err(|e| std::io::Error::other(e))
    }
}

pub fn generate_private_key() -> SigningKey {
    let mut csprng = OsRng;
    SigningKey::generate(&mut csprng)
}

pub fn format_key_base32(key: SigningKey) -> String {
    let key_bytes = key.to_bytes().to_vec();
    BASE32.encode(&key_bytes)
}

pub fn now_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Expected now to be greater than epoch")
        .as_secs()
}

/// Write file to path, creating intermediate directories if needed
pub fn write_file_deep(path: &Path, content: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(path, content)
}

#[cfg(test)]
mod tests {
    use super::*;
}

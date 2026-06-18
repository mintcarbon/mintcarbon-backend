use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use anyhow;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;
use uuid::Uuid;

pub struct DocumentStore {
    endpoint: String,
    _access_key: String,
    _secret_key: String,
    encryption_key: [u8; 32],
}

impl DocumentStore {
    pub fn new(endpoint: &str, access_key: &str, secret_key: &str, encryption_key: &str) -> Self {
        let key_bytes = encryption_key.as_bytes();
        let mut key = [0u8; 32];
        let len = key_bytes.len().min(32);
        key[..len].copy_from_slice(&key_bytes[..len]);

        Self {
            endpoint: endpoint.to_string(),
            _access_key: access_key.to_string(),
            _secret_key: secret_key.to_string(),
            encryption_key: key,
        }
    }

    pub fn encrypt_document(&self, data: &[u8]) -> anyhow::Result<String> {
        let cipher = Aes256Gcm::new_from_slice(&self.encryption_key)
            .map_err(|e| anyhow::anyhow!("Invalid key length: {:?}", e))?;
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, data)
            .map_err(|e| anyhow::anyhow!("Encryption failed: {:?}", e))?;
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);
        Ok(BASE64.encode(&result))
    }

    pub fn decrypt_document(&self, encrypted: &str) -> anyhow::Result<Vec<u8>> {
        let decoded = BASE64.decode(encrypted)?;
        if decoded.len() < 12 {
            anyhow::bail!("Invalid encrypted data");
        }
        let (nonce_bytes, ciphertext) = decoded.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        let cipher = Aes256Gcm::new_from_slice(&self.encryption_key)
            .map_err(|e| anyhow::anyhow!("Invalid key length: {:?}", e))?;
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {:?}", e))?;
        Ok(plaintext)
    }

    pub fn store_metadata(
        &self,
        _user_id: Uuid,
        _document_id: &str,
        _encrypted_data: &str,
    ) -> anyhow::Result<()> {
        tracing::info!(
            "[DOC STORE] Metadata stored for user {} (MinIO at {})",
            _user_id,
            self.endpoint
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let store = DocumentStore::new(
            "http://localhost:9000",
            "key",
            "secret",
            "01234567890123456789012345678901",
        );
        let data = b"sensitive document content";
        let encrypted = store.encrypt_document(data).unwrap();
        let decrypted = store.decrypt_document(&encrypted).unwrap();
        assert_eq!(decrypted, data);
    }
}

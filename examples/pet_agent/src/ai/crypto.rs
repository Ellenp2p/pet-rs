//! API Key 加密工具

const ENCRYPTION_KEY: &[u8] = b"pet-agent-encryption-key-2024-very-secret";

/// 加密 API Key
pub fn encrypt_api_key(api_key: &str) -> String {
    if api_key.is_empty() {
        return String::new();
    }
    let encrypted = xor_encrypt(api_key.as_bytes(), ENCRYPTION_KEY);
    base64_encode(&encrypted)
}

/// 解密 API Key
pub fn decrypt_api_key(encrypted: &str) -> String {
    if encrypted.is_empty() {
        return String::new();
    }
    match base64_decode(encrypted) {
        Ok(bytes) => {
            let decrypted = xor_decrypt(&bytes, ENCRYPTION_KEY);
            String::from_utf8_lossy(&decrypted).to_string()
        }
        Err(_) => String::new(),
    }
}

fn xor_encrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, &b)| b ^ key[i % key.len()])
        .collect()
}

fn xor_decrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    xor_encrypt(data, key)
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };

        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

fn base64_decode(s: &str) -> Result<Vec<u8>, ()> {
    let s = s.replace('\n', "").replace('\r', "").replace(' ', "");
    let mut result = Vec::new();
    let mut i = 0;

    while i < s.len() {
        let c0 = decode_char(s.as_bytes()[i])?;
        let c1 = if i + 1 < s.len() {
            decode_char(s.as_bytes()[i + 1])?
        } else {
            0
        };
        let c2 = if i + 2 < s.len() && s.as_bytes()[i + 2] != b'=' {
            decode_char(s.as_bytes()[i + 2])?
        } else {
            0
        };
        let c3 = if i + 3 < s.len() && s.as_bytes()[i + 3] != b'=' {
            decode_char(s.as_bytes()[i + 3])?
        } else {
            0
        };

        let triple = (c0 as u32) << 18 | (c1 as u32) << 12 | (c2 as u32) << 6 | c3 as u32;

        result.push((triple >> 16) as u8);
        if i + 2 < s.len() && s.as_bytes()[i + 2] != b'=' {
            result.push((triple >> 8) as u8);
        }
        if i + 3 < s.len() && s.as_bytes()[i + 3] != b'=' {
            result.push(triple as u8);
        }

        i += 4;
    }

    Ok(result)
}

fn decode_char(c: u8) -> Result<u8, ()> {
    match c {
        b'A'..=b'Z' => Ok(c - b'A'),
        b'a'..=b'z' => Ok(c - b'a' + 26),
        b'0'..=b'9' => Ok(c - b'0' + 52),
        b'+' => Ok(62),
        b'/' => Ok(63),
        _ => Err(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let api_key = "sk-test-api-key-12345";
        let encrypted = encrypt_api_key(api_key);
        let decrypted = decrypt_api_key(&encrypted);
        assert_eq!(api_key, decrypted);
    }

    #[test]
    fn test_empty_key() {
        assert_eq!(encrypt_api_key(""), "");
        assert_eq!(decrypt_api_key(""), "");
    }
}

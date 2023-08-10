use web3::signing::{keccak256, recover};

pub fn hash_message(message: &str) -> [u8; 32] {
    keccak256(
        format!(
            "{}{}{}",
            "\x19Ethereum Signed Message:\n",
            message.len(),
            message
        )
        .as_bytes(),
    )
}

pub fn verify_signature(pub_key: &str, message: &str, signature: &str) -> bool {
    let message_hash = hash_message(message);

    let signature = match hex::decode(signature) {
        Ok(s) => s,
        Err(_) => {
            return false;
        }
    };

    let calculated_pubkey = match recover(&message_hash, &signature[..64], 0) {
        Ok(s) => s,
        Err(_) => {
            return false;
        }
    };

    let calculated_pubkey = format!("{:02X?}", calculated_pubkey);

    pub_key.eq(&calculated_pubkey)
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn verify_signature_test() {
        assert!(verify_signature(&"0x5C0369359155C836F5D02f1D77fc11F637DBbF2b".to_lowercase(), "Example `personal_sign` message", "74f4f02824ee4d25feb72149027b21821b4b27d23ada2d693f87a3e89f6de4d80dc44f940afba47db9eaf5c4a5677d449ab2e669dac816892b5044b5540474ce1b"))
    }
}

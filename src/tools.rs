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

    let recovery_id = signature[64] as i32 - 27;

    let calculated_pubkey = match recover(&message_hash, &signature[..64], recovery_id) {
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
        assert!(verify_signature(&"0x5C0369359155C836F5D02f1D77fc11F637DBbF2b".to_lowercase(), "Sewer", "d734986394109a62815fe958484fcc9f55dc9a5fed1c43bbfec6fdebaf4cb41d3d344021f4e88c6a35bf0ef381a6e6e6e279f52f398365f25b10eb8bb7fda1921c"))
    }
}

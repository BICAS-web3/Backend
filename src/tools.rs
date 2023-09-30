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
        assert!(verify_signature(&"0x67adcf8c25c88af0df3cab522c9dd5b11d017aca".to_lowercase(), "SewerTT", "c4dfdf84509168530464833260da05f45bc680c188c5c1eff59010b0c6c6c6d00c74e442cfa4cd3e67d70a89fdaba67dcc8eec9ebc8716504cc02b6bd89bb8641c"))
    }
}

use crate::{
    secp256k1::SecretKey, Address, ConversionError, Error, Hash, Network, Payload, PubkeyHash, Script, ScriptHash,
    WPubkeyHash, WScriptHash,
};
use sp_core::H160;
use std::str::FromStr;

pub trait PartialAddress: Sized + Eq + PartialOrd {
    /// Decode the `PartialAddress` from the `Payload` type.
    ///
    /// # Arguments
    /// * `payload` - Bitcoin payload (P2PKH, P2SH, P2WPKH)
    fn from_payload(payload: Payload) -> Result<Self, ConversionError>;

    /// Decode the `PartialAddress` from a string.
    ///
    /// # Arguments
    /// * `btc_address` - encoded Bitcoin address
    fn decode_str(btc_address: &str) -> Result<Self, ConversionError>;

    /// Encode the `PartialAddress` as a string.
    ///
    /// # Arguments
    /// * `network` - network to prefix
    fn encode_str(&self, network: Network) -> Result<String, ConversionError> {
        let address = self.to_address(network)?;
        Ok(address.to_string())
    }

    /// Encode the `PartialAddress` as an address that the bitcoin rpc can use.
    ///
    /// # Arguments
    /// * `network` - network to prefix
    fn to_address(&self, network: Network) -> Result<Address, ConversionError>;
}

#[cfg(feature = "interbtc")]
impl PartialAddress for interbtc_bitcoin::Address {
    fn from_payload(payload: Payload) -> Result<Self, ConversionError> {
        match payload {
            Payload::PubkeyHash(hash) => Ok(Self::P2PKH(H160::from(hash.as_hash().into_inner()))),
            Payload::ScriptHash(hash) => Ok(Self::P2SH(H160::from(hash.as_hash().into_inner()))),
            Payload::WitnessProgram { version: _, program } => {
                if program.len() == 20 {
                    Ok(Self::P2WPKHv0(H160::from_slice(program.as_slice())))
                } else {
                    Err(ConversionError::InvalidPayload)
                }
            }
        }
    }

    fn decode_str(btc_address: &str) -> Result<Self, ConversionError> {
        let addr = Address::from_str(btc_address)?;
        Self::from_payload(addr.payload)
    }

    fn to_address(&self, network: Network) -> Result<Address, ConversionError> {
        let script = match self {
            Self::P2PKH(hash) => Script::new_p2pkh(&PubkeyHash::from_slice(hash.as_bytes())?),
            Self::P2SH(hash) => Script::new_p2sh(&ScriptHash::from_slice(hash.as_bytes())?),
            Self::P2WPKHv0(hash) => Script::new_v0_wpkh(&WPubkeyHash::from_slice(hash.as_bytes())?),
            Self::P2WSHv0(hash) => Script::new_v0_wsh(&WScriptHash::from_slice(hash.as_bytes())?),
        };

        let payload = Payload::from_script(&script).ok_or(ConversionError::InvalidPayload)?;
        Ok(Address { payload, network })
    }
}

impl PartialAddress for Payload {
    fn from_payload(payload: Payload) -> Result<Self, ConversionError> {
        Ok(payload)
    }

    fn decode_str(btc_address: &str) -> Result<Self, ConversionError> {
        let address = Address::from_str(btc_address)?;
        Ok(address.payload)
    }

    fn to_address(&self, network: Network) -> Result<Address, ConversionError> {
        Ok(Address {
            network,
            payload: self.clone(),
        })
    }
}

pub fn calculate_deposit_secret_key(vault_key: SecretKey, issue_key: SecretKey) -> Result<SecretKey, Error> {
    let mut deposit_key = vault_key;
    deposit_key.mul_assign(&issue_key[..])?;
    Ok(deposit_key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::secp256k1;
    use rand::{thread_rng, Rng};
    use secp256k1::{constants::SECRET_KEY_SIZE, PublicKey, Secp256k1, SecretKey};
    use sp_core::H256;

    #[test]
    fn test_encode_and_decode_payload() {
        let addr = "bcrt1q6v2c7q7uv8vu6xle2k9ryfj3y3fuuy4rqnl50f";
        assert_eq!(
            addr,
            Payload::decode_str(addr).unwrap().encode_str(Network::Regtest).unwrap()
        );
    }

    #[test]
    fn test_calculate_deposit_secret_key() {
        let secp = Secp256k1::new();

        // c
        let secure_id = H256::random();
        let secret_key = SecretKey::from_slice(secure_id.as_bytes()).unwrap();

        // v
        let raw_secret_key: [u8; SECRET_KEY_SIZE] = thread_rng().gen();
        let vault_secret_key = SecretKey::from_slice(&raw_secret_key).unwrap();
        // V
        let vault_public_key = PublicKey::from_secret_key(&secp, &vault_secret_key);

        // D = V * c
        let mut deposit_public_key = vault_public_key;
        deposit_public_key.mul_assign(&secp, &secret_key[..]).unwrap();

        // d = v * c
        let deposit_secret_key = calculate_deposit_secret_key(vault_secret_key, secret_key).unwrap();

        assert_eq!(
            deposit_public_key,
            PublicKey::from_secret_key(&secp, &deposit_secret_key)
        );
    }
}

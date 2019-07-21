use cc::{CryptoError, Hash, PrivateKey, PublicKey, Signature};

use lazy_static::lazy_static;
use rand::{CryptoRng, Rng};
use secp256k1::{All, Message, Secp256k1, ThirtyTwoByteHash};

use std::convert::TryFrom;

lazy_static! {
    static ref ENGINE: Secp256k1<All> = Secp256k1::new();
}

pub struct Secp256k1PrivateKey(secp256k1::SecretKey);

pub struct Secp256k1PublicKey(secp256k1::PublicKey);

pub struct Secp256k1Signature(secp256k1::Signature);

#[derive(Debug, PartialEq)]
pub struct Secp256k1Error(secp256k1::Error);

pub struct HashedMessage<'a>(&'a Hash);

pub fn generate_keypair<R: CryptoRng + Rng + ?Sized>(
    rng: &mut R,
) -> (Secp256k1PrivateKey, Secp256k1PublicKey) {
    let (secret_key, public_key) = ENGINE.generate_keypair(rng);

    (
        Secp256k1PrivateKey(secret_key),
        Secp256k1PublicKey(public_key),
    )
}

//
// PrivateKey Impl
//

impl TryFrom<&[u8]> for Secp256k1PrivateKey {
    type Error = CryptoError;

    fn try_from(bytes: &[u8]) -> Result<Secp256k1PrivateKey, Self::Error> {
        let secret_key = secp256k1::SecretKey::from_slice(bytes).map_err(Secp256k1Error)?;

        Ok(Secp256k1PrivateKey(secret_key))
    }
}

impl PrivateKey<32> for Secp256k1PrivateKey {
    type PublicKey = Secp256k1PublicKey;
    type Signature = Secp256k1Signature;

    fn sign_message(&self, msg: &Hash) -> Self::Signature {
        let msg = Message::from(HashedMessage(msg));
        let sig = ENGINE.sign(&msg, &self.0);

        Secp256k1Signature(sig)
    }

    fn pub_key(&self) -> Self::PublicKey {
        let pub_key = secp256k1::PublicKey::from_secret_key(&ENGINE, &self.0);

        Secp256k1PublicKey(pub_key)
    }

    fn to_bytes(&self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&self.0[..]);

        bytes
    }
}

//
// PublicKey Impl
//

impl TryFrom<&[u8]> for Secp256k1PublicKey {
    type Error = CryptoError;

    fn try_from(bytes: &[u8]) -> Result<Secp256k1PublicKey, Self::Error> {
        let pub_key = secp256k1::PublicKey::from_slice(bytes).map_err(Secp256k1Error)?;

        Ok(Secp256k1PublicKey(pub_key))
    }
}

impl PublicKey<33> for Secp256k1PublicKey {
    type Signature = Secp256k1Signature;

    fn verify_signature(&self, msg: &Hash, sig: &Self::Signature) -> Result<(), CryptoError> {
        let msg = Message::from(HashedMessage(msg));

        ENGINE
            .verify(&msg, &sig.0, &self.0)
            .map_err(Secp256k1Error)?;

        Ok(())
    }

    fn to_bytes(&self) -> [u8; 33] {
        self.0.serialize()
    }
}

//
// Signature Impl
//

impl TryFrom<&[u8]> for Secp256k1Signature {
    type Error = CryptoError;

    fn try_from(bytes: &[u8]) -> Result<Secp256k1Signature, Self::Error> {
        let sig = secp256k1::Signature::from_compact(bytes).map_err(Secp256k1Error)?;

        Ok(Secp256k1Signature(sig))
    }
}

impl Signature<64> for Secp256k1Signature {
    type PublicKey = Secp256k1PublicKey;

    fn verify(&self, msg: &Hash, pub_key: &Self::PublicKey) -> Result<(), CryptoError> {
        let msg = Message::from(HashedMessage(msg));

        ENGINE
            .verify(&msg, &self.0, &pub_key.0)
            .map_err(Secp256k1Error)?;

        Ok(())
    }

    fn to_bytes(&self) -> [u8; 64] {
        self.0.serialize_compact()
    }
}

//
// Error Impl
//

impl From<Secp256k1Error> for CryptoError {
    fn from(err: Secp256k1Error) -> Self {
        use secp256k1::Error;

        match err.0 {
            Error::IncorrectSignature => CryptoError::InvalidSignature,
            Error::InvalidMessage => CryptoError::InvalidLength,
            Error::InvalidPublicKey => CryptoError::InvalidPublicKey,
            Error::InvalidSignature => CryptoError::InvalidSignature,
            Error::InvalidSecretKey => CryptoError::InvalidPrivateKey,
            Error::InvalidRecoveryId => CryptoError::InvalidSignature,
            Error::InvalidTweak => CryptoError::Other("secp256k1: bad tweak"),
            Error::NotEnoughMemory => CryptoError::Other("secp256k1: not enough memory"),
        }
    }
}

//
// HashedMessage Impl
//

impl<'a> HashedMessage<'a> {
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }
}

impl<'a> ThirtyTwoByteHash for HashedMessage<'a> {
    fn into_32(self) -> [u8; 32] {
        self.to_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::generate_keypair;

    use cc::{Hash, PrivateKey, Signature};

    use rand::rngs::OsRng;
    use sha2::{Digest, Sha256};

    use std::convert::TryFrom;

    #[test]
    fn should_generate_workable_keypair_from_crypto_rng() {
        let mut rng = OsRng::new().expect("OsRng");
        let (priv_key, pub_key) = generate_keypair(&mut rng);

        let msg = {
            let mut hasher = Sha256::new();
            hasher.input(b"you can(not) redo");
            Hash::try_from(&hasher.result()[..32]).expect("msg")
        };

        let sig = priv_key.sign_message(&msg);
        assert!(sig.verify(&msg, &pub_key).is_ok());
    }
}

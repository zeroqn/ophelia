#![feature(const_generics)]

pub mod hash;
pub use hash::HashValue;

use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub enum CryptoError {
    InvalidLength,
    InvalidSignature,
    InvalidPublicKey,
    InvalidPrivateKey,
    Other(&'static str),
}

pub trait PrivateKey<const LEN: usize>: for<'a> TryFrom<&'a [u8], Error = CryptoError> {
    type PublicKey;
    type Signature;

    fn sign_message(&self, msg: &HashValue) -> Self::Signature;

    fn pub_key(&self) -> Self::PublicKey;

    fn to_bytes(&self) -> [u8; LEN];
}

pub trait PublicKey<const LEN: usize>: for<'a> TryFrom<&'a [u8], Error = CryptoError> {
    type Signature;

    fn verify_signature(&self, msg: &HashValue, sig: &Self::Signature) -> Result<(), CryptoError>;

    fn to_bytes(&self) -> [u8; LEN];
}

pub trait Signature<const LEN: usize>: for<'a> TryFrom<&'a [u8], Error = CryptoError> {
    type PublicKey;

    fn verify(&self, msg: &HashValue, pub_key: &Self::PublicKey) -> Result<(), CryptoError>;

    fn to_bytes(&self) -> [u8; LEN];
}

pub trait Crypto<const SK: usize, const PK: usize, const SIG: usize> {
    type PrivateKey: PrivateKey<{ SK }, PublicKey = Self::PublicKey, Signature = Self::Signature>;
    type PublicKey: PublicKey<{ PK }, Signature = Self::Signature>;
    type Signature: Signature<{ SIG }, PublicKey = Self::PublicKey>;

    fn pub_key(priv_key: &[u8]) -> Result<Self::PublicKey, CryptoError> {
        let priv_key = Self::PrivateKey::try_from(priv_key)?;

        Ok(priv_key.pub_key())
    }

    fn sign_message(msg: &[u8], priv_key: &[u8]) -> Result<Self::Signature, CryptoError> {
        let priv_key = Self::PrivateKey::try_from(priv_key)?;
        let msg = HashValue::try_from(msg)?;

        Ok(priv_key.sign_message(&msg))
    }

    fn verify_signature(msg: &[u8], sig: &[u8], pub_key: &[u8]) -> Result<(), CryptoError> {
        let msg = HashValue::try_from(msg)?;
        let sig = Self::Signature::try_from(sig)?;
        let pub_key = Self::PublicKey::try_from(pub_key)?;

        sig.verify(&msg, &pub_key)?;
        Ok(())
    }
}

#[cfg(feature = "proptest")]
pub use cc_quickcheck_types::Octet32;

#[cfg(feature = "proptest")]
#[macro_export]
macro_rules! impl_quickcheck_arbitrary {
    ($priv_key:ident) => {
        impl Clone for $priv_key {
            fn clone(&self) -> Self {
                Self::try_from(self.to_bytes().as_ref()).unwrap()
            }
        }

        impl quickcheck::Arbitrary for $priv_key {
            fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> $priv_key {
                let octet32 = cc::Octet32::arbitrary(g);

                $priv_key::try_from(octet32.as_ref()).unwrap()
            }
        }
    };
}

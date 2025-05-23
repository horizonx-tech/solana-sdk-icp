#![cfg(feature = "full")]

use {
    crate::{
        pubkey::Pubkey,
        signature::{Signature, Signer, SignerError},
    },
    std::sync::Arc,
};

/// Convenience trait for working with mixed collections of `Signer`s
pub trait Signers {
    fn pubkeys(&self) -> Vec<Pubkey>;
    fn try_pubkeys(&self) -> Result<Vec<Pubkey>, SignerError>;
    fn sign_message(&self, message: &[u8]) -> impl std::future::Future<Output = Vec<Signature>> + Send;
    fn try_sign_message(&self, message: &[u8]) -> impl std::future::Future<Output = Result<Vec<Signature>, SignerError>> + Send;
    fn is_interactive(&self) -> bool;
}

macro_rules! default_keypairs_impl {
    () => {
        fn pubkeys(&self) -> Vec<Pubkey> {
            self.iter().map(|keypair| keypair.pubkey()).collect()
        }

        fn try_pubkeys(&self) -> Result<Vec<Pubkey>, SignerError> {
            let mut pubkeys = Vec::new();
            for keypair in self.iter() {
                pubkeys.push(keypair.try_pubkey()?);
            }
            Ok(pubkeys)
        }

        async fn sign_message(&self, message: &[u8]) -> Vec<Signature> {
            let mut signatures = Vec::new();
            for keypair in self.iter() {
                signatures.push(keypair.sign_message(message).await);
            }
            signatures
        }

        async fn try_sign_message(&self, message: &[u8]) -> Result<Vec<Signature>, SignerError> {
            let mut signatures = Vec::new();
            for keypair in self.iter() {
                signatures.push(keypair.try_sign_message(message).await?);
            }
            Ok(signatures)
        }

        fn is_interactive(&self) -> bool {
            self.iter().any(|s| s.is_interactive())
        }
    };
}

impl<T: Signer> Signers for [&T] {
    default_keypairs_impl!();
}

impl Signers for [Box<dyn Signer>] {
    default_keypairs_impl!();
}

impl Signers for Vec<Box<dyn Signer>> {
    default_keypairs_impl!();
}

impl Signers for [Arc<dyn Signer>] {
    default_keypairs_impl!();
}

impl Signers for [Arc<dyn Signer>; 0] {
    default_keypairs_impl!();
}

impl Signers for [Arc<dyn Signer>; 1] {
    default_keypairs_impl!();
}

impl Signers for [Arc<dyn Signer>; 2] {
    default_keypairs_impl!();
}

impl Signers for [Arc<dyn Signer>; 3] {
    default_keypairs_impl!();
}

impl Signers for [Arc<dyn Signer>; 4] {
    default_keypairs_impl!();
}

impl Signers for Vec<Arc<dyn Signer>> {
    default_keypairs_impl!();
}

impl Signers for Vec<&dyn Signer> {
    default_keypairs_impl!();
}

impl Signers for [&dyn Signer] {
    default_keypairs_impl!();
}

impl Signers for [&dyn Signer; 0] {
    default_keypairs_impl!();
}

impl Signers for [&dyn Signer; 1] {
    default_keypairs_impl!();
}

impl Signers for [&dyn Signer; 2] {
    default_keypairs_impl!();
}

impl Signers for [&dyn Signer; 3] {
    default_keypairs_impl!();
}

impl Signers for [&dyn Signer; 4] {
    default_keypairs_impl!();
}

impl<T: Signer> Signers for [&T; 0] {
    default_keypairs_impl!();
}

impl<T: Signer> Signers for [&T; 1] {
    default_keypairs_impl!();
}

impl<T: Signer> Signers for [&T; 2] {
    default_keypairs_impl!();
}

impl<T: Signer> Signers for [&T; 3] {
    default_keypairs_impl!();
}

impl<T: Signer> Signers for [&T; 4] {
    default_keypairs_impl!();
}

impl<T: Signer> Signers for Vec<&T> {
    default_keypairs_impl!();
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;

    use super::*;

    struct Foo;
    #[async_trait]
    impl Signer for Foo {
        fn try_pubkey(&self) -> Result<Pubkey, SignerError> {
            Ok(Pubkey::default())
        }
        async fn try_sign_message(&self, _message: &[u8]) -> Result<Signature, SignerError> {
            Ok(Signature::default())
        }
        fn is_interactive(&self) -> bool {
            false
        }
    }

    struct Bar;
    #[async_trait]
    impl Signer for Bar {
        fn try_pubkey(&self) -> Result<Pubkey, SignerError> {
            Ok(Pubkey::default())
        }
        async fn try_sign_message(&self, _message: &[u8]) -> Result<Signature, SignerError> {
            Ok(Signature::default())
        }
        fn is_interactive(&self) -> bool {
            false
        }
    }

    #[tokio::test]
    async fn test_dyn_keypairs_compile() {
        let xs: Vec<Box<dyn Signer>> = vec![Box::new(Foo {}), Box::new(Bar {})];
        assert_eq!(
            xs.sign_message(b"").await,
            vec![Signature::default(), Signature::default()],
        );

        // Same as above, but less compiler magic.
        let xs_ref: &[Box<dyn Signer>] = &xs;
        assert_eq!(
            Signers::sign_message(xs_ref, b"").await,
            vec![Signature::default(), Signature::default()],
        );
    }

    #[tokio::test]
    async fn test_dyn_keypairs_by_ref_compile() {
        let foo = Foo {};
        let bar = Bar {};
        let xs: Vec<&dyn Signer> = vec![&foo, &bar];
        assert_eq!(
            xs.sign_message(b"").await,
            vec![Signature::default(), Signature::default()],
        );

        // Same as above, but less compiler magic.
        let xs_ref: &[&dyn Signer] = &xs;
        assert_eq!(
            Signers::sign_message(xs_ref, b"").await,
            vec![Signature::default(), Signature::default()],
        );
    }
}

use did_doc::SignatureSuite;
use did_doc::SignatureData;
use did_doc::Error;
use did_doc::Result;
use serde::Serialize;
use digest::Output;
use digest::Digest;

use crate::crypto::merkle_tree::Proof;
use crate::crypto::merkle_tree::MTree;
use crate::crypto::merkle_tree::Hash;
use crate::crypto::merkle_tree::DigestExt;
use crate::crypto::PublicKey;
use crate::proof::JcsEd25519Signature2020;
use crate::utils::encode_b58;
use crate::utils::decode_b58;

const SIGNATURE_NAME: &str = "Ed25519MerkleSignature2021";

#[derive(Clone, Copy, Debug)]
pub enum MerkleState<'a, D: Digest> {
  Signer {
    node: usize,
    tree: &'a MTree<D>,
  },
  Verifier {
    key: &'a PublicKey,
  },
}

#[derive(Clone, Copy, Debug)]
pub struct JcsEd25519MerkleSignature2021<'a, D: Digest> {
  state: MerkleState<'a, D>,
}

impl<'a, D: Digest> JcsEd25519MerkleSignature2021<'a, D> {
  pub fn new_signer(tree: &'a MTree<D>, node: usize) -> Self {
    Self {
      state: MerkleState::Signer {
        node,
        tree,
      },
    }
  }

  pub fn new_verifier(key: &'a PublicKey) -> Self {
    Self {
      state: MerkleState::Verifier {
        key,
      }
    }
  }
}

impl<'a, D: Digest> SignatureSuite for JcsEd25519MerkleSignature2021<'a, D>
where
  Output<D>: Copy,
{
  fn name(&self) -> &'static str {
    SIGNATURE_NAME
  }

  fn sign<T>(&self, message: &T, secret: &[u8]) -> Result<SignatureData>
  where
    T: Serialize,
  {
    match self.state {
      MerkleState::Signer { node, tree } => {
        let proof: Vec<u8> = tree
          .proof(node)
          .ok_or(Error::message("Merkle Key Invalid Node"))?
          .encode();

        let signature: SignatureData = JcsEd25519Signature2020.sign(message, secret)?;
        let signature: String = format!("{}.{}", encode_b58(&proof), signature.as_str());
        let signature: SignatureData = SignatureData::Signature(signature);

        Ok(signature)
      }
      MerkleState::Verifier { .. } => {
        Err(Error::message("Invalid Merkle Key State"))
      }
    }
  }

  fn verify<T>(&self, message: &T, signature: &SignatureData, public: &[u8]) -> Result<()>
  where
    T: Serialize,
  {
    match self.state {
      MerkleState::Signer { .. } => {
        Err(Error::message("Invalid Merkle Key State"))
      }
      MerkleState::Verifier { key } => {
        let (proof, signature): (&str, &str) = signature
          .as_str()
          .find('.')
          .ok_or(Error::message("Merkle Key Invalid Signature"))
          .map(|index| signature.as_str().split_at(index))
          .map(|(this, that)| (this, that.trim_start_matches('.')))?;

        let root: Hash<D> = Hash::from_slice(public)
          .ok_or(Error::message("Merkle Key Invalid Root Hash"))?;

        let proof: Proof<D> = decode_b58(proof)
          .ok()
          .as_deref()
          .and_then(Proof::decode)
          .ok_or(Error::message("Merkle Key Invalid Proof"))?;

        let target: Hash<D> = D::new().hash_data(key.as_ref());

        if !proof.verify(&root, target) {
          return Err(Error::message("Merkle Tree Invalid Proof"));
        }

        let signature: SignatureData = SignatureData::Signature(signature.to_string());

        JcsEd25519Signature2020.verify(message, &signature, key.as_ref())?;

        Ok(())
      }
    }
  }
}

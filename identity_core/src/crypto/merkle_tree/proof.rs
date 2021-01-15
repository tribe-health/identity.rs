use core::convert::TryInto;
use core::fmt::Debug;
use core::fmt::Formatter;
use core::fmt::Result;
use digest::Digest;
use digest::generic_array::typenum::Unsigned;
use subtle::ConstantTimeEq;

use crate::crypto::merkle_tree::Node;
use crate::crypto::merkle_tree::Hash;
use crate::crypto::merkle_tree::SIZE_U16;

pub struct Proof<D: Digest> {
  nodes: Box<[Node<D>]>,
}

impl<D: Digest> Proof<D> {
  pub fn new(nodes: Box<[Node<D>]>) -> Self {
    Self { nodes }
  }

  pub fn nodes(&self) -> &[Node<D>] {
    &self.nodes
  }

  pub fn verify(&self, root: &Hash<D>, hash: Hash<D>) -> bool {
    self.root(hash).ct_eq(root).into()
  }

  pub fn root(&self, other: Hash<D>) -> Hash<D> {
    self.root_with(&mut D::new(), other)
  }

  pub fn root_with(&self, digest: &mut D, other: Hash<D>) -> Hash<D> {
    self.nodes.iter().fold(other, |acc, item| item.hash_with(digest, &acc))
  }

  // [ U16(HASH-LEN) U16(PATH-LEN) [ [ U8(NODE-TAG) | HASH(NODE-PATH) ] ... ] ]
  pub fn encode(&self) -> Vec<u8> {
    let hsize: usize = D::OutputSize::USIZE;
    let psize: usize = self.nodes.len();

    assert!(hsize <= u16::MAX as usize);
    assert!(psize <= u16::MAX as usize);

    let capacity: usize = (SIZE_U16 << 1) + (psize * (1 + hsize));

    let mut output: Vec<u8> = Vec::with_capacity(capacity);
    output.extend_from_slice(&(hsize as u16).to_be_bytes());
    output.extend_from_slice(&(psize as u16).to_be_bytes());
    output.extend(self.nodes.iter().flat_map(Node::__stream));

    assert_eq!(output.len(), capacity);

    output
  }

  pub fn decode(slice: &[u8]) -> Option<Self> {
    if slice.len() < SIZE_U16 << 1 {
      return None;
    }

    let hsize: usize = read_u16(&slice[0..2])? as usize;
    let psize: usize = read_u16(&slice[2..4])? as usize;

    let mut nodes: Vec<Node<D>> = Vec::with_capacity(psize);
    let mut slice: &[u8] = slice.get(4..)?;

    for _ in 0..psize {
      let ntag: u8 = slice.get(0).copied()?;
      let hash: Hash<D> = slice.get(1..1 + hsize).and_then(Hash::from_slice)?;
      let node: Node<D> = Node::from_tagged(ntag, hash)?;

      nodes.push(node);

      slice = slice.get(1 + hsize..)?;
    }

    Some(Self::new(nodes.into_boxed_slice()))
  }
}

impl<D: Digest> Debug for Proof<D> {
  fn fmt(&self, f: &mut Formatter) -> Result {
    f.debug_struct("Proof")
      .field("nodes", &self.nodes)
      .finish()
  }
}

fn read_u16(slice: &[u8]) -> Option<u16> {
  slice
    .get(0..2)
    .map(TryInto::try_into)
    .transpose()
    .ok()
    .flatten()
    .map(u16::from_be_bytes)
}

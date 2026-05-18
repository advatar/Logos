//! Minimal binary Merkle tree over `Hash32` leaves.
//!
//! The leaves are sorted before hashing so the same set always produces the
//! same root. Each level pads the right side with the last node when the
//! count is odd. Empty trees return the canonical empty root.
//!
//! Membership and non-membership proofs are produced against the same sorted
//! set. Non-membership is encoded as predecessor/successor membership proofs
//! plus adjacency checks on their sorted leaf indices.

use serde::{Deserialize, Serialize};

use crate::{digest, Hash32};

/// Canonical empty-tree root. Distinct from any non-empty root by domain
/// separation.
pub fn empty_root() -> Hash32 {
    digest("merkle-empty", &[])
}

/// Domain-separate leaves from internal nodes so a leaf hash cannot be
/// confused with a subtree root.
fn hash_leaf(leaf: &Hash32) -> Hash32 {
    digest("merkle-leaf", &[leaf])
}

/// Hash an internal Merkle node from two children.
fn hash_node(left: &Hash32, right: &Hash32) -> Hash32 {
    digest("merkle-node", &[left, right])
}

/// Compute the Merkle root over an arbitrary (unsorted) iterator of leaves.
/// Leaves are de-duplicated, sorted, and domain-separated internally so set
/// semantics hold and a leaf hash cannot be replayed as a subtree root.
pub fn root_from_set<I: IntoIterator<Item = Hash32>>(leaves: I) -> Hash32 {
    let mut sorted: Vec<Hash32> = leaves.into_iter().collect();
    sorted.sort_unstable();
    sorted.dedup();
    if sorted.is_empty() {
        return empty_root();
    }
    let mut level: Vec<Hash32> = sorted.iter().map(hash_leaf).collect();
    while level.len() > 1 {
        let mut next = Vec::with_capacity((level.len() + 1) / 2);
        let mut i = 0;
        while i < level.len() {
            let left = level[i];
            let right = if i + 1 < level.len() {
                level[i + 1]
            } else {
                level[i]
            };
            next.push(hash_node(&left, &right));
            i += 2;
        }
        level = next;
    }
    level[0]
}

/// Sibling path for a single leaf at a given index in a power-of-two-padded
/// tree. `is_left[i] = true` means the sibling at level `i` was the right child
/// (so the current node was the left child).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MerklePath {
    pub siblings: Vec<Hash32>,
    pub is_left: Vec<bool>,
    pub leaf_index: u32,
    pub leaf_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MerkleNeighbor {
    pub leaf: Hash32,
    pub path: MerklePath,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NonMembershipProof {
    pub predecessor: Option<MerkleNeighbor>,
    pub successor: Option<MerkleNeighbor>,
}

pub fn prove_membership(leaves: &[Hash32], target: &Hash32) -> Option<MerklePath> {
    let mut sorted: Vec<Hash32> = leaves.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    let idx = sorted.iter().position(|h| h == target)?;
    Some(path_at(&sorted, idx))
}

fn path_at(sorted: &[Hash32], mut idx: usize) -> MerklePath {
    let leaf_index = idx as u32;
    let leaf_count = sorted.len() as u32;
    let mut level: Vec<Hash32> = sorted.iter().map(hash_leaf).collect();
    let mut siblings = Vec::new();
    let mut is_left = Vec::new();
    while level.len() > 1 {
        let sib_idx = if idx % 2 == 0 {
            let s = if idx + 1 < level.len() { idx + 1 } else { idx };
            is_left.push(true); // current is left child
            s
        } else {
            is_left.push(false); // current is right child
            idx - 1
        };
        siblings.push(level[sib_idx]);
        let mut next = Vec::with_capacity((level.len() + 1) / 2);
        let mut i = 0;
        while i < level.len() {
            let l = level[i];
            let r = if i + 1 < level.len() {
                level[i + 1]
            } else {
                level[i]
            };
            next.push(hash_node(&l, &r));
            i += 2;
        }
        idx /= 2;
        level = next;
    }
    MerklePath {
        siblings,
        is_left,
        leaf_index,
        leaf_count,
    }
}

pub fn verify_membership(root: &Hash32, leaf: &Hash32, path: &MerklePath) -> bool {
    if path.siblings.len() != path.is_left.len() {
        return false;
    }
    if path.leaf_count == 0 || path.leaf_index >= path.leaf_count {
        return false;
    }
    let mut acc = hash_leaf(leaf);
    if path.siblings.is_empty() {
        // Singleton tree: root is the (domain-separated) leaf hash itself.
        return path.leaf_count == 1 && path.leaf_index == 0 && &acc == root;
    }
    for (sib, is_left) in path.siblings.iter().zip(path.is_left.iter()) {
        acc = if *is_left {
            hash_node(&acc, sib)
        } else {
            hash_node(sib, &acc)
        };
    }
    &acc == root
}

pub fn prove_non_membership(leaves: &[Hash32], target: &Hash32) -> Option<NonMembershipProof> {
    let mut sorted: Vec<Hash32> = leaves.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    if sorted.binary_search(target).is_ok() {
        return None;
    }
    if sorted.is_empty() {
        return Some(NonMembershipProof {
            predecessor: None,
            successor: None,
        });
    }
    let insertion = sorted.partition_point(|leaf| leaf < target);
    let predecessor = insertion.checked_sub(1).map(|idx| MerkleNeighbor {
        leaf: sorted[idx],
        path: path_at(&sorted, idx),
    });
    let successor = if insertion < sorted.len() {
        Some(MerkleNeighbor {
            leaf: sorted[insertion],
            path: path_at(&sorted, insertion),
        })
    } else {
        None
    };
    Some(NonMembershipProof {
        predecessor,
        successor,
    })
}

pub fn verify_non_membership(root: &Hash32, target: &Hash32, proof: &NonMembershipProof) -> bool {
    if proof.predecessor.is_none() && proof.successor.is_none() {
        return root == &empty_root();
    }

    let leaf_count = proof
        .predecessor
        .as_ref()
        .map(|n| n.path.leaf_count)
        .or_else(|| proof.successor.as_ref().map(|n| n.path.leaf_count));
    let Some(leaf_count) = leaf_count else {
        return false;
    };
    if leaf_count == 0 {
        return false;
    }

    if let Some(pred) = &proof.predecessor {
        if !(pred.leaf < *target) || !verify_membership(root, &pred.leaf, &pred.path) {
            return false;
        }
        if pred.path.leaf_count != leaf_count {
            return false;
        }
    }
    if let Some(succ) = &proof.successor {
        if !(*target < succ.leaf) || !verify_membership(root, &succ.leaf, &succ.path) {
            return false;
        }
        if succ.path.leaf_count != leaf_count {
            return false;
        }
    }

    match (&proof.predecessor, &proof.successor) {
        (Some(pred), Some(succ)) => pred.path.leaf_index + 1 == succ.path.leaf_index,
        (Some(pred), None) => pred.path.leaf_index + 1 == leaf_count,
        (None, Some(succ)) => succ.path.leaf_index == 0,
        (None, None) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn h(b: u8) -> Hash32 {
        digest("test", &[&[b]])
    }

    #[test]
    fn empty_set_has_canonical_root() {
        let r = root_from_set(std::iter::empty());
        assert_eq!(r, empty_root());
    }

    #[test]
    fn singleton_root_is_domain_separated_leaf() {
        let r = root_from_set([h(1)]);
        // With a single leaf, the root is the leaf's domain-separated hash —
        // crucially NOT the raw `h(1)`, so a leaf cannot pose as a subtree root.
        assert_eq!(r, hash_leaf(&h(1)));
        assert_ne!(r, h(1));
    }

    #[test]
    fn root_is_deterministic_across_insertion_order() {
        let a = root_from_set([h(1), h(2), h(3), h(4)]);
        let b = root_from_set([h(4), h(3), h(2), h(1)]);
        assert_eq!(a, b);
    }

    #[test]
    fn membership_proof_round_trips_for_every_leaf() {
        let leaves = vec![h(7), h(3), h(11), h(2), h(5)];
        let root = root_from_set(leaves.clone());
        for leaf in &leaves {
            let path = prove_membership(&leaves, leaf).unwrap();
            assert!(verify_membership(&root, leaf, &path));
        }
    }

    #[test]
    fn membership_proof_fails_for_non_member() {
        let leaves = vec![h(1), h(2)];
        assert!(prove_membership(&leaves, &h(99)).is_none());
    }

    #[test]
    fn tampered_path_is_rejected() {
        let leaves = vec![h(1), h(2), h(3)];
        let root = root_from_set(leaves.clone());
        let mut path = prove_membership(&leaves, &h(1)).unwrap();
        // Flip one sibling byte.
        if let Some(first) = path.siblings.first_mut() {
            first[0] ^= 0xff;
        }
        assert!(!verify_membership(&root, &h(1), &path));
    }

    #[test]
    fn non_membership_proof_round_trips_between_neighbors() {
        let leaves = vec![h(1), h(3), h(5), h(7)];
        let root = root_from_set(leaves.clone());
        let target = h(4);
        let proof = prove_non_membership(&leaves, &target).unwrap();
        assert!(verify_non_membership(&root, &target, &proof));
    }

    #[test]
    fn non_membership_proof_handles_edges_and_empty_set() {
        let leaves = vec![h(4), h(9)];
        let root = root_from_set(leaves.clone());
        let low = [0u8; 32];
        let high = [0xffu8; 32];
        assert!(verify_non_membership(
            &root,
            &low,
            &prove_non_membership(&leaves, &low).unwrap()
        ));
        assert!(verify_non_membership(
            &root,
            &high,
            &prove_non_membership(&leaves, &high).unwrap()
        ));

        let empty = Vec::new();
        let proof = prove_non_membership(&empty, &low).unwrap();
        assert!(verify_non_membership(&empty_root(), &low, &proof));
    }

    #[test]
    fn non_membership_proof_is_none_for_member_and_rejects_tampering() {
        let leaves = vec![h(1), h(3), h(5)];
        let root = root_from_set(leaves.clone());
        assert!(prove_non_membership(&leaves, &h(3)).is_none());

        let target = h(4);
        let mut proof = prove_non_membership(&leaves, &target).unwrap();
        if let Some(succ) = proof.successor.as_mut() {
            succ.path.leaf_index += 1;
        }
        assert!(!verify_non_membership(&root, &target, &proof));
    }
}

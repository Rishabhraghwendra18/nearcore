use borsh::{BorshDeserialize, BorshSerialize};
#[cfg(feature = "deepsize_feature")]
use deepsize::DeepSizeOf;
use serde::{Deserialize, Serialize};

use near_crypto::Signature;

use crate::hash::{hash, CryptoHash};
use crate::merkle::MerklePath;
use crate::sharding::{EncodedShardChunk, ShardChunk, ShardChunkHeader};
use crate::types::AccountId;
use crate::validator_signer::ValidatorSigner;

/// Serialized TrieNodeWithSize
pub type StateItem = Vec<u8>;

#[cfg_attr(feature = "deepsize_feature", derive(DeepSizeOf))]
#[derive(BorshSerialize, BorshDeserialize, Serialize, Debug, Clone, Eq, PartialEq)]
pub struct PartialState(pub Vec<StateItem>);

/// Double signed block.
#[cfg_attr(feature = "deepsize_feature", derive(DeepSizeOf))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Clone, Debug)]
pub struct BlockDoubleSign {
    pub left_block_header: Vec<u8>,
    pub right_block_header: Vec<u8>,
}

impl std::fmt::Display for BlockDoubleSign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

/// Invalid chunk (body of the chunk doesn't match proofs or invalid encoding).
#[cfg_attr(feature = "deepsize_feature", derive(DeepSizeOf))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Clone, Debug)]
pub struct ChunkProofs {
    /// Encoded block header that contains invalid chunk.
    pub block_header: Vec<u8>,
    /// Merkle proof of inclusion of this chunk.
    pub merkle_proof: MerklePath,
    /// Invalid chunk in an encoded form or in a decoded form.
    pub chunk: MaybeEncodedShardChunk,
}

/// Either `EncodedShardChunk` or `ShardChunk`. Used for `ChunkProofs`.
/// `Decoded` is used to avoid re-encoding an already decoded chunk to construct a challenge.
/// `Encoded` is still needed in case a challenge challenges an invalid encoded chunk that can't be
/// decoded.
#[cfg_attr(feature = "deepsize_feature", derive(DeepSizeOf))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Clone, Debug)]
pub enum MaybeEncodedShardChunk {
    Encoded(EncodedShardChunk),
    Decoded(ShardChunk),
}

/// Doesn't match post-{state root, outgoing receipts, gas used, etc} results after applying previous chunk.
#[cfg_attr(feature = "deepsize_feature", derive(DeepSizeOf))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Clone, Debug)]
pub struct ChunkState {
    /// Encoded prev block header.
    pub prev_block_header: Vec<u8>,
    /// Encoded block header that contains invalid chunnk.
    pub block_header: Vec<u8>,
    /// Merkle proof in inclusion of prev chunk.
    pub prev_merkle_proof: MerklePath,
    /// Previous chunk that contains transactions.
    pub prev_chunk: ShardChunk,
    /// Merkle proof of inclusion of this chunk.
    pub merkle_proof: MerklePath,
    /// Invalid chunk header.
    pub chunk_header: ShardChunkHeader,
    /// Partial state that was affected by transactions of given chunk.
    pub partial_state: PartialState,
}

#[cfg_attr(feature = "deepsize_feature", derive(DeepSizeOf))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Clone, Debug)]
// TODO(#1313): Use Box
#[allow(clippy::large_enum_variant)]
pub enum ChallengeBody {
    BlockDoubleSign(BlockDoubleSign),
    ChunkProofs(ChunkProofs),
    ChunkState(ChunkState),
}

#[cfg_attr(feature = "deepsize_feature", derive(DeepSizeOf))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Clone, Debug)]
#[borsh_init(init)]
pub struct Challenge {
    pub body: ChallengeBody,
    pub account_id: AccountId,
    pub signature: Signature,

    #[borsh_skip]
    pub hash: CryptoHash,
}

impl Challenge {
    pub fn init(&mut self) {
        self.hash = hash(&self.body.try_to_vec().expect("Failed to serialize"));
    }

    pub fn produce(body: ChallengeBody, signer: &dyn ValidatorSigner) -> Self {
        let (hash, signature) = signer.sign_challenge(&body);
        Self { body, account_id: signer.validator_id().clone(), signature, hash }
    }
}

pub type Challenges = Vec<Challenge>;

#[cfg_attr(feature = "deepsize_feature", derive(DeepSizeOf))]
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct SlashedValidator {
    pub account_id: AccountId,
    pub is_double_sign: bool,
}

impl SlashedValidator {
    pub fn new(account_id: AccountId, is_double_sign: bool) -> Self {
        SlashedValidator { account_id, is_double_sign }
    }
}

/// Result of checking challenge, contains which accounts to slash.
/// If challenge is invalid this is sender, otherwise author of chunk (and possibly other participants that signed invalid blocks).
pub type ChallengesResult = Vec<SlashedValidator>;

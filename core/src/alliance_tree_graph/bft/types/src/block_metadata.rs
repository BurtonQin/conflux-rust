// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{account_address::AccountAddress, byte_array::ByteArray};
use anyhow::Result;
use libra_crypto::{secp256k1::Secp256k1Signature, HashValue};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Struct that will be persisted on chain to store the information of the
/// current block.
///
/// The flow will look like following:
/// 1. The executor will pass this struct to VM at the end of a block proposal.
/// 2. The VM will use this struct to create a special system transaction that
/// will modify the on    chain resource that represents the information of the
/// current block. This transaction can't    be emitted by regular users and is
/// generated by each of the validators on the fly. Such    transaction will be
/// executed before all of the user-submitted transactions in the blocks.
/// 3. Once that special resource is modified, the other user transactions can
/// read the consensus    info by calling into the read method of that resource,
/// which would thus give users the    information such as the current leader.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockMetadata {
    id: HashValue,
    timestamp_usec: u64,
    // Since Move doesn't support hashmaps, this vote map would be stored as a
    // vector of key value pairs in the Move module. Thus we need a
    // BTreeMap here to define how the values are being ordered.
    previous_block_votes: BTreeMap<AccountAddress, Secp256k1Signature>,
    proposer: AccountAddress,
}

impl BlockMetadata {
    pub fn new(
        id: HashValue, timestamp_usec: u64,
        previous_block_votes: BTreeMap<AccountAddress, Secp256k1Signature>,
        proposer: AccountAddress,
    ) -> Self
    {
        Self {
            id,
            timestamp_usec,
            previous_block_votes,
            proposer,
        }
    }

    pub fn into_inner(
        self,
    ) -> Result<(ByteArray, u64, ByteArray, AccountAddress)> {
        let id = ByteArray::new(self.id.to_vec());
        let vote_maps =
            ByteArray::new(lcs::to_bytes(&self.previous_block_votes)?);
        Ok((id, self.timestamp_usec, vote_maps, self.proposer))
    }
}
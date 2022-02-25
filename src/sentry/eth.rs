use super::devp2p::*;
use anyhow::anyhow;
use arrayvec::ArrayString;
use enum_primitive_derive::*;
use ethereum_forkid::{ForkFilter, ForkId, ForkHash};
use ethereum_types::*;
use rlp_derive::*;
use std::{collections::BTreeSet, convert::TryFrom, str::FromStr};

pub fn capability_name() -> CapabilityName {
    CapabilityName(ArrayString::from("eth").unwrap())
}

#[derive(Clone, Debug, RlpEncodable, RlpDecodable)]
pub struct StatusMessage {
    pub protocol_version: usize,
    pub network_id: u64,
    pub total_difficulty: U256,
    pub best_hash: H256,
    pub genesis_hash: H256,
    pub fork_id: ForkId,
}

#[derive(Clone, Debug)]
pub struct Forks {
    pub genesis: H256,
    pub forks: BTreeSet<u64>,
}

/// An example of a status data when used for a handshake is:
/// {ProtocolVersion:66 NetworkID:1 TD:+6088371363059432 Head:0xce585e7a973311b8db0470a1739ab9eddb38d7edfe3562c5f9eae1d86518d816 Genesis:0xd4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3 ForkID:{Hash:[252 100 236 4] Next:1150000}}
/// {ProtocolVersion:66 NetworkID:1 TD:+36206751599115524359527 Head:0xfeb27336ca7923f8fab3bd617fcb6e75841538f71c1bcfc267d7838489d9e13d Genesis:0xd4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3 ForkID:{Hash:[183 21 7 125] Next:0}}
/// ```
/// use ethereum_types::{U256, H256};
/// use akula::sentry::eth::{StatusMessage, EthProtocolVersion};
/// use ethereum_forkid::{ForkHash, ForkId};
/// use std::str::FromStr;
///
/// let one_status_message = StatusMessage {
///     protocol_version: EthProtocolVersion::Eth66 as usize,
///     network_id: 1,
///     total_difficulty: U256::from_dec_str("36206751599115524359527").unwrap(),
///     best_hash: H256::from_str("0xfeb27336ca7923f8fab3bd617fcb6e75841538f71c1bcfc267d7838489d9e13d").unwrap(),
///     genesis_hash: H256::from_str("0xd4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3").unwrap(),
///     fork_id: ForkId {
///         hash: ForkHash(0xb715077du32),
///         next: 0,
///     }
/// };
///
/// let another_status_message = StatusMessage {
///     protocol_version: EthProtocolVersion::Eth66 as usize,
///     network_id: 1,
///     total_difficulty: U256::from_dec_str("6088371363059432").unwrap(),
///     best_hash: H256::from_str("0xce585e7a973311b8db0470a1739ab9eddb38d7edfe3562c5f9eae1d86518d816").unwrap(),
///     genesis_hash: H256::from_str("0xd4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3").unwrap(),
///     fork_id: ForkId {
///         hash: ForkHash(0xb715077du32),
///         next: 0,
///     }
///
/// };
/// ```
#[derive(Clone, Debug)]
pub struct StatusData {
    pub network_id: u64,
    pub total_difficulty: U256,
    pub best_hash: H256,
    pub fork_data: Forks,
}

#[derive(Clone, Debug)]
pub struct FullStatusData {
    pub status: StatusData,
    pub fork_filter: ForkFilter,
}

impl TryFrom<ethereum_interfaces::sentry::StatusData> for FullStatusData {
    type Error = anyhow::Error;

    fn try_from(value: ethereum_interfaces::sentry::StatusData) -> Result<Self, Self::Error> {
        let ethereum_interfaces::sentry::StatusData {
            network_id,
            total_difficulty,
            best_hash,
            fork_data,
            max_block,
        } = value;

        let fork_data = fork_data.ok_or_else(|| anyhow!("no fork data"))?;
        let genesis = fork_data
            .genesis
            .ok_or_else(|| anyhow!("no genesis"))?
            .into();

        let fork_filter = ForkFilter::new(max_block, genesis, fork_data.forks.clone());
        let status = StatusData {
            network_id,
            total_difficulty: total_difficulty
                .ok_or_else(|| anyhow!("no total difficulty"))?
                .into(),
            best_hash: best_hash.ok_or_else(|| anyhow!("no best hash"))?.into(),
            fork_data: Forks {
                genesis,
                forks: fork_data.forks.into_iter().collect(),
            },
        };

        Ok(Self {
            status,
            fork_filter,
        })
    }
}

#[derive(Clone, Copy, Debug, Primitive)]
pub enum EthMessageId {
    Status = 0,
    NewBlockHashes = 1,
    Transactions = 2,
    GetBlockHeaders = 3,
    BlockHeaders = 4,
    GetBlockBodies = 5,
    BlockBodies = 6,
    NewBlock = 7,
    NewPooledTransactionHashes = 8,
    GetPooledTransactions = 9,
    PooledTransactions = 10,
    GetNodeData = 13,
    NodeData = 14,
    GetReceipts = 15,
    Receipts = 16,
}

#[derive(Clone, Copy, Debug, Primitive)]
pub enum EthProtocolVersion {
    Eth65 = 65,
    Eth66 = 66,
}

#[test]
fn test_perform_handshake() {
    let one_status_message = StatusMessage {
        protocol_version: EthProtocolVersion::Eth66 as usize,
        network_id: 1,
        total_difficulty: U256::from_dec_str("36206751599115524359527").unwrap(),
        best_hash: H256::from_str("0xfeb27336ca7923f8fab3bd617fcb6e75841538f71c1bcfc267d7838489d9e13d").unwrap(),
        genesis_hash: H256::from_str("0xd4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3").unwrap(),
        fork_id: ForkId {
            hash: ForkHash(0xb715077du32),
            next: 0,
        }
    };

    let another_status_message = StatusMessage {
        protocol_version: EthProtocolVersion::Eth66 as usize,
        network_id: 1,
        total_difficulty: U256::from_dec_str("6088371363059432").unwrap(),
        best_hash: H256::from_str("0xce585e7a973311b8db0470a1739ab9eddb38d7edfe3562c5f9eae1d86518d816").unwrap(),
        genesis_hash: H256::from_str("0xd4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3").unwrap(),
        fork_id: ForkId {
            hash: ForkHash(0xb715077du32),
            next: 0,
        }

    };
}

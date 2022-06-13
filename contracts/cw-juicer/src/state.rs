use cosmwasm_std::Coin;
use cw_storage_plus::{Item, Map};

use juicer::merkle_tree::MerkleTreeWithHistory;
use juicer::verifier::Verifier;

pub const BASE_COIN: Item<Coin> = Item::new("base_coin");

pub const VERIFIER: Item<Verifier> = Item::new("VERIFIER");

pub const COMMITMENTS: Item<MerkleTreeWithHistory> = Item::new("COMMITMENTS");

pub const NULLIFIER_HASHES: Map<String, bool> = Map::new("NULLIFIER_HASHES");

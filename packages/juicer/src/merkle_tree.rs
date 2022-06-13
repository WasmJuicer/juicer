#![allow(non_snake_case)]

use crate::bignum;
use crate::poseidon::Poseidon;
use cosmwasm_std::Uint256 as U256;

use serde::{Deserialize, Serialize};

const ROOT_HISTORY_SIZE: u32 = 100;

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MerkleTreeWithHistory {
    pub levels: u32,
    pub filled_subtrees: Vec<U256>,
    pub zeros: Vec<U256>,
    pub current_root_index: u32,
    pub next_index: u32,
    pub roots: Vec<U256>,

    pub ZERO_VALUE: U256,
}

impl MerkleTreeWithHistory {
    pub fn new(levels: u32) -> Self {
        let mut this: Self = Default::default();
        assert!(levels > 0, "_treeLevels should be greater than zero");
        assert!(levels < 32, "_treeLevels should be less than 32");

        let ZERO_VALUE = bignum!(
            "21663839004416932945382355908790599225266501822907911457504978515578255421292"
        );

        this.levels = levels;
        this.roots = vec![U256::zero(); ROOT_HISTORY_SIZE as usize];

        this.ZERO_VALUE = ZERO_VALUE.clone();

        let mut current_zero = ZERO_VALUE.clone();
        this.zeros.push(current_zero.clone());
        this.filled_subtrees.push(current_zero.clone());

        for _ in 1..levels {
            current_zero = this.hash_left_right(&current_zero, &current_zero);
            this.zeros.push(current_zero.clone());
            this.filled_subtrees.push(current_zero.clone());
        }

        this.roots[0] = this.hash_left_right(&current_zero, &current_zero);
        this
    }

    pub fn hash_left_right(&self, left: &U256, right: &U256) -> U256 {
        let poseidon = Poseidon::new();
        // let mut left_bytes: [u8; 32] = [0; 32];
        // let mut right_bytes: [u8; 32] = [0; 32];

        let left_bytes = left.to_le_bytes();
        let right_bytes = right.to_le_bytes();

        let inputs = vec![left_bytes, right_bytes];

        poseidon.hash_as_u256(inputs).unwrap()
    }

    pub fn insert(&mut self, leaf: &U256) -> Option<u32> {
        let mut idx = self.next_index;
        if idx == 2_u32.saturating_pow(self.levels) {
            //"Merkle tree is full. No more leafs can be added");
            return None;
        }

        self.next_index += 1;
        let mut current_level_hash: U256 = *leaf;
        let mut left: &U256;
        let mut right: &U256;

        for i in 0..(self.levels as u32) {
            if idx % 2 == 0 {
                left = &current_level_hash;
                right = &self.zeros[i as usize];

                self.filled_subtrees[i as usize] = current_level_hash.clone();
            } else {
                left = &self.filled_subtrees[i as usize];
                right = &current_level_hash;
            }

            current_level_hash = self.hash_left_right(left, right);

            idx /= 2;
        }

        self.current_root_index = (self.current_root_index + 1) % ROOT_HISTORY_SIZE;
        self.roots[self.current_root_index as usize] = current_level_hash;
        Some(self.next_index as u32 - 1)
    }

    pub fn is_known_root(&self, root: &U256) -> bool {
        if root == &U256::zero() {
            return false;
        }
        let mut i = self.current_root_index;

        for _ in 0..ROOT_HISTORY_SIZE {
            if *root == self.roots[i as usize] {
                return true;
            }
            if i == 0 {
                i = ROOT_HISTORY_SIZE;
            }

            i -= 1;

            if i == self.current_root_index {
                break;
            }
        }

        false
    }

    pub fn get_last_root(&self) -> U256 {
        self.roots[self.current_root_index as usize].clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::Uint256 as U256;

    #[test]
    fn test_merkletree_new() {
        let mt = MerkleTreeWithHistory::new(16);
        assert_eq!(mt.filled_subtrees[0], mt.ZERO_VALUE);
        assert_eq!(mt.zeros[0], mt.ZERO_VALUE);
    }

    #[test]
    fn test_merkletree_root() {
        let mt = MerkleTreeWithHistory::new(20);
        assert_eq!(mt.filled_subtrees[0], mt.ZERO_VALUE);
        assert_eq!(mt.zeros[0], mt.ZERO_VALUE);

        assert_eq!(
            mt.get_last_root(),
            bignum!(
                "19476726467694243150694636071195943429153087843379888650723427850220480216251"
            )
        )
    }

    #[test]
    fn test_merkletree_insert_single_01() {
        let mut mt = MerkleTreeWithHistory::new(20);
        mt.insert(&U256::from(42 as u32));
        let expected = bignum!(
            "13801027358871474054350913888493740197706640469969388660938924863508695867545"
        );
        let root = mt.get_last_root();
        assert_eq!(root, expected);
    }

    // #[test]
    // fn test_merkletree_insert_single_3() {
    //     let mut mt = MerkleTreeWithHistory::new(3);
    //     mt.insert(&U256::one());
    //     let expected = bignum!(
    //         "14817887234532324632578486942317778767513333548116388705259454362287888156301"
    //     );
    //     let root = mt.get_last_root();
    //     assert_eq!(root, expected);
    // }

    // #[test]
    // fn test_merkletree_insert_single_16() {
    //     let mut mt = MerkleTreeWithHistory::new(16);
    //     mt.insert(&U256::from(5 as i64));
    //     let expected = bignum!(
    //         "20078220768011993253497856250024317483006104588209594787144509816521675548945"
    //     );
    //     assert_eq!(mt.current_root_index, 1);
    //     let root = mt.get_last_root();
    //     assert_eq!(root, expected);
    // }

    // #[test]
    // fn test_merkletree_insert() {
    //     let mut mt = MerkleTreeWithHistory::new(16);
    //     let expected = vec![
    //         bignum!("3431256714363396804770991575090970055302175921802683225882378599453141462503"),
    //         bignum!("7575821202546991722047889195143698024641067539407824397010939985717182566799"),
    //         bignum!("7102419650151881575380791103194015368648640006236895399277475380346088306449"),
    //         bignum!("3663265918960820756765744378616083555095944410653161772251208095179127101510"),
    //         bignum!(
    //             "15302658532613586889202868102641369060511299011842796454718345900410135644534"
    //         ),
    //         bignum!(
    //             "19867311980617909474730049456052719869948526667934900087741729669853083711560"
    //         ),
    //         bignum!("6061878619835624285838818217971195365504071979555702464817484176105688178577"),
    //         bignum!("2521963888311190328687829229664642120391801081246544527123137783093792814465"),
    //         bignum!(
    //             "10214875608306830392931189580024717263641319338206990452441323784791611321245"
    //         ),
    //         bignum!("7692234562883530752899755807890957688721742766928110244142163893445927985263"),
    //     ];

    //     for i in 1_u32..11 {
    //         mt.insert(&U256::from(i as u128));
    //         assert_eq!(mt.current_root_index, i);
    //         assert_eq!(mt.get_last_root(), expected[i - 1], "{}", i);
    //     }
    // }

    // // TODO(albttx): add an option to skip or not the test
    // // This test takes ~60s
    // // #[test]
    // // fn test_tree_full() {
    // //     let levels = 6;
    // //     let mut mt = MerkleTreeWithHistory::new(6);

    // //     for i in 0..(2_u128.pow(levels)) {
    // //         assert!(mt.insert(&U256::from(i + 42)).is_some());
    // //     }

    // //     assert!(mt.insert(&U256::from(1337)).is_none());
    // // }

    // #[test]
    // fn test_insert_root() {
    //     let mut mt = MerkleTreeWithHistory::new(16);

    //     mt.insert(
    //         &U256::from_dec_str(
    //             "8144601074668623426925770169834644636770764159380454737463139103752848208415",
    //         )
    //         .unwrap(),
    //     );
    //     // mt.insert(&*bignum!(
    //     //     "8144601074668623426925770169834644636770764159380454737463139103752848208415"
    //     // ));
    //     let expected_root = U256::from_dec_str(
    //         "18759831220824932236585314001088159476096807910838182935046606337929711439019",
    //     )
    //     .unwrap();
    //     assert_eq!(Box::new(expected_root), mt.roots[1]);
    // }

    // #[test]
    // fn test_insert_22root_2() {
    //     let mut mt = MerkleTreeWithHistory::new(20);
    //     mt.insert(&*bignum!(
    //         "8144601074668623426925770169834644636770764159380454737463139103752848208415"
    //     ));
    //     let expected_root = bignum!(
    //         "18141211044530898481780712096785380507009040886197825359491225784587697908689"
    //     );
    //     assert_eq!(expected_root, mt.roots[1]);
    // }

    // #[test]
    // fn test_js_is_known_root() {
    //     let mut tree = MerkleTreeWithHistory::new(20);
    //     tree.insert(&*bignum!(
    //         "1866323185055346905292342102045161906568087644423856689017414658012314956636"
    //     ));
    //     tree.insert(&*bignum!(
    //         "4016374133574857326600334351736513770264093401701386212397218194262148592751"
    //     ));

    //     let ret = tree.is_known_root(&*bignum!(
    //         "1367642430770975058816244910087815566771626808166978076304510393771136408411"
    //     ));
    //     assert!(ret);
    //     // let expected_root = bignum!(
    //     //     "18141211044530898481780712096785380507009040886197825359491225784587697908689"
    //     // );
    //     // assert_eq!(expected_root, tree.roots[1]);
    // }
}

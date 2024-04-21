/*
* Copyright (C) 2019-2024 EverX. All Rights Reserved.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific EVERX DEV software governing permissions and
* limitations under the License.
*/
#![allow(clippy::inconsistent_digit_grouping, clippy::unusual_byte_groupings)]
use crate::{AccountIdPrefixFull, BlockIdExt, read_single_root_boc, SliceData};
use super::*;

use std::{collections::HashSet, str::FromStr};

fn parse_shard_state_unsplit(ss: ShardStateUnsplit) {
    println!("messages");
    let mut len = 0;
    ss.read_out_msg_queue_info().unwrap().out_queue().iterate_objects(|message| {
        println!("message: {:?}", message);
        len += 1;
        Ok(true)
    }).unwrap();
    println!("count: {}", len);

    println!("accounts");
    let mut len = 0;
    ss.read_accounts().unwrap().iterate_objects(|sh_account_ref| {
        let account = sh_account_ref.read_account().unwrap();
        println!("account: {}", account.get_id().unwrap());
        println!("  balance: {}", account.get_balance().unwrap());

        len += 1;
        Ok(true)
    }).unwrap();
    println!("count: {}", len);
    println!();

    if let Some(custom) = ss.read_custom().unwrap() {
        println!("custom.validator_info: {:?}", custom.validator_info);
        println!("custom.config.address {}", custom.config.config_addr);
        println!("custom.configparams");
        crate::dump_config(&custom.config.config_params);

        let mut i: u64 = 0;
        custom.prev_blocks.iterate_with_keys(|seq_no, blkref| -> Result<bool> {
            println!("\tblock seq_no: {} | {:?}", seq_no, blkref);
            i += 1;
            Ok(i <= 5)
        }).unwrap();
        println!("Old mc blocks info: let = {}", custom.prev_blocks.len().unwrap());
    }
}

#[test]
fn test_real_ton_shardstate() {
    // getstate (-1,8000000000000000,0)
    let in_path = "src/tests/data/shard_state.boc";
    println!("ShardState file: {:?}", in_path);
    let bytes = std::fs::read(in_path).unwrap();
    let root_cell = read_single_root_boc(bytes).unwrap();
    println!("cell = {:#.2}", root_cell);
    
    let ss = ShardState::construct_from_cell(root_cell).unwrap();

    match ss {
        ShardState::UnsplitState(ss) => {
            parse_shard_state_unsplit(ss);
        },
        ShardState::SplitState(ss) => {
            parse_shard_state_unsplit(ShardStateUnsplit::construct_from_cell(ss.left).unwrap());
            parse_shard_state_unsplit(ShardStateUnsplit::construct_from_cell(ss.right).unwrap());
        }
    }
}

#[test]
fn test_shard_state_unsplit_serialize() {
    let in_path = "src/tests/data/shard_state.boc";
    let bytes = std::fs::read(in_path).unwrap();
    let root_cell = read_single_root_boc(bytes).unwrap();

    let ss = ShardState::construct_from_cell(root_cell).unwrap();

    match ss {
        ShardState::UnsplitState(mut ss) => {
            let cell = ss.clone().serialize().unwrap();
            let restored_ss = ShardStateUnsplit::construct_from_cell(cell).unwrap();
            assert_eq!(restored_ss, ss);

            let mut copyleft_rewards = CopyleftRewards::default();
            let address = MsgAddressInt::with_standart(None, 0, AccountId::from([1; 32])).unwrap();
            copyleft_rewards.set(&address.address(), &100.into()).unwrap();
            let address = MsgAddressInt::with_standart(None, 0, AccountId::from([2; 32])).unwrap();
            copyleft_rewards.set(&address.address(), &200.into()).unwrap();
            ss.set_copyleft_reward(copyleft_rewards).unwrap();
            assert_eq!(ss.read_custom().unwrap().unwrap().state_copyleft_rewards.len().unwrap(), 2);

            let cell = ss.clone().serialize().unwrap();
            let restored_ss = ShardStateUnsplit::construct_from_cell(cell).unwrap();
            assert_eq!(restored_ss, ss);
            assert_eq!(restored_ss.read_custom().unwrap().unwrap(), ss.read_custom().unwrap().unwrap());
        },
        ShardState::SplitState(_) => {
            unreachable!()
        }
    }
}

#[test]
fn test_shard_state_unsplit_serialize_fast_finality() {
    use crate::write_read_and_assert;

    
    let in_path = "src/tests/data/shard_state.boc";
    let bytes = std::fs::read(in_path).unwrap();
    let root_cell = read_single_root_boc(&bytes).unwrap();

    let ss = ShardState::construct_from_cell(root_cell).unwrap();

    match ss {
        ShardState::UnsplitState(mut ss) => {
            *ss.shard_mut() = ShardIdent::with_tagged_prefix(0, SHARD_FULL).unwrap();
            ss.write_custom(None).unwrap();
            let mut ids = HashSet::new();
            ids.insert((BlockIdExt {
                shard_id: ShardIdent::with_tagged_prefix(1, 0x4000_0000_0000_0000).unwrap(),
                seq_no: 25,
                root_hash: UInt256::rand(),
                file_hash: UInt256::rand(),
            }, 1001000));
            ids.insert((BlockIdExt {
                shard_id: ShardIdent::with_tagged_prefix(1, 0xc000_0000_0000_0000).unwrap(),
                seq_no: 28,
                root_hash: UInt256::rand(),
                file_hash: UInt256::rand(),
            }, 1001000));
            let rsb = RefShardBlocks::with_ids(ids.iter()).unwrap();

            ss.set_ref_shard_blocks(Some(rsb));

            write_read_and_assert(ss);
        },
        ShardState::SplitState(_) => {
            unreachable!()
        }
    }
}

#[test]
fn test_shard_prefix_as_str_with_tag() {
    let sp = ShardIdent::with_prefix_len(
        2,
        0,
        0b0100_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000
    ).unwrap();
    assert_eq!(sp.shard_prefix_as_str_with_tag(), "6000000000000000");

    let sp = ShardIdent::with_prefix_len(
        2,
        0,
        0
    ).unwrap();
    assert_eq!(sp.shard_prefix_as_str_with_tag(), "2000000000000000");

    let sp = ShardIdent::with_prefix_len(
        0,
        0,
        0
    ).unwrap();
    assert_eq!(sp.shard_prefix_as_str_with_tag(), "8000000000000000");

    let sp = ShardIdent::with_prefix_len(
        12,
        0,
        0b0101_1000_00000000_00000000_00000000_00000000_00000000_00000000_00000000
    ).unwrap();
    assert_eq!(sp.shard_prefix_as_str_with_tag(), "5808000000000000");

    let sp = ShardIdent::with_prefix_len(
        60,
        0,
        0b0101_1000_00000000_00000000_00000000_00000000_00000000_00000000_00000000
    ).unwrap();
    assert_eq!(sp.shard_prefix_as_str_with_tag(), "5800000000000008");

    for len in 61_u64..=255_u64 {
        ShardIdent::with_prefix_len(
            len as u8,
            0,
            0b0101_1000_00000000_00000000_00000000_00000000_00000000_00000000_00000000
        ).unwrap_err();
    }

    let sp = ShardIdent::with_tagged_prefix(
        0,
        0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00001000
    ).unwrap();
    assert_eq!(sp.shard_prefix_as_str_with_tag(), "0000000000000008");

    let sp = ShardIdent::with_prefix_slice(0, SliceData::new_empty()).unwrap();
    assert_eq!(sp.shard_prefix_as_str_with_tag(), "8000000000000000");

    ShardIdent::with_tagged_prefix(
        0,
        0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000100
    ).unwrap_err();

    ShardIdent::with_tagged_prefix(
        0,
        0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000010
    ).unwrap_err();

    ShardIdent::with_tagged_prefix(
        0,
        0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000001
    ).unwrap_err();
}

#[test]
fn test_shard_ident_with_tagged_prefix() {
    assert_eq!(
        ShardIdent::with_prefix_len(
            2,
            0,
            0b0100_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000
        ).unwrap(),
        ShardIdent::with_tagged_prefix(0, 0x6000000000000000).unwrap()
    );
    assert_eq!(
        ShardIdent::with_prefix_len(
            2,
            0,
            0
        ).unwrap(),
        ShardIdent::with_tagged_prefix(0, 0x2000000000000000).unwrap()
    );
    assert_eq!(
        ShardIdent::with_prefix_len(
            0,
            0,
            0
        ).unwrap(),
        ShardIdent::with_tagged_prefix(0, 0x8000000000000000).unwrap()
    );
    assert_eq!(
        ShardIdent::with_prefix_len(
            12,
            0,
            0b0101_1000_00000000_00000000_00000000_00000000_00000000_00000000_00000000
        ).unwrap(),
        ShardIdent::with_tagged_prefix(0, 0x5808000000000000).unwrap()
    );
}

#[test]
fn test_shard_ident_with_prefix_slice() {
    assert_eq!(
        ShardIdent::with_prefix_len(
            2,
            0,
            0b0100_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000
        ).unwrap(),
        ShardIdent::with_prefix_slice(0, SliceData::from_string("6_").unwrap()).unwrap()
    );
    assert_eq!(
        ShardIdent::with_prefix_len(
            2,
            0,
            0
        ).unwrap(),
        ShardIdent::with_prefix_slice(0, SliceData::from_string("2_").unwrap()).unwrap()
    );
    assert_eq!(
        ShardIdent::with_prefix_len(
            0,
            0,
            0
        ).unwrap(),
        ShardIdent::with_prefix_slice(0, SliceData::from_string("_").unwrap()).unwrap()
    );
    assert_eq!(
        ShardIdent::with_prefix_len(
            12,
            0,
            0b0101_1000_00000000_00000000_00000000_00000000_00000000_00000000_00000000
        ).unwrap(),
        ShardIdent::with_prefix_slice(0, SliceData::from_string("5808_").unwrap()).unwrap()
    );
}

#[test]
fn test_shard_ident_merge() {
    assert!(
        ShardIdent::with_tagged_prefix(0, 0b1000_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap().merge().is_err()
    );
    assert_eq!(
        ShardIdent::with_tagged_prefix(0, 0b0101_1000_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap().merge().unwrap(),
        ShardIdent::with_tagged_prefix(0, 0b0101_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap(),
    );
    assert_eq!(
        ShardIdent::with_tagged_prefix(0, 0b0100_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap().merge().unwrap(),
        ShardIdent::with_tagged_prefix(0, 0b1000_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap(),
    );
    assert_eq!(
        ShardIdent::with_tagged_prefix(0, 0b0101_1001_00000000_00010000_00000000_00000000_00000000_00000000_00000000).unwrap().merge().unwrap(),
        ShardIdent::with_tagged_prefix(0, 0b0101_1001_00000000_00100000_00000000_00000000_00000000_00000000_00000000).unwrap(),
    );
    assert_eq!(
        ShardIdent::with_tagged_prefix(0, 0b1110_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap().merge().unwrap(),
        ShardIdent::with_tagged_prefix(0, 0b1100_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap(),
    );
}

#[test]
fn test_shard_ident_is_ancestor_for() {
    assert!(
        ShardIdent::with_tagged_prefix(0, 0b0101_1000_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap()
        .is_ancestor_for(
            &ShardIdent::with_tagged_prefix(0, 0b0101_1010_00000000_00000000_00000000_01000000_00000000_01100000_10000000).unwrap()
        )
    );
    assert!(
        !ShardIdent::with_tagged_prefix(-1, 0b0101_1000_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap()
        .is_ancestor_for(
            &ShardIdent::with_tagged_prefix(0, 0b0101_1010_00000000_00000000_00000000_01000000_00000000_01100000_10000000).unwrap()
        )
    );
    assert!(
        ShardIdent::with_tagged_prefix(0, 0b1000_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap()
        .is_ancestor_for(
            &ShardIdent::with_tagged_prefix(0, 0b0101_1001_00000000_00010000_00000000_00000000_00000000_00000000_00000000).unwrap()
        )
    );
    assert!(
        ShardIdent::with_tagged_prefix(0, 0b1000_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap()
        .is_ancestor_for(
            &ShardIdent::with_tagged_prefix(0, 0b1000_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap()
        )
    );
    assert!(
        !ShardIdent::with_tagged_prefix(0, 0b1001_1000_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap()
        .is_ancestor_for(
            &ShardIdent::with_tagged_prefix(0, 0b1000_1000_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap()
        )
    );
}

#[test]
fn test_shard_ident_is_parent_for() {
    assert!(
        ShardIdent::with_tagged_prefix(0, 0b0101_1000_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap()
        .is_parent_for(
            &ShardIdent::with_tagged_prefix(0, 0b0101_1100_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap()
        )
    );
    assert!(
        !ShardIdent::with_tagged_prefix(-1, 0b0101_1000_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap()
        .is_parent_for(
            &ShardIdent::with_tagged_prefix(0, 0b0101_1100_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap()
        )
    );
    assert!(
        ShardIdent::with_tagged_prefix(0, 0b1000_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap()
        .is_parent_for(
            &ShardIdent::with_tagged_prefix(0, 0b0100_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap()
        )
    );
    assert!(
        !ShardIdent::with_tagged_prefix(0, 0b1000_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap()
        .is_parent_for(
            &ShardIdent::with_tagged_prefix(0, 0b1000_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap()
        )
    );
    assert!(
        !ShardIdent::with_tagged_prefix(0, 0b1001_1000_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap()
        .is_parent_for(
            &ShardIdent::with_tagged_prefix(0, 0b1001_1000_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap()
        )
    );
    assert!(
        !ShardIdent::with_tagged_prefix(0, 0b1100_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap()
        .is_parent_for(
            &ShardIdent::with_tagged_prefix(0, 0b1000_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap()
        )
    );
}

#[test]
fn test_shard_ident_split() {
    assert!(
        ShardIdent::with_tagged_prefix(0, 0b0000_0000_00000000_00000000_00000000_00000000_00000000_00000000_0000_1000).unwrap().split().is_err(),
    );
    assert_eq!(
        ShardIdent::with_tagged_prefix(0, 0b0000_1111_00000000_00000000_00000000_00000000_00000000_00000000_0001_0000).unwrap().split().unwrap(),
        (
            ShardIdent::with_tagged_prefix(0, 0b0000_1111_00000000_00000000_00000000_00000000_00000000_00000000_0000_1000).unwrap(),
            ShardIdent::with_tagged_prefix(0, 0b0000_1111_00000000_00000000_00000000_00000000_00000000_00000000_0001_1000).unwrap(),
        )
    );
    assert_eq!(
        ShardIdent::with_tagged_prefix(0, 0b0101_1000_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap().split().unwrap(),
        (
            ShardIdent::with_tagged_prefix(0, 0b0101_0100_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap(),
            ShardIdent::with_tagged_prefix(0, 0b0101_1100_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap(),
        )
    );
    assert_eq!(
        ShardIdent::with_tagged_prefix(0, 0b0101_1001_00000000_00010000_00000000_00000000_00000000_00000000_00000000).unwrap().split().unwrap(),
        (
            ShardIdent::with_tagged_prefix(0, 0b0101_1001_00000000_00001000_00000000_00000000_00000000_00000000_00000000).unwrap(),
            ShardIdent::with_tagged_prefix(0, 0b0101_1001_00000000_00011000_00000000_00000000_00000000_00000000_00000000).unwrap(),
        )
    );
    assert_eq!(
        ShardIdent::with_tagged_prefix(0, 0b1110_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap().split().unwrap(),
        (
            ShardIdent::with_tagged_prefix(0, 0b1101_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap(),
            ShardIdent::with_tagged_prefix(0, 0b1111_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap(),
        )
    );
}

#[test]
fn test_shard_ident_contains_account() {
    assert!(
        ShardIdent::with_prefix_slice(0, SliceData::from_string("7ff95eed4bc8_").unwrap()).unwrap()
            .contains_account(AccountId::from_str("7ff95eed4bc3a5fe1e590d8111f471281d100d2eadc737fd3ee8b209c21a21be").unwrap()).unwrap()
    );
    assert!(
        ShardIdent::with_tagged_prefix(0, 0x6000_0000_0000_0000).unwrap()
            .contains_account(AccountId::from_str("79b1756926764d88d0b9bc8f42806939f293fb7733fba0959bb024234447c900").unwrap()).unwrap()
    );
    assert!(
        ShardIdent::with_prefix_slice(0, SliceData::from_string("_").unwrap()).unwrap()
            .contains_account(AccountId::from_str("7ff95eed4bc3a5fe1e590d8111f471281d100d2eadc737fd3ee8b209c21a21be").unwrap()).unwrap()
    );
    assert!(
        !ShardIdent::with_prefix_slice(0, SliceData::from_string("7ff950ed4bc8_").unwrap()).unwrap()
            .contains_account(AccountId::from_str("7ff95eed4bc3a5fe1e590d8111f471281d100d2eadc737fd3ee8b209c21a21be").unwrap()).unwrap()
    );
}

#[test]
fn test_shard_prefix_without_tag() {
    assert_eq!(
        ShardIdent::with_prefix_len(
            2,
            0,
            0b0100_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000
        ).unwrap().shard_prefix_without_tag(),
        0b0100_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000
    );
    assert_eq!(
        ShardIdent::with_prefix_len(
            2,
            0,
            0
        ).unwrap().shard_prefix_without_tag(),
        0
    );
    assert_eq!(
        ShardIdent::with_prefix_len(
            0,
            0,
            0
        ).unwrap().shard_prefix_without_tag(),
        0
    );
    assert_eq!(
        ShardIdent::with_prefix_len(
            12,
            0,
            0b0101_1000_00000000_00000000_00000000_00000000_00000000_00000000_00000000
        ).unwrap().shard_prefix_without_tag(),
        0b0101_1000_00000000_00000000_00000000_00000000_00000000_00000000_00000000
    );
}

#[test]
fn test_shard_ident_contains_prefix() {
    assert!(
        ShardIdent::with_tagged_prefix(0, 0b0100_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap()
            .contains_prefix(0, 0b0110_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000)
    );
    assert!(
        ShardIdent::with_tagged_prefix(0, 0b0100_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap()
            .contains_prefix(0, 0b0100_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000)
    );
    assert!(
        ShardIdent::with_tagged_prefix(0, 0b0100_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap()
            .contains_prefix(0, 0b0000_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000)
    );
    assert!(
        !ShardIdent::with_tagged_prefix(0, 0b0100_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap()
            .contains_prefix(0, 0b1000_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000)
    );
    assert!(
        !ShardIdent::with_tagged_prefix(0, 0b0100_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap()
            .contains_prefix(0, 0b1100_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000)
    );
    assert!(
        !ShardIdent::with_tagged_prefix(0, 0b0100_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000).unwrap()
            .contains_prefix(0, 0b1010_0000_00000000_00000000_00000000_00000000_00000000_00000000_00000000)
    );
}

#[test]
fn test_shard_siblings() {
    assert_eq!(
        ShardIdent::with_tagged_prefix(0, 0x2000_0000_0000_0000).unwrap().sibling(),
        ShardIdent::with_tagged_prefix(0, 0x6000_0000_0000_0000).unwrap()
    );
    assert_eq!(
        ShardIdent::with_tagged_prefix(0, 0x2000_0000_0000_0000).unwrap(),
        ShardIdent::with_tagged_prefix(0, 0x6000_0000_0000_0000).unwrap().sibling()
    );
}

mod account_id_prefix_full {
    use crate::{MsgAddrStd, AnycastInfo, MsgAddrVar, Number5, Number9, IntermediateAddressSimple, IntermediateAddressExt};
    use super::super::*;

    fn get_anycast_info() -> AnycastInfo {
        let depth = 12;
        AnycastInfo {
            depth: Number5::new_checked(depth, 31).unwrap(),
            rewrite_pfx: SliceData::from_raw(vec![0x32, 0x1F], depth as usize)
        }
    }

    fn get_msg_addr_std_with_workchain_id(workchain_id: i8, anycast: Option<AnycastInfo>) -> MsgAddrStd {
        MsgAddrStd {
            anycast,
            workchain_id,
            address: AccountId::from_raw(vec![
                0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0,
                0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0
            ], 128),
        }
    }

    fn get_msg_addr_std(anycast: Option<AnycastInfo>) -> MsgAddrStd {
        get_msg_addr_std_with_workchain_id(1, anycast)
    }

    fn get_msg_addr_var_with_workchain_id(workchain_id: i32, anycast: Option<AnycastInfo>) -> MsgAddrVar {
        let addr_len = 120;
        MsgAddrVar {
            anycast,
            addr_len: Number9::new_checked(addr_len, 511).unwrap(),
            workchain_id,
            address: SliceData::from_raw(vec![
                0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0,
                0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE
            ], addr_len as usize),
        }
    }

    fn get_msg_addr_var(anycast: Option<AnycastInfo>) -> MsgAddrVar {
        get_msg_addr_var_with_workchain_id(1, anycast)
    }

    #[test]
    fn test_construction() -> Result<()> {
        let expected = AccountIdPrefixFull {
            workchain_id: 1,
            prefix: 0x123456789ABCDEF0
        };

        let address = MsgAddressInt::AddrStd(get_msg_addr_std(None));
        let prefix = AccountIdPrefixFull::prefix(&address)?;

        assert_eq!(prefix, expected);

        let address = MsgAddressInt::AddrVar(get_msg_addr_var(None));
        let prefix = AccountIdPrefixFull::prefix(&address)?;

        assert_eq!(prefix, expected);

        Ok(())
    }

    #[test]
    fn test_construction_anycast() -> Result<()> {
        let expected = AccountIdPrefixFull {
            workchain_id: 1,
            prefix: 0x321456789ABCDEF0
        };

        let address = MsgAddressInt::AddrStd(get_msg_addr_std(Some(get_anycast_info())));
        let prefix = AccountIdPrefixFull::prefix(&address)?;

        assert_eq!(prefix, expected);

        let address = MsgAddressInt::AddrVar(get_msg_addr_var(Some(get_anycast_info())));
        let prefix = AccountIdPrefixFull::prefix(&address)?;

        assert_eq!(prefix, expected);

        Ok(())
    }

    #[test]
    fn test_checked_construction_valid() {
        let address = MsgAddressInt::AddrVar(get_msg_addr_var(None));
        AccountIdPrefixFull::checked_prefix(&address).unwrap();
    }

    #[test]
    fn test_checked_construction_invalid() {
        let address = MsgAddressInt::AddrVar(get_msg_addr_var_with_workchain_id(INVALID_WORKCHAIN_ID, None));
        AccountIdPrefixFull::checked_prefix(&address).unwrap_err();
    }

    #[test]
    fn test_prefix_to_valid() {
        let address = MsgAddressInt::AddrVar(get_msg_addr_var(None));
        let mut prefix = AccountIdPrefixFull::default();
        assert!(AccountIdPrefixFull::prefix_to(&address, &mut prefix));
    }

    #[test]
    fn test_prefix_to_invalid() {
        let address = MsgAddressInt::AddrVar(get_msg_addr_var_with_workchain_id(INVALID_WORKCHAIN_ID, None));
        let mut prefix = AccountIdPrefixFull::default();
        assert!(!AccountIdPrefixFull::prefix_to(&address, &mut prefix));
    }

    #[test]
    fn test_interpolate_addr() {
        let prefix1 = AccountIdPrefixFull {
            workchain_id: 1,
            prefix: 0x123456789ABCDEF0
        };

        let prefix2 = AccountIdPrefixFull {
            workchain_id: -1,
            prefix: 0x0FEDCBA987654321
        };

        assert_eq!(prefix1.interpolate_addr(&prefix2, 0), prefix1);

        assert_eq!(prefix1.interpolate_addr(&prefix2, 1), AccountIdPrefixFull {
            workchain_id: 0x8000_0001u64 as i32,
            prefix: prefix1.prefix
        });

        assert_eq!(prefix1.interpolate_addr(&prefix2, 20), AccountIdPrefixFull {
            workchain_id: 0xFFFF_F001u64 as i32,
            prefix: prefix1.prefix
        });

        assert_eq!(prefix1.interpolate_addr(&prefix2, 32 + 20), AccountIdPrefixFull {
            workchain_id: prefix2.workchain_id,
            prefix: 0x0FEDC6789ABCDEF0
        });

        assert_eq!(prefix1.interpolate_addr(&prefix2, 32 + 64), prefix2);
    }

    #[test]
    fn test_interpolate_addr_intermediate() -> Result<()> {
        let prefix1 = AccountIdPrefixFull {
            workchain_id: 1,
            prefix: 0x123456789ABCDEF0
        };

        let prefix2 = AccountIdPrefixFull {
            workchain_id: -1,
            prefix: 0x0FEDCBA987654321
        };

        assert_eq!(prefix1.interpolate_addr_intermediate(&prefix2, &IntermediateAddress::use_dest_bits(0)?)?, prefix1);

        assert_eq!(prefix1.interpolate_addr_intermediate(&prefix2, &IntermediateAddress::use_dest_bits(1)?)?, AccountIdPrefixFull {
            workchain_id: 0x8000_0001u64 as i32,
            prefix: prefix1.prefix
        });

        assert_eq!(prefix1.interpolate_addr_intermediate(&prefix2, &IntermediateAddress::use_dest_bits(20)?)?, AccountIdPrefixFull {
            workchain_id: 0xFFFF_F001u64 as i32,
            prefix: prefix1.prefix
        });

        assert_eq!(prefix1.interpolate_addr_intermediate(&prefix2, &IntermediateAddress::use_dest_bits(32 + 20)?)?, AccountIdPrefixFull {
            workchain_id: prefix2.workchain_id,
            prefix: 0x0FEDC6789ABCDEF0
        });

        assert_eq!(prefix1.interpolate_addr_intermediate(&prefix2, &IntermediateAddress::use_dest_bits(32 + 64)?)?, prefix2);

        prefix1.interpolate_addr_intermediate(&prefix2, &IntermediateAddress::Simple(
            IntermediateAddressSimple::default()
        )).unwrap_err();

        prefix1.interpolate_addr_intermediate(&prefix2, &IntermediateAddress::Ext(
            IntermediateAddressExt::default()
        )).unwrap_err();

        Ok(())
    }

    #[test]
    fn test_count_matching_bits() {
        let prefix1 = 0x123456789ABCDEF0;
        let prefix2 = 0x0FEDCBA987654321;

        assert_eq!(AccountIdPrefixFull {
            workchain_id: 1,
            prefix: prefix1
        }.count_matching_bits(&AccountIdPrefixFull {
            workchain_id: -1,
            prefix: prefix2
        }), 0);

        assert_eq!(AccountIdPrefixFull {
            workchain_id: 1,
            prefix: prefix1
        }.count_matching_bits(&AccountIdPrefixFull {
            workchain_id: 2,
            prefix: prefix2
        }), 30);

        assert_eq!(AccountIdPrefixFull {
            workchain_id: -1,
            prefix: prefix1
        }.count_matching_bits(&AccountIdPrefixFull {
            workchain_id: -1,
            prefix: prefix2
        }), 35);

        assert_eq!(AccountIdPrefixFull {
            workchain_id: 1,
            prefix: prefix1
        }.count_matching_bits(&AccountIdPrefixFull {
            workchain_id: 1,
            prefix: prefix1
        }), 32 + 64);
    }
}

#[test]
fn test_shard_to_slice() {
    let shard = ShardIdent::with_tagged_prefix(128, 0x6000_0000_0000_0000).unwrap();
    assert_eq!(shard.prefix_len(), 2);
    assert_eq!(shard.shard_key(true), SliceData::from_string("000000806_").unwrap());

    let shard = ShardIdent::masterchain();
    assert_eq!(shard.prefix_len(), 0);
    assert_eq!(shard.shard_key(true), SliceData::from_string("FFFFFFFF").unwrap());

    let shard = ShardIdent::with_tagged_prefix(128, 0x6000_0000_0000_0000).unwrap();
    assert_eq!(shard.prefix_len(), 2);
    assert_eq!(shard.shard_key(false), SliceData::from_string("6_").unwrap());

    let shard = ShardIdent::masterchain();
    assert_eq!(shard.prefix_len(), 0);
    assert_eq!(shard.shard_key(false), SliceData::from_string("").unwrap());
}

#[test]
fn test_shard_intersect_with() {
    let shard1 = ShardIdent::with_tagged_prefix(0, 0x6000_0000_0000_0000).unwrap();
    let shard2 = ShardIdent::with_tagged_prefix(0, 0x7000_0000_0000_0000).unwrap();
    assert!(shard1.intersect_with(&shard2));
    let shard3 = ShardIdent::with_tagged_prefix(0, 0xE000_0000_0000_0000).unwrap();
    assert!(!shard1.intersect_with(&shard3));
}

#[test]
fn test_hypercube_routing() -> Result<()> {
    let prefix1 = AccountIdPrefixFull {
        workchain_id: 1,
        prefix: 0x123456789ABCDEF0
    };

    let prefix2 = AccountIdPrefixFull {
        workchain_id: 1,
        prefix: 0x0FEDCBA987654321
    };

    let cur_shard = ShardIdent::with_prefix_len(12, 1, 0x1230_0000_0000_0000)?;

    prefix1.perform_hypercube_routing(&prefix2, &cur_shard, IntermediateAddress::use_dest_bits(52)?).unwrap_err();
    prefix1.perform_hypercube_routing(&prefix2, &cur_shard, IntermediateAddress::use_dest_bits(44)?).unwrap_err();

    assert_eq!(
        prefix1.perform_hypercube_routing(&prefix2, &cur_shard, IntermediateAddress::use_dest_bits(32)?)?,
        (IntermediateAddress::use_dest_bits(32)?, IntermediateAddress::use_dest_bits(36)?)
    );
    assert_eq!(
        prefix1.perform_hypercube_routing(&prefix2, &cur_shard, IntermediateAddress::use_dest_bits(10)?)?,
        (IntermediateAddress::use_dest_bits(32)?, IntermediateAddress::use_dest_bits(36)?)
    );

    let cur_shard = ShardIdent::with_prefix_len(20, 1, 0x1230_0000_0000_0000)?;

    prefix1.perform_hypercube_routing(&prefix2, &cur_shard, IntermediateAddress::use_dest_bits(96)?).unwrap_err();

    Ok(())
}


#[test]
fn test_can_split() {
    let shard = ShardIdent::with_tagged_prefix(
        0,
        0b0100_0000_00000000_00000000_00000000_00000000_00000000_00001100_00000000
    ).unwrap();
    assert_eq!(shard.can_split(), shard.split().is_ok());

    let shard = ShardIdent::with_tagged_prefix(
        0,
        0b0100_0000_00000000_11111111_00000000_10000000_00000000_00000000_00000000
    ).unwrap();
    assert_eq!(shard.can_split(), shard.split().is_ok());

    let shard = ShardIdent::with_tagged_prefix(
        0,
        0b0101_0000_00000000_11111111_00000000_10000000_00000000_01000000_0001_0000
    ).unwrap();
    assert_eq!(shard.can_split(), shard.split().is_ok());

    let shard = ShardIdent::with_tagged_prefix(
        0,
        0b0100_0000_00000000_11111111_00000000_10000000_01000000_00000000_1000_1000
    ).unwrap();
    assert_eq!(shard.can_split(), shard.split().is_ok());
}

#[test]
fn test_shards_heighbors() {
    let shard1 = ShardIdent::with_tagged_prefix(0, 0b0011_1100_1000 << 52).unwrap();
    let shard2 = ShardIdent::with_tagged_prefix(0, 0b0000_1100_1000 << 52).unwrap();
    let shard3 = ShardIdent::with_tagged_prefix(0, 0b0000_1000_1000 << 52).unwrap();

    assert!(shard1.is_neighbor_for(&shard2));
    assert!(!shard1.is_neighbor_for(&shard3));
}

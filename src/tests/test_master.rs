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

use super::*;
use crate::{
    read_single_root_boc, write_read_and_assert, write_read_and_assert_with_opts, Block, BlockExtra,
    Deserializable, ExtBlkRef, HashmapAugType, MsgAddressInt, ShardStateUnsplit, 
    BASE_WORKCHAIN_ID, SERDE_OPTS_EMPTY, CommonMessage, Transaction, BlockInfo, ValueFlow,
    MerkleUpdate, transactions::tests::generate_test_shard_account_block,
    HashmapType, HashmapE, InMsgFinal,
};
use std::collections::{HashMap, HashSet};
use rand::Rng;

#[test]
fn test_libraries() {
    let mut id = [0u8; 32];
    id[0] = 44;
    let acc_id = AccountId::from(id);

    let mut id = [0u8; 32];
    id[0] = 39;
    let my_id = AccountId::from(id);

    let mut id = [0u8; 32];
    id[0] = 157;
    let your_id = AccountId::from(id);

    let lib_code = SliceData::new(vec![0x11, 0x80]).into_cell();
    let lib1 = LibDescr::from_lib_data_by_publisher(lib_code, my_id.clone());

    let lib_code = SliceData::new(vec![0x75, 0x80]).into_cell();
    let mut lib2 = LibDescr::from_lib_data_by_publisher(lib_code, my_id);
    lib2.publishers_mut().set(&your_id, &()).unwrap();

    let mut data = HashmapE::with_bit_len(256);
    let key = SliceData::load_builder(acc_id.write_to_new_cell().unwrap()).unwrap();
    data.set_builder(key.clone(), &lib1.write_to_new_cell().unwrap()).unwrap();
    data.set_builder(key, &lib2.write_to_new_cell().unwrap()).unwrap();

    let cell = data.serialize().unwrap();
    let mut restored_data = HashmapE::with_bit_len(256);
    restored_data.read_from_cell(cell).unwrap();

    assert_eq!(data, restored_data);
}

#[test]
fn test_shard_descr() {
    let descr_none = ShardDescr::with_params(42, 17, 25, UInt256::from([70; 32]), FutureSplitMerge::None);
    let descr_split = ShardDescr::with_params(42, 17, 25, UInt256::from([70; 32]), FutureSplitMerge::Split{split_utime: 0x12345678, interval: 0x87654321});
    let descr_merge = ShardDescr::with_params(42, 17, 25, UInt256::from([70; 32]), FutureSplitMerge::Merge{merge_utime: 0x12345678, interval: 0x87654321});

    write_read_and_assert(descr_none);
    write_read_and_assert(descr_split);
    write_read_and_assert(descr_merge);
}

#[test]
fn test_shard_descr_with_copyleft() {
    let mut copyleft_rewards = CopyleftRewards::default();
    let address = MsgAddressInt::with_standart(None, 0, AccountId::from([1; 32])).unwrap();
    copyleft_rewards.set(&address.address(), &100.into()).unwrap();
    let address = MsgAddressInt::with_standart(None, 0, AccountId::from([2; 32])).unwrap();
    copyleft_rewards.set(&address.address(), &200.into()).unwrap();

    let mut descr_none = ShardDescr::with_params(42, 17, 25, UInt256::from([70; 32]), FutureSplitMerge::None);
    descr_none.copyleft_rewards = copyleft_rewards.clone();
    let mut descr_split = ShardDescr::with_params(42, 17, 25, UInt256::from([70; 32]), FutureSplitMerge::Split{split_utime: 0x12345678, interval: 0x87654321});
    descr_split.copyleft_rewards = copyleft_rewards.clone();
    let mut descr_merge = ShardDescr::with_params(42, 17, 25, UInt256::from([70; 32]), FutureSplitMerge::Merge{merge_utime: 0x12345678, interval: 0x87654321});
    descr_merge.copyleft_rewards = copyleft_rewards.clone();

    write_read_and_assert(descr_none);
    write_read_and_assert(descr_split);
    write_read_and_assert(descr_merge);
}

#[test]
fn test_shard_descr_fast_finality() {
    let mut descr_none = ShardDescr::with_params(42, 17, 25, UInt256::from([70; 32]), FutureSplitMerge::None);
    descr_none.collators = Some(ShardCollators {
        prev: gen_collator(),
        prev2: None,
        current: gen_collator(),
        next: gen_collator(),
        next2: None,
        updated_at: 0x12345678,
    });

    let mut descr_split = ShardDescr::with_params(42, 17, 25, UInt256::from([70; 32]), FutureSplitMerge::Split{split_utime: 0x12345678, interval: 0x87654321});
    descr_split.collators = Some(ShardCollators {
        prev: gen_collator(),
        prev2: None,
        current: gen_collator(),
        next: gen_collator(),
        next2: Some(gen_collator()),
        updated_at: 0x12345678,
    });

    let mut descr_merge = ShardDescr::with_params(42, 17, 25, UInt256::from([70; 32]), FutureSplitMerge::Merge{merge_utime: 0x12345678, interval: 0x87654321});
    descr_merge.collators = Some(ShardCollators {
        prev: gen_collator(),
        prev2: Some(gen_collator()),
        current: gen_collator(),
        next: gen_collator(),
        next2: None,
        updated_at: 0x12345678,
    });

    write_read_and_assert(descr_none);
    write_read_and_assert(descr_split);
    write_read_and_assert(descr_merge);

}

fn build_mesh_queue_descr() -> ConnectedNwOutDescr {
    ConnectedNwOutDescr {
        out_queue_update: HashUpdate::with_hashes(UInt256::rand(), UInt256::rand()),
        exported: 1234567890.into(),
    }
}

fn build_mesh_descr() -> ConnectedNwDescrExt {
    let mut descr = ConnectedNwDescrExt::default();
    descr.queue_descr = build_mesh_queue_descr();
    descr.descr = Some(ConnectedNwDescr {
        seq_no: 34,
        root_hash: UInt256::rand(),
        file_hash: UInt256::rand(),
        imported: 1234567890.into(),
        gen_utime: 1234567890,
    });
    descr
}

#[test]
fn test_shard_descr_mesh() {
    let mut descr = ShardDescr::with_params(42, 17, 25, UInt256::from([70; 32]), FutureSplitMerge::None);
    descr.collators = Some(ShardCollators {
        prev: gen_collator(),
        prev2: None,
        current: gen_collator(),
        next: gen_collator(),
        next2: None,
        updated_at: 0x12345678,
    });
    let mesh_descr = build_mesh_queue_descr();
    descr.mesh_msg_queues.set(&12345678, &mesh_descr).unwrap();
    write_read_and_assert(descr);

    let mut descr = ShardDescr::with_params(42, 17, 25, UInt256::from([70; 32]), FutureSplitMerge::None);
    descr.mesh_msg_queues.set(&12345678, &mesh_descr).unwrap();
    write_read_and_assert(descr);

}

#[test]
fn test_mc_state_extra() {
    let mut extra = McStateExtra::default();
    let shard1 = ShardDescr::with_params(23, 77, 234, UInt256::from([131; 32]), FutureSplitMerge::None);
    let shard1_1 = ShardDescr::with_params(25, 177, 230, UInt256::from([131; 32]), FutureSplitMerge::None);
    let shard2 = ShardDescr::with_params(15, 78, 235, UInt256::from([77; 32]), FutureSplitMerge::Split{split_utime: 0x12345678, interval: 0x87654321});
    let shard2_2 = ShardDescr::with_params(115, 8, 35, UInt256::from([77; 32]), FutureSplitMerge::Split{split_utime: 0x12345678, interval: 0x87654321});
    let ident = extra.add_workchain(11, &shard1).unwrap();
    extra.shards.split_shard(&ident, |_| Ok((shard1, shard1_1))).unwrap();
    let ident = extra.add_workchain(22, &shard2).unwrap();
    extra.shards.split_shard(&ident, |_| Ok((shard2, shard2_2))).unwrap();

    let key = SliceData::load_builder(123u32.write_to_new_cell().unwrap()).unwrap();
    let value = 0x11u8.write_to_new_cell().unwrap();
    extra.config.config_params.set_builder(key, &value).unwrap();

    extra.prev_blocks.set(&2342, &KeyExtBlkRef {
        key: false,
        blk_ref: ExtBlkRef {
            end_lt: 1,
            seq_no: 999,
            root_hash: UInt256::from([10;32]),
            file_hash: UInt256::from([10;32])
        }
    }, &KeyMaxLt {
        key: false,
        max_end_lt: 1000001
    }).unwrap();
    extra.prev_blocks.set(&664324, &KeyExtBlkRef {
        key: false,
        blk_ref: ExtBlkRef {
            end_lt: 1000,
            seq_no: 1999,
            root_hash: UInt256::from([13;32]),
            file_hash: UInt256::from([14;32])
        }
    }, &KeyMaxLt {
        key: false,
        max_end_lt: 1000002
    }).unwrap();

   write_read_and_assert(extra.clone());

   extra.mesh.set(&1, &ConnectedNwDescr::default()).unwrap();
   extra.mesh.set(&2, &ConnectedNwDescr::default()).unwrap();

   write_read_and_assert(extra.clone());

}

fn build_mc_block_extra(serde_opts: u8) -> McBlockExtra {
    let mut extra = if serde_opts & SERDE_OPTS_COMMON_MESSAGE != 0{
        McBlockExtra::with_common_message_support()
    } else {
        McBlockExtra::default()
    };
    let shard1 = ShardDescr::with_params(23, 77, 234, UInt256::from([131; 32]), FutureSplitMerge::None);
    let shard1_1 = ShardDescr::with_params(25, 177, 230, UInt256::from([131; 32]), FutureSplitMerge::None);
    let shard2 = ShardDescr::with_params(15, 78, 235, UInt256::from([77; 32]), FutureSplitMerge::Split{split_utime: 0x12345678, interval: 0x87654321});
    let shard2_2 = ShardDescr::with_params(115, 8, 35, UInt256::from([77; 32]), FutureSplitMerge::Split{split_utime: 0x12345678, interval: 0x87654321});
    let ident = ShardIdent::with_workchain_id(11).unwrap();
    extra.shards.add_workchain(11, 134, UInt256::default(), UInt256::default(), None).unwrap();
    extra.fees.store_shard_fees(&ident, CurrencyCollection::with_grams(1), CurrencyCollection::with_grams(1)).unwrap();
    extra.shards.split_shard(&ident, |_| Ok((shard1, shard1_1))).unwrap();
    let ident = ShardIdent::with_workchain_id(22).unwrap();
    extra.shards.add_workchain(22, 135, UInt256::default(), UInt256::default(), None).unwrap();
    extra.fees.store_shard_fees(&ident, CurrencyCollection::with_grams(1), CurrencyCollection::with_grams(1)).unwrap();
    extra.shards.split_shard(&ident, |_| Ok((shard2, shard2_2))).unwrap();
    extra.write_recover_create_msg(Some(&InMsg::Final(InMsgFinal::default()))).unwrap();
    extra
}

#[test]
fn test_mc_block_extra() {
    let extra = build_mc_block_extra(0);
    let extra = write_read_and_assert(extra);

    let mut block_extra = BlockExtra::default();
    block_extra.write_account_blocks(&generate_test_shard_account_block(SERDE_OPTS_EMPTY)).unwrap();
    block_extra.write_custom(Some(&extra)).unwrap();

    write_read_and_assert(block_extra);

    // let mut count = 0;
    // restored_extra.shard_hashes.iterate_with_keys(|id: u32, shard_descrs| {
    //     shard_descrs.iterate(|descr| {
    //         count += 1;
    //         println!("{}. {} {}", count, id, descr.0);
    //         Ok(true)
    //     }).unwrap();
    //     Ok(true)
    // }).unwrap();
}

#[test]
fn test_common_msg_mcblockextra() {
    let extra: McBlockExtra = McBlockExtra::with_common_message_support();
    let _extra = write_read_and_assert_with_opts(extra, SERDE_OPTS_COMMON_MESSAGE);
    let mut extra = McBlockExtra::with_common_message_support();
    let opts = SERDE_OPTS_COMMON_MESSAGE;
    let in_msg = InMsg::external(
        ChildCell::with_struct_and_opts(&CommonMessage::default(), opts).unwrap(),
        ChildCell::with_struct_and_opts(&Transaction::with_common_msg_support(AccountId::from([0;32])), opts).unwrap()
    );
    extra.write_recover_create_msg(Some(&in_msg)).unwrap();
    extra.write_mint_msg(Some(&in_msg)).unwrap();
    // extra.write_copyleft_msgs(&[in_msg]).unwrap();
    let _extra = write_read_and_assert_with_opts(extra, SERDE_OPTS_COMMON_MESSAGE).unwrap();
}

#[test]
fn test_mcblockextra_mesh() {

    let mut mc_extra = build_mc_block_extra(SERDE_OPTS_COMMON_MESSAGE);
    mc_extra.mesh_descr_mut().set(&7, &build_mesh_descr()).unwrap();
    mc_extra.mesh_descr().get(&7).unwrap().unwrap();
    let mc_extra2 = write_read_and_assert_with_opts(mc_extra.clone(), SERDE_OPTS_COMMON_MESSAGE).unwrap();
    mc_extra2.mesh_descr().get(&7).unwrap().unwrap();

    let mut extra = BlockExtra::with_common_msg_support();
    extra.write_custom(Some(&mc_extra)).unwrap();
    let extra2 = write_read_and_assert_with_opts(extra, SERDE_OPTS_COMMON_MESSAGE).unwrap();
    let mc_extra3 = extra2.read_custom().unwrap().unwrap();
    mc_extra3.mesh_descr().get(&7).unwrap();

    let block = Block::with_common_msg_support(
        34, &BlockInfo::default(), &ValueFlow::default(), &MerkleUpdate::default(), None, &extra2
    ).unwrap();
    let block2 = write_read_and_assert_with_opts(block, SERDE_OPTS_COMMON_MESSAGE).unwrap();

    let mc_extra4 = block2.read_extra().unwrap().read_custom().unwrap().expect("need mc block extra");
    mc_extra4.mesh_descr().get(&7).unwrap();
}

#[test]
fn test_mc_block_extra_2() {
    let mut extra = build_mc_block_extra(0);
    extra.write_copyleft_msgs(&[InMsg::default(), InMsg::default()]).unwrap();
    write_read_and_assert(extra);
}

#[test]
fn test_mc_block_extra_3() {
    let mut extra = build_mc_block_extra(SERDE_OPTS_COMMON_MESSAGE);
    extra.mesh_descr_mut().set(&12345678, &build_mesh_descr()).unwrap();
    let extra2 = write_read_and_assert_with_opts(extra, SERDE_OPTS_COMMON_MESSAGE).unwrap();
    extra2.read_recover_create_msg().unwrap();
}

#[test]
fn test_serialization_shard_hashes() {
    let mut shard_hashes = ShardHashes::default();

    for n in 0..12i32 {
        let descr = ShardDescr::with_params(42, 17, 25, UInt256::from([n as u8; 32]), FutureSplitMerge::None);
        let shards = BinTree::with_item(&descr).unwrap();
        shard_hashes.set(&n, &InRefValue(shards)).unwrap();
    }

    write_read_and_assert(shard_hashes);
}

#[test]
fn test_real_shard_hashes() {
    let block = Block::construct_from_file("src/tests/data/key_block_not_all_shardes.boc").unwrap();
    let extra = block.read_extra().unwrap().read_custom().unwrap().expect("need key block");
    let shards = extra.shards();
    let mut count = shards.dump("shards");
    println!("total: {}", count);

    let mut result = vec![];
    println!("---- pairs ----");
    shards.iterate_shards_with_siblings(|shard, _descr, sibling| {
        let sib = shard.sibling();
        result.iter().for_each(|item| assert_ne!(item, &sib));
        println!("shard: {}:{:064b} sibling: {}",
            shard.workchain_id(), shard.shard_prefix_with_tag(), sibling.is_some());
        result.push(shard);
        count -= 1;
        count -= sibling.is_some() as usize;
        Ok(true)
    }).unwrap();
    println!("total: {}", result.len());
    println!("----  end  ----");
    assert_eq!(count, 0);

    // 0400000000000000

    let shard = ShardIdent::with_tagged_prefix(0, 0b0000010000000000000000000000000000000000000000000000000000000000).unwrap();

    let found_shard = shards.get_shard(&shard).unwrap();
    assert!(found_shard.is_some());
    assert_eq!(*found_shard.unwrap().shard(), shard);

    let shard2 = ShardIdent::with_tagged_prefix(0, 0b0000011000000000000000000000000000000000000000000000000000000000).unwrap();
    let found_shard = shards.get_shard(&shard2).unwrap();
    assert!(found_shard.is_none());

    let found_shard = shards.find_shard(&shard2).unwrap();
    assert!(found_shard.is_some());
    assert_eq!(*found_shard.unwrap().shard(), shard);

    let left_ancestor_mask = shard.left_ancestor_mask().unwrap();
    let right_ancestor_mask = shard.right_ancestor_mask().unwrap();

    let found_shard = shards.get_shard(&left_ancestor_mask).unwrap();
    assert!(found_shard.is_none());

    let found_shard = shards.get_shard(&right_ancestor_mask).unwrap();
    assert!(found_shard.is_none());

    let found_shard = shards.find_shard(&left_ancestor_mask).unwrap();
    assert!(found_shard.is_some());
    assert_eq!(*found_shard.unwrap().shard(), shard);

    let found_shard = shards.find_shard(&right_ancestor_mask).unwrap();
    assert!(found_shard.is_some());
    assert_eq!(*found_shard.unwrap().shard(), shard);


    // 5400000000000000

    let shard = ShardIdent::with_tagged_prefix(0, 0b0101010000000000000000000000000000000000000000000000000000000000).unwrap();

    let found_shard = shards.get_shard(&shard).unwrap();
    assert!(found_shard.is_some());
    assert_eq!(*found_shard.unwrap().shard(), shard);

    let shard2 = ShardIdent::with_tagged_prefix(0, 0b0101010010000000000100000000000000000000000000000000000000000000).unwrap();
    let found_shard = shards.get_shard(&shard2).unwrap();
    assert!(found_shard.is_none());

    let found_shard = shards.find_shard(&shard2).unwrap();
    assert!(found_shard.is_some());
    assert_eq!(*found_shard.unwrap().shard(), shard);

    let left_ancestor_mask = shard.left_ancestor_mask().unwrap();
    let right_ancestor_mask = shard.right_ancestor_mask().unwrap();

    let found_shard = shards.get_shard(&left_ancestor_mask).unwrap();
    assert!(found_shard.is_none());

    let found_shard = shards.get_shard(&right_ancestor_mask).unwrap();
    assert!(found_shard.is_none());

    let found_shard = shards.find_shard(&left_ancestor_mask).unwrap();
    assert!(found_shard.is_some());
    assert_eq!(*found_shard.unwrap().shard(), shard);

    let found_shard = shards.find_shard(&right_ancestor_mask).unwrap();
    assert!(found_shard.is_some());
    assert_eq!(*found_shard.unwrap().shard(), shard);


    // 1400000000000000 + 1c00000000000000 = 1800000000000000

    let shard = ShardIdent::with_tagged_prefix(0, 0x1800000000000000).unwrap();
    let left_ancestor = ShardIdent::with_tagged_prefix(0, 0x1400000000000000).unwrap();
    let right_ancestor = ShardIdent::with_tagged_prefix(0, 0x1c00000000000000).unwrap();

    let left_ancestor_mask = shard.left_ancestor_mask().unwrap();
    let right_ancestor_mask = shard.right_ancestor_mask().unwrap();

    let found_shard = shards.find_shard(&left_ancestor_mask).unwrap();
    assert!(found_shard.is_some());
    assert_eq!(*found_shard.unwrap().shard(), left_ancestor);

    let found_shard = shards.find_shard(&right_ancestor_mask).unwrap();
    assert!(found_shard.is_some());
    assert_eq!(*found_shard.unwrap().shard(), right_ancestor);
}

#[test]
fn test_serialization_shard_fees() {
    let mut shard_fees = ShardFees::default();

    //let mut summ = 0;
    for n in 1..12u32 {
        //summ += 2 * n * 100;
        let mut cc = CurrencyCollection::with_grams(n as u64 * 100);
        cc.set_other(n, n as u128).unwrap();
        let fee = ShardFeeCreated::with_fee(cc);
        let ident = ShardIdentFull::new(n as i32, 0x8000_0000_0000_0000);
        shard_fees.set_augmentable(&ident, &fee).unwrap();
        assert!(!shard_fees.is_empty());
        //assert_eq!(shard_fees.root_extra().fees.grams, summ.into());
    }

    write_read_and_assert(shard_fees);
}

#[test]
fn test_get_next_prev_key_block() {

    let bytes = std::fs::read("src/tests/data/free-ton-mc-state-61884").unwrap();
    let root = read_single_root_boc(&bytes).unwrap();
    let shard_state = ShardStateUnsplit::construct_from_cell(root).unwrap();
    let prev_blocks = &shard_state.read_custom().unwrap().unwrap().prev_blocks;

    // Find all key blocks by full hashmap's enumerating (brute force)
    let mut all_key_blocks = HashMap::new();
    prev_blocks.iterate_with_keys_and_aug(|seqno, id, aug| {
        if aug.key && seqno != 0{
            println!("{:?}", id);
            all_key_blocks.insert(seqno, id);
        }
        Ok(true)
    }).unwrap();

    let mut seqno = 0;
    let mut key_blocks = vec!();
    while let Some(id) = prev_blocks.get_next_key_block(seqno + 1).unwrap() {
        println!("{:?}", id);
        seqno = id.seq_no;
        key_blocks.push(id);
    }
    assert_eq!(key_blocks.len(), all_key_blocks.len());
    for id in key_blocks.iter() {
        assert!(all_key_blocks.contains_key(&id.seq_no));
    }

    let key_id = key_blocks[key_blocks.len() - 1].clone();
    let id = prev_blocks.get_prev_key_block(key_id.seq_no).unwrap().unwrap();
    assert_eq!(id.root_hash, key_id.root_hash);

    let mut seqno = key_blocks[key_blocks.len() - 1].seq_no + 2;
    let mut key_blocks2 = vec!();
    while let Some(id) = prev_blocks.get_prev_key_block(seqno - 1).unwrap() {
        println!("{:?}", id);
        seqno = id.seq_no;
        if seqno == 0 {
            break;
        }
        key_blocks2.insert(0, id);
    }
    assert_eq!(key_blocks, key_blocks2);

    for id in key_blocks {

        let id = BlockIdExt {
            shard_id: ShardIdent::masterchain(),
            seq_no: id.seq_no,
            root_hash: id.root_hash,
            file_hash: id.file_hash
        };
        assert!(prev_blocks.check_block(&id).is_ok());

        let mut fake_id = id.clone();
        fake_id.root_hash = UInt256::from([123; 32]);
        assert!(prev_blocks.check_block(&fake_id).is_err());

        let mut fake_id = id.clone();
        fake_id.file_hash = UInt256::from([123; 32]);
        assert!(prev_blocks.check_block(&fake_id).is_err());

        let mut fake_id = id.clone();
        fake_id.shard_id = ShardIdent::with_workchain_id(BASE_WORKCHAIN_ID).unwrap();
        assert!(prev_blocks.check_block(&fake_id).is_err());
    }
}

#[test]
fn test_counters() {
    let mut c = Counters::default();
    assert!(c.increase_by(1, 100500));
    assert!(c.increase_by(1, 100501));
    assert!(c.increase_by(1, 100502));
    assert!(c.increase_by(1, 100503));
    assert_eq!(c.total(), 4);
}

fn gen_collator() -> CollatorRange {
    let mut rng = rand::thread_rng();
    let collator = rng.gen_range(0..100);
    let start = rng.gen_range(0..100);
    let finish = rng.gen_range(start..100);
    CollatorRange {
        collator,
        start,
        finish,
    }
}

#[test]
fn test_shard_collators() {
    
    let collators = ShardCollators {
        prev: gen_collator(),
        prev2: Some(gen_collator()),
        current: gen_collator(),
        next: gen_collator(),
        next2: Some(gen_collator()),
        updated_at: 0x12345678,
    };
    write_read_and_assert(collators);

    let collators = ShardCollators {
        prev: gen_collator(),
        prev2: None,
        current: gen_collator(),
        next: gen_collator(),
        next2: None,
        updated_at: 0x12345678,
    };
    write_read_and_assert(collators);

    let collators = ShardCollators {
        prev: gen_collator(),
        prev2: None,
        current: gen_collator(),
        next: gen_collator(),
        next2: Some(gen_collator()),
        updated_at: 0x12345678,
    };
    write_read_and_assert(collators);

    let collators = ShardCollators {
        prev: gen_collator(),
        prev2: Some(gen_collator()),
        current: gen_collator(),
        next: gen_collator(),
        next2: None,
        updated_at: 0x12345678,
    };
    write_read_and_assert(collators);

}

impl RefShardBlocks {
    pub fn collect_ref_shard_blocks(&self) -> Result<HashSet<(BlockIdExt, u64)>> {
        let mut res = HashSet::new();
        self.iterate_shard_block_refs(|block_id, u64| {
            res.insert((block_id, u64));
            Ok(true)
        })?;
        Ok(res)
    }
}

#[test]
fn test_shard_descr_ref_shard_blocks_err() {
    std::env::set_var("RUST_BACKTRACE", "full");

    let mut ids = HashSet::new();
    ids.insert((BlockIdExt {
        shard_id: ShardIdent::with_tagged_prefix(1, 0x4000_0000_0000_0000).unwrap(),
        seq_no: 25,
        root_hash: UInt256::rand(),
        file_hash: UInt256::rand(),
    }, 1000200));
    ids.insert((BlockIdExt {
        shard_id: ShardIdent::with_tagged_prefix(1, 0x9000_0000_0000_0000).unwrap(),
        seq_no: 26,
        root_hash: UInt256::rand(),
        file_hash: UInt256::rand(),
    }, 1000300));
    ids.insert((BlockIdExt {
        shard_id: ShardIdent::with_tagged_prefix(1, 0xb000_0000_0000_0000).unwrap(),
        seq_no: 26,
        root_hash: UInt256::rand(),
        file_hash: UInt256::rand(),
    }, 1000400));
    ids.insert((BlockIdExt {
        shard_id: ShardIdent::with_tagged_prefix(1, 0xc800_0000_0000_0000).unwrap(),
        seq_no: 26,
        root_hash: UInt256::rand(),
        file_hash: UInt256::rand(),
    }, 1000100));
    ids.insert((BlockIdExt {
        shard_id: ShardIdent::with_tagged_prefix(1, 0xd800_0000_0000_0000).unwrap(),
        seq_no: 26,
        root_hash: UInt256::rand(),
        file_hash: UInt256::rand(),
    }, 1000101));
    assert!(RefShardBlocks::with_ids(ids.iter()).is_err());

    let mut ids = HashSet::new();
    ids.insert((BlockIdExt {
        shard_id: ShardIdent::with_tagged_prefix(1, 0xa000_0000_0000_0000).unwrap(),
        seq_no: 25,
        root_hash: UInt256::rand(),
        file_hash: UInt256::rand(),
    }, 1000103));
    assert!(RefShardBlocks::with_ids(ids.iter()).is_err());


    let mut ids = HashSet::new();
    ids.insert((BlockIdExt {
        shard_id: ShardIdent::with_tagged_prefix(1, 0x4000_0000_0000_0000).unwrap(),
        seq_no: 25,
        root_hash: UInt256::rand(),
        file_hash: UInt256::rand(),
    }, 1000105));
    ids.insert((BlockIdExt {
        shard_id: ShardIdent::with_tagged_prefix(1, 0xa000_0000_0000_0000).unwrap(),
        seq_no: 25,
        root_hash: UInt256::rand(),
        file_hash: UInt256::rand(),
    }, 1000100));
    assert!(RefShardBlocks::with_ids(ids.iter()).is_err());

    let mut ids = HashSet::new();
    ids.insert((BlockIdExt {
        shard_id: ShardIdent::with_tagged_prefix(1, 0x4000_0000_0000_0000).unwrap(),
        seq_no: 25,
        root_hash: UInt256::rand(),
        file_hash: UInt256::rand(),
    }, 2000100));
    ids.insert((BlockIdExt {
        shard_id: ShardIdent::with_tagged_prefix(1, 0xc000_0000_0000_0000).unwrap(),
        seq_no: 25,
        root_hash: UInt256::rand(),
        file_hash: UInt256::rand(),
    }, 3000100));
    ids.insert((BlockIdExt {
        shard_id: ShardIdent::with_tagged_prefix(1, 0xb000_0000_0000_0000).unwrap(),
        seq_no: 25,
        root_hash: UInt256::rand(),
        file_hash: UInt256::rand(),
    }, 4000100));
    assert!(RefShardBlocks::with_ids(ids.iter()).is_err());
}

#[test]
fn test_shard_descr_ref_shard_blocks() {
    let mut ids = HashSet::new();
    ids.insert((BlockIdExt {
        shard_id: ShardIdent::with_tagged_prefix(1, 0x4000_0000_0000_0000).unwrap(),
        seq_no: 25,
        root_hash: UInt256::rand(),
        file_hash: UInt256::rand(),
    }, 1000100));
    ids.insert((BlockIdExt {
        shard_id: ShardIdent::with_tagged_prefix(1, 0x9000_0000_0000_0000).unwrap(),
        seq_no: 26,
        root_hash: UInt256::rand(),
        file_hash: UInt256::rand(),
    }, 1000100));
    ids.insert((BlockIdExt {
        shard_id: ShardIdent::with_tagged_prefix(1, 0xb000_0000_0000_0000).unwrap(),
        seq_no: 26,
        root_hash: UInt256::rand(),
        file_hash: UInt256::rand(),
    }, 1000101));
    ids.insert((BlockIdExt {
        shard_id: ShardIdent::with_tagged_prefix(1, 0xc800_0000_0000_0000).unwrap(),
        seq_no: 26,
        root_hash: UInt256::rand(),
        file_hash: UInt256::rand(),
    }, 1000100));
    ids.insert((BlockIdExt {
        shard_id: ShardIdent::with_tagged_prefix(1, 0xd800_0000_0000_0000).unwrap(),
        seq_no: 26,
        root_hash: UInt256::rand(),
        file_hash: UInt256::rand(),
    }, 1000102));
    ids.insert((BlockIdExt {
        shard_id: ShardIdent::with_tagged_prefix(1, 0xf000_0000_0000_0000).unwrap(),
        seq_no: 26,
        root_hash: UInt256::rand(),
        file_hash: UInt256::rand(),
    }, 1000100));
    let rsb = RefShardBlocks::with_ids(ids.iter()).unwrap();
    assert_eq!(rsb.collect_ref_shard_blocks().unwrap(), ids);

    let mut ids = HashSet::new();
    ids.insert((BlockIdExt {
        shard_id: ShardIdent::with_tagged_prefix(1, 0x8000_0000_0000_0000).unwrap(),
        seq_no: 25,
        root_hash: UInt256::rand(),
        file_hash: UInt256::rand(),
    }, 1000104));
    let rsb = RefShardBlocks::with_ids(ids.iter()).unwrap();
    assert_eq!(rsb.collect_ref_shard_blocks().unwrap(), ids);

    let mut ids = HashSet::new();
    ids.insert((BlockIdExt {
        shard_id: ShardIdent::with_tagged_prefix(1, 0x4000_0000_0000_0000).unwrap(),
        seq_no: 25,
        root_hash: UInt256::rand(),
        file_hash: UInt256::rand(),
    }, 1000105));
    ids.insert((BlockIdExt {
        shard_id: ShardIdent::with_tagged_prefix(1, 0xc000_0000_0000_0000).unwrap(),
        seq_no: 26,
        root_hash: UInt256::rand(),
        file_hash: UInt256::rand(),
    }, 1000100));
    let rsb = RefShardBlocks::with_ids(ids.iter()).unwrap();
    assert_eq!(rsb.collect_ref_shard_blocks().unwrap(), ids);

}

#[test]
fn test_connected_network_descr() {
    let cnd = ConnectedNwDescr {
        seq_no: 34,
        root_hash: UInt256::rand(),
        file_hash: UInt256::rand(),
        imported: 1234567890.into(),
        gen_utime: 1234567890,
    };
    write_read_and_assert(cnd);
}
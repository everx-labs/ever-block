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

use std::fs::{read, read_dir};
use std::path::Path;
use std::str::FromStr;

use crate::{
    HashmapAugType, HashmapE,
    AccountBlock, Message, TickTock, write_read_and_assert,
    bintree::BinTreeType, CommonMessage, Transaction,
    types::{AddSub, Grams}, OutMsg, UsageTree,
    AccountId, Cell, read_boc, MsgPackId,
    read_single_root_boc, MsgEnvelope,
    transactions::tests::{generate_test_shard_account_block, create_test_transaction_set},
};
use super::*;

#[test]
fn test_serialize_tick_tock(){

    let tt = TickTock::default();
    let mut tt = write_read_and_assert(tt);

    tt.tick = true;
    let mut tt = write_read_and_assert(tt);

    tt.tock = true;
    write_read_and_assert(tt);
}

fn test_blockinfo(block_info: BlockInfo) {
    let mut block_extra = BlockExtra::new();
    block_extra.write_account_blocks(&generate_test_shard_account_block(SERDE_OPTS_EMPTY)).unwrap();

    let mut collection = CurrencyCollection::with_grams(3);
    collection.set_other(1005004, 2_000_003).unwrap();

    let value_flow = ValueFlow {
        from_prev_blk: collection,
        to_next_blk: CurrencyCollection::default(),
        imported: CurrencyCollection::default(),
        exported: CurrencyCollection::default(),
        fees_collected: CurrencyCollection::default(),
        fees_imported: CurrencyCollection::default(),
        recovered: CurrencyCollection::default(),
        created: CurrencyCollection::default(),
        minted: CurrencyCollection::default(),
        copyleft_rewards: CopyleftRewards::new(),
        mesh_exported: MeshExported::new(),
    };

    let state_update = MerkleUpdate::default();

    let block = Block::with_params(
        0,
        block_info,
        value_flow,
        state_update,
        block_extra,
    ).unwrap();

    let mut block = write_read_and_assert(block);

    let mut qu = OutQueueUpdates::default();
    qu.set(&1, &OutQueueUpdate { is_empty: true, update: MerkleUpdate::default()}).unwrap();
    qu.set(&11, &OutQueueUpdate { is_empty: false, update: MerkleUpdate::default()}).unwrap();
    qu.set(&121, &OutQueueUpdate { is_empty: true, update: MerkleUpdate::default()}).unwrap();
    block.out_msg_queue_updates = Some(qu);

    write_read_and_assert(block);
}

#[test]
#[should_panic]
fn test_block_info_with_invalid_seq_no(){
    let mut info = BlockInfo::new();
    info.set_seq_no(0).unwrap();
}

#[test]
#[should_panic]
fn test_block_info_with_invalid_prev_stuff_1(){
    let mut info = BlockInfo::new();
    info.set_prev_stuff(false, &BlkPrevInfo::default_blocks()).unwrap();

}
#[test]
#[should_panic]
fn test_block_info_with_invalid_prev_stuff_2(){
    let mut info = BlockInfo::new();
    info.set_prev_stuff(true, &BlkPrevInfo::default_block()).unwrap();

}

#[test]
#[should_panic]
fn test_block_info_with_invalid_vertical_stuff_1(){
    let mut info = BlockInfo::new();
    info.set_vertical_stuff(1, 0, None).unwrap();
}
#[test]
#[should_panic]
fn test_block_info_with_invalid_vertical_stuff_2(){
    let mut info = BlockInfo::new();
    info.set_vertical_stuff(1, 0, None).unwrap();
}

#[test]
fn test_block_info_with_seq_no(){

    let mut info = BlockInfo::new();
    info.set_seq_no(1).unwrap();
    info.set_prev_stuff(
        true,
        &BlkPrevInfo::Blocks {
            prev1: ChildCell::with_struct(
                &ExtBlkRef {
                    end_lt: 1,
                    seq_no: 1000,
                    root_hash: UInt256::from([10;32]),
                    file_hash: UInt256::from([10;32])
                }
                ).unwrap(),
            prev2: ChildCell::with_struct(
                    &ExtBlkRef {
                        end_lt: 1,
                        seq_no: 999,
                        root_hash: UInt256::from([10;32]),
                        file_hash: UInt256::from([10;32])
                    }
                ).unwrap()
        }
    ).unwrap();
    write_read_and_assert(info.clone());

    info.set_vertical_stuff(
        1,
        32,
        Some(
            BlkPrevInfo::Block {
                prev: ExtBlkRef {
                        end_lt: 1,
                        seq_no: 1000,
                        root_hash: UInt256::from([10;32]),
                        file_hash: UInt256::from([10;32])
                    }
            }
        )
    ).unwrap();
    write_read_and_assert(info);
}

#[test]
fn test_blockinfo_some_some_none() {
    let mut info = BlockInfo::new();
    info.set_shard(ShardIdent::with_workchain_id(0x22222222).unwrap());
    info.set_seq_no(std::u32::MAX - 22).unwrap();
    info.set_prev_stuff(
        false,
        &BlkPrevInfo::Block {
            prev: ExtBlkRef {
                end_lt: 1,
                seq_no: 1000,
                root_hash: UInt256::from([10;32]),
                file_hash: UInt256::from([10;32])
            }
        }
    ).unwrap();
    test_blockinfo(info);
}

#[test]
fn test_blockinfo_with_pack() {
    let mut info = BlockInfo::new();
    info.set_shard(ShardIdent::with_workchain_id(0x22222222).unwrap());
    info.set_seq_no(std::u32::MAX - 22).unwrap();
    info.set_prev_stuff(
        false,
        &BlkPrevInfo::Block {
            prev: ExtBlkRef {
                end_lt: 1,
                seq_no: 1000,
                root_hash: UInt256::from([10;32]),
                file_hash: UInt256::from([10;32])
            }
        }
    ).unwrap();
    info.write_pack_info(Some(&MsgPackProcessingInfo { 
        last_id: MsgPackId::new(ShardIdent::with_tagged_prefix(0, 0x4000_0000_0000_0000_u64).unwrap(), 2339488, UInt256::rand()),
        last_partially_included: Some(UInt256::rand())
     })).unwrap();
    test_blockinfo(info.clone());

    info.write_pack_info(None).unwrap();
    test_blockinfo(info);
}

#[test]
fn test_currency_collection() {
    let mut cc = CurrencyCollection::from_grams(Grams::one());
    cc.set_other(500,     9_000_000+777).unwrap();
    cc.set_other(1005001, 8_000_000+1005700).unwrap();
    cc.set_other(1005002, 555_000_000+1070500).unwrap();
    cc.set_other(10023,   1_000_000+1).unwrap();
    cc.set_other(1005004, 6_767_000_000+8888).unwrap();
    cc.set_other(10035,   13_000_000+1).unwrap();
    cc.set_other(1005006, 4_000_000+6).unwrap();
    cc.set_other(1005007, 5_000_000+7).unwrap();
    cc.set_other(10047,   1_000_000+1).unwrap();
    cc.set_other(10050,   1_111_000_000+100500).unwrap();
    cc.set_other(1001,    10_042_222_000_000+1006500).unwrap();
    cc.set_other(105,     1_000_000+1).unwrap();
    cc.set_other(1000,    2_000_000+5).unwrap();
    cc.set_other(10500,   3_000_000+6).unwrap();
    cc.set_other(10,      4_000_000+777).unwrap();
    cc.set_other(100,     74_000_000+7).unwrap();
    cc.set_other(1000,    1_000_000+1).unwrap();
    cc.set_other(1005000, 1_005_050_000_000+100500).unwrap();
    cc.set_other(80,      100_500_000_000+8).unwrap();

    write_read_and_assert(cc);
}

#[test]
fn test_value_flow() {
    let mut from_prev_blk  = CurrencyCollection::with_grams(1);
    let mut to_next_blk    = CurrencyCollection::with_grams(1);
    let mut imported       = CurrencyCollection::with_grams(1);
    let mut exported       = CurrencyCollection::with_grams(1);
    let mut fees_collected = CurrencyCollection::with_grams(1);
    let mut fees_imported  = CurrencyCollection::with_grams(1);
    let mut recovered      = CurrencyCollection::with_grams(1);
    let mut created        = CurrencyCollection::with_grams(1);
    let mut minted         = CurrencyCollection::with_grams(1);

    from_prev_blk.set_other(1001,   1_000_000+1).unwrap();
    from_prev_blk.set_other(100500, 9_000_000+777).unwrap();
    from_prev_blk.set_other(100500, 8_000_000+1005700).unwrap();
    from_prev_blk.set_other(100500, 555_000_000+1070500).unwrap();

    to_next_blk.set_other(1002,   1_000_000+1).unwrap();
    to_next_blk.set_other(100500, 6_767_000_000+8888).unwrap();

    imported.set_other(1003,   1_000_000+1).unwrap();
    imported.set_other(100500, 4_000_000+6).unwrap();
    imported.set_other(100500, 5_000_000+7).unwrap();

    exported.set_other(1004,   1_000_000+1).unwrap();
    exported.set_other(100500, 1_111_000_000+100500).unwrap();
    exported.set_other(100500, 1_002_222_000_000+100500).unwrap();
   
    fees_collected.set_other(1005,   1_000_000+1).unwrap();
    fees_collected.set_other(100500, 2_000_000+5).unwrap();
    fees_collected.set_other(100500, 3_000_000+6).unwrap();
    fees_collected.set_other(100500, 4_000_000+777).unwrap();

    fees_imported.set_other(100500, 123).unwrap();

    recovered.set_other(100500, 321).unwrap();

    created.set_other(100,    7_000_000+7).unwrap();

    minted.set_other(100,    1_000_000+1).unwrap();
    minted.set_other(100500, 100_500_000_000+100500).unwrap();
    minted.set_other(8,      100_500_000_000+8).unwrap();
   
    let mut copyleft_rewards = CopyleftRewards::default();
    let address = AccountId::from([1; 32]);
    copyleft_rewards.set(&address, &100.into()).unwrap();
    let address = AccountId::from([2; 32]);
    copyleft_rewards.set(&address, &200.into()).unwrap();

    let value_flow = ValueFlow {        
        from_prev_blk: from_prev_blk.clone(),
        to_next_blk: to_next_blk.clone(),
        imported: imported.clone(),
        exported: exported.clone(),
        fees_collected: fees_collected.clone(),
        fees_imported: fees_imported.clone(),
        recovered: recovered.clone(),
        created: created.clone(),
        minted: minted.clone(),
        copyleft_rewards,
        mesh_exported: MeshExported::new(),
    };

    write_read_and_assert(value_flow);

    let value_flow_without_copyleft = ValueFlow {
        from_prev_blk,
        to_next_blk,
        imported,
        exported,
        fees_collected,
        fees_imported,
        recovered,
        created,
        minted,
        copyleft_rewards: CopyleftRewards::default(),
        mesh_exported: MeshExported::new(),
    };

    write_read_and_assert(value_flow_without_copyleft);
}


fn read_file_de_and_serialise(filename: &Path) -> Cell {
    let orig_bytes = read(Path::new(filename)).unwrap_or_else(|_| panic!("Error reading file {:?}", filename));
    let mut root_cells = read_boc(orig_bytes).expect("Error deserializing BOC").roots;
    root_cells.remove(0)
}

#[test]
fn test_real_ton_boc() {
    for entry in read_dir(Path::new("src/tests/data")).expect("Error reading BOCs dir") {
        let entry = entry.unwrap();
        let in_path = entry.path();
        if !in_path.is_dir() {
            if let Some(_in_file_name) = in_path.clone().file_name() {
                if match in_path.extension() { Some(ext) => ext != "boc", _ => true } {
                    continue;
                }
                println!("BOC file: {:?}", in_path);
                read_file_de_and_serialise(&in_path);				
            }
        }
    }
}

#[test]
fn test_real_ton_mgs() {
    //let in_path = Path::new("src/tests/data/wallet-query.boc");
    //let in_path = Path::new("src/tests/data/new-wallet-query.boc");
    //let in_path = Path::new("src/tests/data/send-to-query.boc"); 
    let in_path = Path::new("src/tests/data/int-msg-query.boc"); 
    
    println!("MSG file: {:?}", in_path);
    let root_cell = read_file_de_and_serialise(in_path);

    println!("slice = {}", root_cell);
    let msg = Message::construct_from_cell(root_cell).unwrap();
    println!("Message = {:?}", msg);
}

fn test_real_block(in_path: &Path) -> Block {
    println!();
    println!("Block file: {:?}", in_path);
    let root_cell = read_file_de_and_serialise(in_path);
    // println!("slice = {}", root_cell);
    
    let block = Block::construct_from_cell(root_cell.clone()).unwrap();

    // TODO: Restore output
    // println!("Block:\n{}\n\n", serde_json::to_string_pretty(&block).unwrap());

    // block.extra().in_msg_descr().iterate(|mut in_msg| {
    //     if let Some(msg) = in_msg.message_mut() {
    //         println!("InMsg:\n{}\n\n", serde_json::to_string_pretty(&msg).unwrap());
    //     }
    //     Ok(true)
    // }).unwrap();

    // block.extra().out_msg_descr().iterate(|mut out_msg| {
    //     if let Some(msg) = out_msg.message_mut() {
    //         println!("OutMsg:\n{}\n\n", serde_json::to_string_pretty(&msg).unwrap());
    //     }
    //     Ok(true)
    // }).unwrap();

    // block.extra().account_blocks().iterate(|account_block| {
    //     println!("AccountBlock ID: {:?}", account_block.account_id());
    //     account_block.transaction_iterate(|transaction| {
    //         println!("\nTransaction: {}\n\n", serde_json::to_string_pretty(&transaction).unwrap());
    //         Ok(true)
    //     })?;
    //     Ok(true)
    // }).unwrap();

    let extra = block.read_extra().unwrap();
    if let Some(custom) = extra.read_custom().unwrap() {
        println!("McBlockExtra\n\nShardes");
        custom.hashes().iterate_with_keys(|key, InRefValue(shard_hashes)| {
            println!("\nnext workchain");
            shard_hashes.iterate(|shard, shard_descr| {
                let shard_ident = ShardIdent::with_prefix_slice(key, shard)?;
                println!(
                    "\n\nshard: {}, shard_descr: {:?}\n\n", 
                    shard_ident.shard_prefix_as_str_with_tag(), 
                    shard_descr
                );
                Ok(true)
            })?;
            Ok(true)
        }).unwrap();

        println!("Fees");
        custom.fees().iterate_with_keys(|key, shard_fees| {
            println!("\n\nkey: {:?}, shard_fees: {:?}\n\n", key, shard_fees);
            Ok(true)
        }).unwrap();
    }

    extra.read_in_msg_descr().unwrap().iterate_objects(|in_msg| {
        println!();
        println!("InMsg: {:?}", in_msg);
        Ok(true)
    }).unwrap();

    extra.read_out_msg_descr().unwrap().iterate_objects(|out_msg| {
        println!();
        println!("OutMsg: {:?}", out_msg);
        Ok(true)
    }).unwrap();

    extra.read_account_blocks().unwrap().iterate_objects(|account_block| {
        //println!("AccountBlock: {:?}", account_block);
        println!("AccountBlock ID: {:?}", account_block.account_id());
        account_block.transaction_iterate(|transaction| {
            println!();
            println!("\tTransaction: {:?}", transaction);
            Ok(true)
        })?;
        Ok(true)
    }).unwrap();
    println!();

    let cell = block.serialize().unwrap();
    assert_eq!(root_cell, cell);
    write_read_and_assert(block)
}

#[test]
fn test_real_ton_key_block() {
    let in_path = Path::new("src/tests/data/key_block.boc");
    let block = test_real_block(in_path);

    if let Some(custom) = block.read_extra().unwrap().read_custom().unwrap() {
        if let Some(c) = custom.config() {
            crate::config_params::dump_config(&c.config_params);
            // let bytes = serialize_toc(c.config_params.data().unwrap()).unwrap();
            // std::fs::write("src/tests/data/config.boc", bytes).unwrap();
        }
    }
}

#[test]
fn test_all_real_ton_block_with_transaction() {
    for entry in read_dir(
        Path::new("src/tests/data/block_with_transaction")
    ).expect("Error reading BOCs dir") {
        let entry = entry.unwrap();
        let in_path = entry.path();
        if !in_path.is_dir() {
            if let Some(_in_file_name) = in_path.clone().file_name() {
                if match in_path.extension() { Some(ext) => ext != "boc", _ => true } {
                    continue;
                }
                test_real_block(&in_path);				
            }
        }
    }
}

#[test]
fn test_real_ton_config() {
    // to get current config run lite_client with saveconfig config.boc
    let in_path = Path::new("src/tests/data/config.boc");
    println!("Config file: {:?}", in_path);
    let root_cell = read_file_de_and_serialise(in_path);
    println!("cell = {:#.2}", root_cell);

    crate::config_params::dump_config(&HashmapE::with_hashmap(32, Some(root_cell)));
}

#[test]
fn test_block_id_ext () {
    let b = BlockIdExt::default();
    let b1 = BlockIdExt::default();

    assert_eq!(b, b1);

    let b = BlockIdExt::with_params(
        ShardIdent::default(), 3784685, UInt256::from([1;32]), UInt256::from([2;32]));

    write_read_and_assert(b);
}

#[test]
fn test_block_id_ext_from_str() {
    let id1: BlockIdExt = 
        "(0:1800000000000000, 1203696, rh 59b6e56610aa5df5e8ee4cc5f1081cd5d08473f10e0899f7763d580b2a635f90, fh 1b4d177339538562d10166d87823783b7e747ee80d85d033459928fd0605a126)"
        .parse().unwrap();

    let id2 = BlockIdExt::with_params(
        ShardIdent::with_tagged_prefix(0, 0x1800000000000000).unwrap(),
        1203696, 
        UInt256::from_str("59b6e56610aa5df5e8ee4cc5f1081cd5d08473f10e0899f7763d580b2a635f90").unwrap(),
        UInt256::from_str("1b4d177339538562d10166d87823783b7e747ee80d85d033459928fd0605a126").unwrap()
    );

    assert_eq!(id1, id2);

    let id1: BlockIdExt = 
        "(-1:8000000000000000, 994703, rh 04da9f61d063d49a5bb4e0c253ed81e1e2a27513e77d630a9aca1e29971fbf4e, fh ba1059b7a17104b4b44742326076e8394f21c1a1dd21fc1b3737d3ca8d779756)"
        .parse().unwrap();

    let id2 = BlockIdExt::with_params(
        ShardIdent::with_tagged_prefix(-1, 0x8000000000000000).unwrap(),
        994703, 
        UInt256::from_str("04da9f61d063d49a5bb4e0c253ed81e1e2a27513e77d630a9aca1e29971fbf4e").unwrap(),
        UInt256::from_str("ba1059b7a17104b4b44742326076e8394f21c1a1dd21fc1b3737d3ca8d779756").unwrap()
    );

    assert_eq!(id1, id2);

    let id3 = BlockIdExt::with_params(
        ShardIdent::with_tagged_prefix(-1, 0x8000000000000000).unwrap(),
        31333, 
        UInt256::from_str("04da9f616763d49a5bb4e0c253ed81e1e2a27513e77d630a9aca1e29971fbf4e").unwrap(),
        UInt256::from_str("ba1059b7a17104b4b44742323336e8394f21c1a1dd21fc1b376643ca8d779756").unwrap()
    );

    assert_eq!(id3, id3.to_string().parse().unwrap());
}


#[test]
fn calc_value_flow() {
    let root_cell = read_file_de_and_serialise(
        Path::new(
            //"src/tests/data/91FDE9DA6661FE9D1FCB013C1079411AFC7BFEDF7FE533C6FD48D25388A3FC26.boc" // master
            "src/tests/data/9C2B3FC5AD455917D374CFADBED8FC2343E31A27C1DF2EB29E84404FA96DE9F8.boc" // old testnet WC
            //"src/tests/data/EC6D799FC7EA14D9FD1D840542514BA774EA3FB5E04B70B6E314C48F95B0C131.boc" // new testnet WC
        ));
    let block = Block::construct_from_cell(root_cell).unwrap();

    let mut new_transaction_fees = Grams::default();

    block
        .read_extra().unwrap()
        .read_account_blocks().unwrap()
        .iterate_objects(|account_block: AccountBlock| {
            new_transaction_fees.add(&account_block.transactions().root_extra().grams)?;
            Ok(true)
        }).unwrap();

    let import_fees = block
        .read_extra().unwrap()
        .read_in_msg_descr().unwrap()
        .root_extra().clone();
    let mut fees_collected = import_fees.fees_collected;
    let value_imported = import_fees.value_imported;

    let exported = block
        .read_extra().unwrap()
        .read_out_msg_descr().unwrap()
        .root_extra().grams;

    fees_collected.add(&new_transaction_fees).unwrap();

    let ethalon_value_flow = block.read_value_flow().unwrap();

    println!("exported       = {:12}", ethalon_value_flow.exported.grams);
    println!("fees_imported  = {:12}", ethalon_value_flow.fees_imported.grams);
    println!("recovered      = {:12}", ethalon_value_flow.recovered.grams);
    println!("created        = {:12}", ethalon_value_flow.created.grams);
    println!("minted         = {:12}", ethalon_value_flow.minted.grams);
    println!("imported       = {:12}", ethalon_value_flow.imported.grams);
    println!("fees_collected = {:12}", ethalon_value_flow.fees_collected.grams);

    let created = Grams::from(1_000_000_000 / 4); // 1G / 4 shards  // 1_700_000_000 for masterchain

    fees_collected.add(&ethalon_value_flow.fees_imported.grams).unwrap(); // TODO calc it
    fees_collected.add(&created).unwrap(); // TODO calc it


    assert_eq!(ethalon_value_flow.exported.grams, exported);
    // assert_eq!(ethalon_value_flow.fees_imported.grams, // TODO
    // assert_eq!(ethalon_value_flow.recovered.grams, // TODO
    assert_eq!(ethalon_value_flow.created.grams, created);
    // assert_eq!(ethalon_value_flow.minted.grams, // TODO
    assert_eq!(ethalon_value_flow.imported, value_imported);
    assert_eq!(ethalon_value_flow.fees_collected.grams, fees_collected);
}

#[test]
fn test_read_tob_block_descr() {
    let data = std::fs::read("src/tests/data/top_block_descr.boc").unwrap();
    let cell = read_single_root_boc(data).unwrap();
    let descr = TopBlockDescr::construct_from_cell(cell).unwrap();
    println!("{:?}", descr);
}

#[test]
fn test_copyleft_rewards() {
    let mut copyleft_rewards = CopyleftRewards::default();
    copyleft_rewards.add_copyleft_reward(&AccountId::from([1; 32]), &100.into()).unwrap();
    copyleft_rewards.add_copyleft_reward(&AccountId::from([2; 32]), &200.into()).unwrap();
    copyleft_rewards.add_copyleft_reward(&AccountId::from([3; 32]), &300.into()).unwrap();
    copyleft_rewards.add_copyleft_reward(&AccountId::from([4; 32]), &400.into()).unwrap();
    copyleft_rewards.add_copyleft_reward(&AccountId::from([5; 32]), &500.into()).unwrap();

    assert_eq!(copyleft_rewards.len().unwrap(), 5);
    let mut index = 1;
    copyleft_rewards.iterate_with_keys(|address: AccountId, value| {
        assert_eq!(address, AccountId::from([index as u8; 32]));
        assert_eq!(value, Grams::from(index * 100));
        index += 1;
        Ok(true)
    }).unwrap();
    assert_eq!(index, 6);

    copyleft_rewards.add_copyleft_reward(&AccountId::from([2; 32]), &300.into()).unwrap();
    copyleft_rewards.add_copyleft_reward(&AccountId::from([2; 32]), &500.into()).unwrap();
    copyleft_rewards.add_copyleft_reward(&AccountId::from([5; 32]), &700.into()).unwrap();

    assert_eq!(copyleft_rewards.len().unwrap(), 5);
    let mut index = 1;
    copyleft_rewards.iterate_with_keys(|address: AccountId, value| {
        if index != 2 && index != 5 {
            assert_eq!(address, AccountId::from([index as u8; 32]));
            assert_eq!(value, Grams::from(index * 100));
        }
        index += 1;
        Ok(true)
    }).unwrap();
    assert_eq!(index, 6);
    assert_eq!(copyleft_rewards.get(&AccountId::from([2; 32])).unwrap().unwrap(), 1000);
    assert_eq!(copyleft_rewards.get(&AccountId::from([5; 32])).unwrap().unwrap(), 1200);
}

#[test]
fn test_copyleft_rewards_merge() {
    let mut copyleft_rewards = CopyleftRewards::default();
    copyleft_rewards.add_copyleft_reward(&AccountId::from([1; 32]), &100.into()).unwrap();
    copyleft_rewards.add_copyleft_reward(&AccountId::from([2; 32]), &200.into()).unwrap();
    copyleft_rewards.add_copyleft_reward(&AccountId::from([3; 32]), &300.into()).unwrap();
    copyleft_rewards.add_copyleft_reward(&AccountId::from([4; 32]), &400.into()).unwrap();
    copyleft_rewards.add_copyleft_reward(&AccountId::from([5; 32]), &500.into()).unwrap();

    let mut copyleft_rewards2 = CopyleftRewards::default();
    copyleft_rewards2.add_copyleft_reward(&AccountId::from([1; 32]), &1000.into()).unwrap();
    copyleft_rewards2.add_copyleft_reward(&AccountId::from([2; 32]), &2000.into()).unwrap();
    copyleft_rewards2.add_copyleft_reward(&AccountId::from([4; 32]), &4000.into()).unwrap();
    copyleft_rewards2.add_copyleft_reward(&AccountId::from([6; 32]), &6000.into()).unwrap();
    copyleft_rewards2.add_copyleft_reward(&AccountId::from([7; 32]), &7000.into()).unwrap();
    copyleft_rewards2.add_copyleft_reward(&AccountId::from([8; 32]), &8000.into()).unwrap();

    copyleft_rewards.merge_rewards(&copyleft_rewards2).unwrap();
    assert_eq!(copyleft_rewards.len().unwrap(), 8);
    let mut index = 1;
    copyleft_rewards.iterate_with_keys(|address: AccountId, value| {
        if index != 3 && index != 5 {
            assert_eq!(address, AccountId::from([index as u8; 32]));
            if index < 5 {
                assert_eq!(value, Grams::from(index * 100 + index * 1000));
            } else {
                assert_eq!(value, Grams::from(index * 1000));
            }
        }
        index += 1;
        Ok(true)
    }).unwrap();
    assert_eq!(index, 9);
    assert_eq!(copyleft_rewards.get(&AccountId::from([3; 32])).unwrap().unwrap(), 300);
    assert_eq!(copyleft_rewards.get(&AccountId::from([5; 32])).unwrap().unwrap(), 500);
}

#[test]
fn test_copyleft_rewards_merge_threshold() {
    let mut copyleft_rewards = CopyleftRewards::default();
    copyleft_rewards.add_copyleft_reward(&AccountId::from([1; 32]), &100.into()).unwrap();
    copyleft_rewards.add_copyleft_reward(&AccountId::from([2; 32]), &200.into()).unwrap();
    copyleft_rewards.add_copyleft_reward(&AccountId::from([3; 32]), &300.into()).unwrap();
    copyleft_rewards.add_copyleft_reward(&AccountId::from([4; 32]), &400.into()).unwrap();
    copyleft_rewards.add_copyleft_reward(&AccountId::from([5; 32]), &5100.into()).unwrap();

    let mut copyleft_rewards2 = CopyleftRewards::default();
    copyleft_rewards2.add_copyleft_reward(&AccountId::from([1; 32]), &1000.into()).unwrap();
    copyleft_rewards2.add_copyleft_reward(&AccountId::from([2; 32]), &2000.into()).unwrap();
    copyleft_rewards2.add_copyleft_reward(&AccountId::from([4; 32]), &4000.into()).unwrap();
    copyleft_rewards2.add_copyleft_reward(&AccountId::from([6; 32]), &6000.into()).unwrap();
    copyleft_rewards2.add_copyleft_reward(&AccountId::from([7; 32]), &7000.into()).unwrap();
    copyleft_rewards2.add_copyleft_reward(&AccountId::from([8; 32]), &8000.into()).unwrap();

    let arr = copyleft_rewards.merge_rewards_with_threshold(&copyleft_rewards2, &5000.into()).unwrap();
    assert_eq!(copyleft_rewards.len().unwrap(), 5);
    assert_eq!(copyleft_rewards.get(&AccountId::from([1; 32])).unwrap().unwrap(), 1100);
    assert_eq!(copyleft_rewards.get(&AccountId::from([2; 32])).unwrap().unwrap(), 2200);
    assert_eq!(copyleft_rewards.get(&AccountId::from([3; 32])).unwrap().unwrap(), 300);
    assert_eq!(copyleft_rewards.get(&AccountId::from([4; 32])).unwrap().unwrap(), 4400);
    assert_eq!(copyleft_rewards.get(&AccountId::from([5; 32])).unwrap().unwrap(), 5100); // because old values don't check

    assert_eq!(arr.len(), 3);
    assert_eq!(arr[0], (AccountId::from([6; 32]), 6000.into()));
    assert_eq!(arr[1], (AccountId::from([7; 32]), 7000.into()));
    assert_eq!(arr[2], (AccountId::from([8; 32]), 8000.into()));
}

#[test]
fn block_info_serde(){
    let block_info = super::BlockInfo{
        version: 0,
        gen_utime: 1684756262u32.into(),
        gen_utime_ms_part: 99,
        ..Default::default()
    };
    let serialized = block_info.serialize().unwrap();
    let deserialized = super::BlockInfo::construct_from(&mut SliceData::load_cell(serialized).unwrap()).unwrap();
    {
        let gen_utime_ms = 1684756262 * 1000 + 99;
        assert_eq!(deserialized.gen_utime_ms(), gen_utime_ms);
    }
    assert_eq!(block_info, deserialized);

}

fn create_test_block(opts: u8) -> Block {
    let mut outmsg_descr = OutMsgDescr::with_serde_opts(opts);
    let trans_data = create_test_transaction_set();
    let (enveloped, mut tr) = if opts & SERDE_OPTS_COMMON_MESSAGE != 0 {
        ( 
            MsgEnvelope::with_common_msg_support(
                &CommonMessage::Std(Message::default()),
                1.into(),
            ).unwrap(),
            Transaction::with_common_msg_support(trans_data.account_id),
        )
    } else {
        (
            MsgEnvelope::with_message_and_fee(
                &Message::default(),
                1.into(),
            ).unwrap(),
            Transaction::with_address_and_status(
                trans_data.account_id, 
                trans_data.orig_status.clone()
            ),
        )
    };
    for ref msg in trans_data.out_msgs {
        tr.add_out_message(msg).unwrap();
    }
    tr.write_in_msg(Some(&trans_data.in_msg)).unwrap();
    tr.set_logical_time(trans_data.lt);
    tr.orig_status = trans_data.orig_status;
    let out_msg = OutMsg::new(
        ChildCell::with_struct_and_opts(
                &enveloped,
                opts,
            ).unwrap(),
        ChildCell::with_struct_and_opts(
            &tr,
            opts,
        ).unwrap()
    );
    outmsg_descr.insert(&out_msg).unwrap();
    let mut block_extra = if opts & SERDE_OPTS_COMMON_MESSAGE != 0 {
        BlockExtra::with_common_msg_support() 
    } else {
        BlockExtra::new()
    };
    block_extra.write_account_blocks(&generate_test_shard_account_block(opts)).unwrap();
    block_extra.write_out_msg_descr(&outmsg_descr).unwrap();
    
    let block_info = BlockInfo::new();
    let value_flow = ValueFlow::default();
    let state_update = MerkleUpdate::default();
    let updates = Some(OutQueueUpdates::new());

    if opts & SERDE_OPTS_COMMON_MESSAGE != 0 {
        Block::with_common_msg_support(
            1,
            &block_info,
            &value_flow,
            &state_update,
            updates,
            &block_extra,
        ).unwrap()
    } else {
        Block::with_out_queue_updates(
            1,
            block_info,
            value_flow,
            state_update,
            updates,
            block_extra,
        ).unwrap()
    }
}

#[test]
fn test_serde_block_options_empty() {
    let mut block = create_test_block(SERDE_OPTS_EMPTY);
    block.out_msg_queue_updates = None;
    let cell = block.serialize().unwrap();
    let block2 = Block::construct_from_cell(cell.clone()).unwrap();
    let block3 = Block::construct_from_cell_with_opts(cell.clone(), SERDE_OPTS_COMMON_MESSAGE).unwrap();
    assert_eq!(block, block2);
    assert_eq!(block2, block3);
    assert!(matches!(block3.serialize_with_opts(SERDE_OPTS_COMMON_MESSAGE), Err(_)));
}

#[test]
fn test_serde_block_options_commonmsg() {
    let block = create_test_block(SERDE_OPTS_COMMON_MESSAGE);
    let cell = block.serialize_with_opts(SERDE_OPTS_COMMON_MESSAGE).unwrap();
    assert!(matches!(block.serialize(), Err(_)));
    let block1 = Block::construct_from_cell_with_opts(cell.clone(), SERDE_OPTS_COMMON_MESSAGE).unwrap();
    let block2 = Block::construct_from_cell(cell.clone()).unwrap();
    assert_eq!(block1, block2);
}

#[test]
fn test_block_with_common_message() -> Result<()> {
    let block = create_test_block(SERDE_OPTS_COMMON_MESSAGE);

    let err = block.serialize().unwrap_err();
    assert!(matches!(err.downcast_ref().unwrap(), &BlockError::MismatchedSerdeOptions(_, _, _)));

    let err = block.serialize_with_opts(SERDE_OPTS_EMPTY).unwrap_err();
    assert!(matches!(err.downcast_ref().unwrap(), &BlockError::MismatchedSerdeOptions(_, _, _)));

    let cell = block.serialize_with_opts(SERDE_OPTS_COMMON_MESSAGE)?;

    let block2 = Block::construct_from_cell_with_opts(cell.clone(), SERDE_OPTS_COMMON_MESSAGE)?;
    let extra = block2.read_extra()?;
    let msg_descr = extra.read_out_msg_descr()?;
    assert_eq!(msg_descr.serde_opts(), SERDE_OPTS_COMMON_MESSAGE);

    let mut msg = None;
    let _ = msg_descr.iterate_objects(|x| {
        let enveloped = x.read_out_message()?.unwrap();
        msg = Some(enveloped.read_common_message()?);
        Ok(true)
    }).unwrap();
    let msg = msg.unwrap();
    assert_eq!(msg.get_std().unwrap(), &Message::default());

    let block3 = Block::construct_from_cell(cell)?;
    assert_eq!(block2, block3);
    Ok(())
}

#[test]
fn test_block_queue_updates_serde() {
    let mut block = create_test_block(SERDE_OPTS_COMMON_MESSAGE);
    block.out_msg_queue_updates = None;
    let cell = block.serialize_with_opts(SERDE_OPTS_COMMON_MESSAGE).unwrap();
    let block2 = Block::construct_from_cell_with_opts(cell, SERDE_OPTS_COMMON_MESSAGE).unwrap();
    assert_eq!(block2.out_msg_queue_updates, None);
}


fn create_block_proof() -> MerkleProof {
    let block_root = read_single_root_boc(std::fs::read("src/tests/data/key_block.boc").unwrap()).unwrap();
    let usage_tree = UsageTree::with_root(block_root.clone());
    let block = Block::construct_from_cell(usage_tree.root_cell()).unwrap();
    block.read_info().unwrap();
    block.read_state_update().unwrap();
    MerkleProof::create_by_usage_tree(&block_root, usage_tree).unwrap()
}

#[test]
fn test_mesh_kit_serde() {
    let mut mesh_kit = MeshKit::default();
    mesh_kit.mc_block_part = create_block_proof();
    mesh_kit.queues = MeshMsgQueuesKit::default();
    mesh_kit.queues.add_queue(&ShardIdent::with_tagged_prefix(0, 0x4000_0000_0000_0000_u64).unwrap(), OutMsgQueueInfo::default()).unwrap();
    mesh_kit.queues.add_queue(&ShardIdent::with_tagged_prefix(0, 0xc000_0000_0000_0000_u64).unwrap(), OutMsgQueueInfo::default()).unwrap();

    let cell = mesh_kit.serialize().unwrap();
    let mesh_kit2 = MeshKit::construct_from_cell(cell).unwrap();
    assert_eq!(mesh_kit, mesh_kit2);
}

#[test]
fn test_mesh_update_serde() {
    let mut mesh_update = MeshUpdate::default();
    mesh_update.mc_block_part = create_block_proof();
    mesh_update.queue_updates = MeshMsgQueueUpdates::default();
    mesh_update.queue_updates.add_queue_update(&ShardIdent::with_tagged_prefix(0, 0x4000_0000_0000_0000_u64).unwrap(), MerkleUpdate::default()).unwrap();
    mesh_update.queue_updates.add_queue_update(&ShardIdent::with_tagged_prefix(0, 0xc000_0000_0000_0000_u64).unwrap(), MerkleUpdate::default()).unwrap();

    mesh_update.queue_updates.get_queue_update(&ShardIdent::with_tagged_prefix(0, 0x4000_0000_0000_0000_u64).unwrap()).unwrap().unwrap();
    mesh_update.queue_updates.get_queue_update(&ShardIdent::with_tagged_prefix(0, 0xc000_0000_0000_0000_u64).unwrap()).unwrap().unwrap();

    let cell = mesh_update.serialize().unwrap();
    let mesh_update2 = MeshUpdate::construct_from_cell(cell).unwrap();
    assert_eq!(mesh_update, mesh_update2);
}
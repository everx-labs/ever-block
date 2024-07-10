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
use crate::{
    ShardIdent, blocks::Block, config_params::ConfigParamEnum, merkle_proof::MerkleProof,
    signature::BlockProof, write_read_and_assert, MASTERCHAIN_ID, BASE_WORKCHAIN_ID, BinTree,
    ShardDescr, InRefValue,
};
use std::collections::HashSet;
use super::*;

#[test]
fn test_validator_info_new_default() {
    let vi = ValidatorInfo::default();
    let vi2 = ValidatorInfo::default();

    assert_eq!(vi, vi2);
    write_read_and_assert(vi);
}

#[test]
fn test_validator_info_new_with() {
    let vi = ValidatorInfo::with_params(1, 2, false);

    assert_ne!(vi, ValidatorInfo::with_params(3, 2, true));
    write_read_and_assert(vi);
}


#[test]
fn test_validator_base_info_new_default() {
    let vi = ValidatorBaseInfo::new();
    let vi2 = ValidatorBaseInfo::default();

    assert_eq!(vi, vi2);
    write_read_and_assert(vi);
}

#[test]
fn test_validator_base_info_new_with() {
    let vi = ValidatorBaseInfo::with_params(1, 2);

    assert_ne!(vi, ValidatorBaseInfo::with_params(3, 2));
    write_read_and_assert(vi);
}


#[test]
fn test_validator_desc_new_default() {
    let vd = ValidatorDescr::new();
    let vd2 = ValidatorDescr::default();

    assert_eq!(vd, vd2);
    write_read_and_assert(vd);
}

#[test]
fn test_validator_desc_info_new_with() {
    let keypair = crate::Ed25519KeyOption::generate().unwrap();
    let key = SigPubKey::from_bytes(keypair.pub_key().unwrap()).unwrap();
    let vd = ValidatorDescr::with_params(key.clone(), 2121212121, None, None);

    assert_ne!(vd, ValidatorDescr::with_params(key, 2, None, None));
    write_read_and_assert(vd);
}

#[test]
fn test_validator_set_serialize(){
    let mut list = vec!();
    for n in 0..20 {
        let keypair = crate::Ed25519KeyOption::generate().unwrap();
        let key = SigPubKey::from_bytes(keypair.pub_key().unwrap()).unwrap();
        let vd = ValidatorDescr::with_params(key, n, None, None);
        list.push(vd);
    }

    let vset = ValidatorSet::new(
        0,
        100, 
        1,
        list
    ).unwrap();

    write_read_and_assert(vset);
}

fn check_block_proof(key_block_file_name: &str, proof_file_name: &str) {
    let key_block = Block::construct_from_file(key_block_file_name).unwrap();
    let proof = BlockProof::construct_from_file(proof_file_name).unwrap();

    let merkle_proof = MerkleProof::construct_from_cell(proof.root.clone()).unwrap();
    let block_virt_root = merkle_proof.proof.virtualize(1);
    let virt_block = Block::construct_from_cell(block_virt_root).unwrap();

    let config = key_block
        .read_extra().unwrap()
        .read_custom().unwrap().unwrap()
        .config().unwrap().clone();

    let cp34 = config.config(34).unwrap().unwrap();
    let cur_validator_set = if let ConfigParamEnum::ConfigParam34(vs) = cp34 { vs.cur_validators } else { unreachable!() };

    let cp28 = config.config(28).unwrap().unwrap();
    let cc_config = if let ConfigParamEnum::ConfigParam28(ccc) = cp28 { ccc } else { unreachable!() };

    let virt_info = virt_block.read_info().unwrap();

    let (validators, hash_short) = cur_validator_set.calc_subset(
            &cc_config, 
            proof.proof_for.shard_id.shard_prefix_with_tag(),
            proof.proof_for.shard_id.workchain_id(),
            proof.signatures.as_ref().map(|s| s.validator_info.catchain_seqno)
                .unwrap_or_else(|| virt_info.gen_catchain_seqno()),
            virt_info.gen_utime()
        )
        .unwrap();

    if let Some(signatures) = proof.signatures.as_ref() {

        assert_eq!(signatures.validator_info.catchain_seqno, virt_info.gen_catchain_seqno());

        assert_eq!(signatures.validator_info.validator_list_hash_short, hash_short);

        let pure_signatures = &signatures.pure_signatures;

        let data = Block::build_data_for_sign(&proof.proof_for.root_hash, &proof.proof_for.file_hash);
        let weight = pure_signatures.check_signatures(&validators, &data).unwrap();
        assert_eq!(weight, pure_signatures.weight());
    } else {
        assert_eq!(virt_info.gen_validator_list_hash_short(), hash_short);
    }
}

#[test]
fn test_calc_mc_subset() {
    check_block_proof(
        "src/tests/data/test_calc_subset/key_block__no_shuffle",
        "src/tests/data/test_calc_subset/proof__no_shuffle"
    );
}

#[test]
fn test_calc_mc_subset_shuffle() {
    check_block_proof(
        "src/tests/data/test_calc_subset/key_block__shuffle",
        "src/tests/data/test_calc_subset/proof__shuffle"
    );
}

#[test]
fn test_calc_shard_subset() {
    check_block_proof(
        "src/tests/data/test_calc_shard_subset/key_block",
        "src/tests/data/test_calc_shard_subset/proof_4377252"
    );
}

#[test]
fn test_isolate_mc_validators() {
    let key_block = Block::construct_from_file("src/tests/data/test_calc_subset/key_block__shuffle").unwrap();
    let config = key_block
        .read_extra().unwrap()
        .read_custom().unwrap().unwrap()
        .config().unwrap().clone();

    let cp34 = config.config(34).unwrap().unwrap();
    let cur_validator_set = if let ConfigParamEnum::ConfigParam34(vs) = cp34 { vs.cur_validators } else { unreachable!() };

    let cp28 = config.config(28).unwrap().unwrap();
    let mut cc_config = if let ConfigParamEnum::ConfigParam28(ccc) = cp28 { ccc } else { unreachable!() };
    cc_config.isolate_mc_validators = true;

    // calc subsets for shardes and check it does not contain main validators

    let (main_validators, _) = cur_validator_set.calc_subset(
        &cc_config, 
        ShardIdent::masterchain().shard_prefix_with_tag(),
        MASTERCHAIN_ID,
        123,
        1619168373.into()
    )
    .unwrap();

    println!("main validators");
    for v in main_validators.iter() {
        println!("{:x}", v.adnl_addr.as_ref().unwrap());
    }

    for shard in 0..16 {
        let shard = ShardIdent::with_tagged_prefix(
            BASE_WORKCHAIN_ID,
            (shard << 60) | (8 << 56),
        ).unwrap();
        let (shard_validators, _) = cur_validator_set.calc_subset(
            &cc_config, 
            shard.shard_prefix_with_tag(),
            BASE_WORKCHAIN_ID,
            123,
            UnixTime32::new(1619168373)
        )
        .unwrap();
        
        println!("shard {} validators", shard);
        for v in shard_validators.iter() {
            println!("{:x}", v.adnl_addr.as_ref().unwrap());
        }

        for sv in shard_validators.iter() {
            for mv in main_validators.iter() {
                assert_ne!(sv.public_key, mv.public_key)
            }
        }
    }
}

#[test]
fn test_validator_shard_stat() {
    // std::env::set_var("RUST_BACKTRACE", "full");
    for len in 0..1024 {
        println!("len = {}", len);
        let mut stat = ValidatorsStat::default();
        for i in 0..len {
            stat.values.push(i as u16);
        }
        write_read_and_assert(stat.clone());
    }
}

#[test]
fn test_fast_finality_roles() {

    // std::env::set_var("RUST_BACKTRACE", "full");

    const VALIDATORS_COUNT: u16 = 20;
    const SHARD_BLOCK_RANGE: u32 = 5_000;

    let descr = ShardDescr {
        seq_no: 1,
        ..Default::default()
    };
    let mut shards_tree = BinTree::with_item(&descr).unwrap();
    shards_tree.split(SliceData::with_bitstring(vec![0], 0), |d| Ok((d.clone(), d.clone()))).unwrap();
    shards_tree.split(SliceData::with_bitstring(vec![0b0000_0000], 1), |d| Ok((d.clone(), d.clone()))).unwrap();
    shards_tree.split(SliceData::with_bitstring(vec![0b1000_0000], 1), |d| Ok((d.clone(), d.clone()))).unwrap();
    shards_tree.split(SliceData::with_bitstring(vec![0b0000_0000], 2), |d| Ok((d.clone(), d.clone()))).unwrap();
    shards_tree.split(SliceData::with_bitstring(vec![0b0100_0000], 2), |d| Ok((d.clone(), d.clone()))).unwrap();
    shards_tree.split(SliceData::with_bitstring(vec![0b1000_0000], 2), |d| Ok((d.clone(), d.clone()))).unwrap();
    shards_tree.split(SliceData::with_bitstring(vec![0b1100_0000], 2), |d| Ok((d.clone(), d.clone()))).unwrap();

    let mut shards = ShardHashes::new();
    shards.set(&0, &InRefValue(shards_tree)).unwrap();

    // fill config
    let ff_config =  FastFinalityConfig::default();

    // fill validators stat
    let mut common_stat = ValidatorsStat::new(VALIDATORS_COUNT);

    let mut idents = vec!();
    shards.iterate_shards(|ident, _| {
        print!("{}  ", ident);
        idents.push(ident);
        Ok(true)
    }).unwrap();
    print!("\n");

    let mut rng = rand::thread_rng();
    let mut total_collator_times = vec!(0; VALIDATORS_COUNT as usize);
    let mut total_msgpool_times = vec!(0; VALIDATORS_COUNT as usize);

    for _mc_seqno in 0..10_000 {

        let mut changed = HashSet::new();
        let mut working_validators = HashSet::new();

        rand::seq::SliceRandom::shuffle(&mut idents[..], &mut rng);
        for ident in idents.iter() {

            let mut descr = shards.get_shard(&ident).unwrap().unwrap().descr;
            let problem = rand::random::<u32>() % 1000 == 0;


            let init = descr.collators.is_none();
            let mut collators = descr.collators.unwrap_or_default();

            //if init { println!("init {ident}"); }
            //if problem { println!("problem in {ident}"); }

            if !problem && !init {

                // import new blocks
                descr.seq_no += rand::random::<u32>() % 32;

                // increase familiarity for current collator and mempool
                let c = collators.current.collator;
                working_validators.insert(c);
                collators.stat.update(c, |f| f + ff_config.familiarity_collator_fine).unwrap();
                for mp in &collators.current.mempool {
                    working_validators.insert(*mp);
                    collators.stat.update(*mp, |f| f + ff_config.familiarity_msgpool_fine).unwrap();
                }
            }
            if init {
                collators.stat = ValidatorsStat::new(VALIDATORS_COUNT);
            } else {
                // increase familiarity for other validators
                for i in 0..VALIDATORS_COUNT {
                    if i != collators.current.collator {
                        collators.stat.update(i as u16, |f| f.saturating_sub(ff_config.familiarity_fading)).unwrap();
                    }
                }
            }


            // update ranges

            if collators.current.finish < descr.seq_no || problem {
                changed.insert(ident);

                // just update current collators, don't process prev and next at all.

                let black_list = if init {
                    vec!()
                } else {
                    vec!(collators.current.collator)
                };

                let new_range = descr.seq_no + 1..descr.seq_no + SHARD_BLOCK_RANGE;
                // let now = std::time::Instant::now();
                let (new_collator, new_mp) = find_validators(
                    ident,
                    new_range.clone(),
                    &shards,
                    &common_stat,
                    &ff_config,
                    &black_list,
                    UInt256::rand().as_slice(),
                ).unwrap();
                // println!("find_validator_for_role time: {}micros", now.elapsed().as_micros());
                total_collator_times[new_collator as usize] += 1;
                collators.current.collator = new_collator;
                collators.current.start = new_range.start;
                collators.current.finish = new_range.end;
                collators.current.mempool.clear();
                for mp in new_mp {
                    collators.current.mempool.push(mp);
                    total_msgpool_times[mp as usize] += 1;
                }
            }

            descr.collators = Some(collators);
            shards.update_shard(&ident, |_| Ok(descr)).unwrap();
        }

        // Update unreliability stat
        for validator in 0..(common_stat.values.len() as u16) {
            let fading = if working_validators.contains(&(validator)) {
                ff_config.unreliability_strong_fading
            } else {
                ff_config.unreliability_weak_fading
            };
            common_stat.update(validator, |unreliability| {
                unreliability.saturating_sub(fading)
            }).unwrap();
        }

        // Print new state
        if !changed.is_empty() {
            shards.iterate_shards(|ident, descr| {
                if changed.contains(&ident) {
                    let collators = descr.collators().unwrap();
                    print!("{:>02} (", collators.current.collator);
                    for mp in &collators.current.mempool {
                        print!("{:>02} ", mp);
                    }
                    print!(")      ");
                } else {
                    print!("                    ");
                }
                Ok(true)
            }).unwrap();
            print!("\n");
        }
    }

    for i in 0..common_stat.values.len() {
        let stat = common_stat.get(i as u16).unwrap();
        println!("{:>2}: unreliability {}", i, stat);
    }

    let mut total_familiarity = vec!(0; VALIDATORS_COUNT as usize);
    shards.iterate_shards(|ident, descr| {
        print!("Shard {} familiarity   ", ident);
        let collators = descr.collators().unwrap();
        for (i, s) in collators.stat.values.iter().enumerate() {
            print!("{i:>2}: {s:<6} ");
            total_familiarity[i] += *s as usize;
        }
        print!("\n");

        Ok(true)
    }).unwrap();

    print!("Total familiarity                      ");
    for (i, s) in total_familiarity.iter().enumerate() {
        print!("{i:>2}: {s:<6} ");
    }

    print!("\nTotal collator times                   ");
    for (i, s) in total_collator_times.iter().enumerate() {
        print!("{i:>2}: {s:<6} ");
    }
    print!("\nTotal msgpool times                    ");
    for (i, s) in total_msgpool_times.iter().enumerate() {
        print!("{i:>2}: {s:<6} ");
    }
    print!("\n");


}

#[test]
fn test_fast_finality_roles_2() -> Result<()> {

    let ff_config = FastFinalityConfig::default();
    let start = 1;
    let finish = 499;
    let mut shards = ShardHashes::default();
    shards.add_workchain(0, 10, UInt256::rand(), UInt256::rand(), None)?;
    let validators_common_stat = ValidatorsStat::new(5);
    let prev_id = UInt256::rand();

    let mut prev_result = None;

    for _ in 0..10 {
        let (collator, mempool) = find_validators(
            &ShardIdent::full(0),
            start..finish+1,
            &shards,
            &validators_common_stat,
            &ff_config,
            &[],
            prev_id.as_slice(),
        )?;

        let result = crate::CollatorRange {
            collator,
            mempool,
            start,
            finish,
        };

        if let Some(prev_result) = prev_result {
            assert_eq!(prev_result, result);
        }

        prev_result = Some(result)
    }

    Ok(())
}
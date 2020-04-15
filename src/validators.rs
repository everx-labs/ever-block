/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.  You may obtain a copy of the
* License at: https://ton.dev/licenses
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use crate::{
    define_HashmapE,
    error::BlockError,
    signature::SigPubKey,
    types::{Number16, UnixTime32},
    Serializable, Deserializable,
    config_params::CatchainConfig,
    shard::{SHARD_FULL, MASTERCHAIN_ID}
};

use std::{
    io::{Write, Cursor},
    cmp::{min, Ordering},
};
use sha2::{Sha256, Sha512, Digest};
use ton_types::types::ByteOrderRead;
use crc::{crc32, Hasher32};
use ton_types::{
    error, fail, Result,
    UInt256,
    BuilderData, Cell, HashmapE, HashmapType, IBitstring, SliceData,
};

/*
validator_info$_
  validator_list_hash_short:uint32 
  catchain_seqno:uint32
  nx_cc_updated:Bool
= ValidatorInfo;
*/

/// Validator info struct
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ValidatorInfo {
    validator_list_hash_short: u32,
    catchain_seqno: u32,
    nx_cc_updated: bool
}

impl ValidatorInfo {
    pub fn new() -> Self {
        ValidatorInfo {
            validator_list_hash_short: 0,
            catchain_seqno: 0,
            nx_cc_updated: false
        }
    }

    pub fn with_params(
        validator_list_hash_short: u32, 
        catchain_seqno: u32, 
        nx_cc_updated: bool) -> Self 
    {
        ValidatorInfo {
            validator_list_hash_short,
            catchain_seqno,
            nx_cc_updated
        }
    }
}


impl Serializable for ValidatorInfo {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.validator_list_hash_short.write_to(cell)?;
        self.catchain_seqno.write_to(cell)?;
        self.nx_cc_updated.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for ValidatorInfo {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.validator_list_hash_short.read_from(cell)?;
        self.catchain_seqno.read_from(cell)?;
        self.nx_cc_updated.read_from(cell)?;
        Ok(())
    }
}


/*
validator_base_info$_
  validator_list_hash_short:uint32 
  catchain_seqno:uint32
= ValidatorBaseInfo;
*/

///
/// ValidatorBaseInfo
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ValidatorBaseInfo {
    pub validator_list_hash_short: u32,
    pub catchain_seqno: u32,
}

impl ValidatorBaseInfo {
    pub fn new() -> Self {
        ValidatorBaseInfo {
            validator_list_hash_short: 0,
            catchain_seqno: 0,
        }
    }

    pub fn with_params(
        validator_list_hash_short: u32, 
        catchain_seqno: u32) -> Self 
    {
        ValidatorBaseInfo {
            validator_list_hash_short,
            catchain_seqno,
        }
    }
}


impl Serializable for ValidatorBaseInfo {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.validator_list_hash_short.write_to(cell)?;
        self.catchain_seqno.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for ValidatorBaseInfo {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.validator_list_hash_short.read_from(cell)?;
        self.catchain_seqno.read_from(cell)?;
        Ok(())
    }
}


/*
validator#53 
    public_key:SigPubKey 
    weight:uint64 
= ValidatorDescr;
*/

///
/// ValidatorDescr
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ValidatorDescr {
    pub public_key: SigPubKey, 
    pub weight: u64,
    pub adnl_addr: Option<UInt256>,

    // Total weight of the previous validators in the list.
    // The field is not serialized.
    pub prev_weight_sum: u64,
}

impl ValidatorDescr {
    pub fn new() -> Self {
        ValidatorDescr {
            public_key: SigPubKey::default(),
            weight: 0,
            adnl_addr: None,
            prev_weight_sum: 0
        }
    }

    pub fn with_params(
        public_key: SigPubKey, 
        weight: u64,
        adnl_addr: Option<UInt256>) -> Self 
    {
        ValidatorDescr {
            public_key,
            weight,
            adnl_addr,
            prev_weight_sum: 0,
        }
    }

    pub fn compute_node_id_short(&self) -> UInt256 {
        let mut hasher = Sha256::new();
        let magic = [0xc6, 0xb4, 0x13, 0x48]; // magic 0x4813b4c6 from original node's code
        hasher.input(&magic);
        hasher.input(self.public_key.key_bytes());
        UInt256::from(hasher.result().as_slice())
    }
}

const VALIDATOR_DESC_TAG: u8 = 0x53;
const VALIDATOR_DESC_ADDR_TAG: u8 = 0x73;


impl Serializable for ValidatorDescr {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        let tag = if self.adnl_addr.is_some() {VALIDATOR_DESC_ADDR_TAG} else {VALIDATOR_DESC_TAG};
        cell.append_u8(tag)?;
        self.public_key.write_to(cell)?;
        self.weight.write_to(cell)?;
        if let Some(adnl_addr) = self.adnl_addr.as_ref() {
            adnl_addr.write_to(cell)?;
        }
        Ok(())
    }
}

impl Deserializable for ValidatorDescr {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag = cell.get_next_byte()?;
        if tag != VALIDATOR_DESC_TAG && tag != VALIDATOR_DESC_ADDR_TAG {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: "ValidatorDescr".to_string()
                }
            )
        }
        self.public_key.read_from(cell)?;
        self.weight.read_from(cell)?;
        if tag == VALIDATOR_DESC_ADDR_TAG {
            let mut adnl_addr = UInt256::default();
            adnl_addr.read_from(cell)?;
            self.adnl_addr = Some(adnl_addr);
        }
        Ok(())
    }
}

/*
validators#11 
    utime_since:uint32 
    utime_until:uint32 
    total:(## 16) 
    main:(## 16) 
    { main <= total } 
    { main >= 1 } 
    list:(Hashmap 16 ValidatorDescr) 
= ValidatorSet;

validators_ext#12 
    utime_since:uint32 
    utime_until:uint32 
    total:(## 16) 
    main:(## 16) 
    { main <= total } 
    { main >= 1 } 
    total_weight:uint64 
    list:(HashmapE 16 ValidatorDescr) 
= ValidatorSet;
*/

define_HashmapE!{ValidatorDescriptions, 16, ValidatorDescr}

///
/// ValidatorSet
/// 
#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct ValidatorSet {
    utime_since: u32,
    utime_until: u32, 
    total: Number16, 
    main: Number16,
    total_weight: u64,
    list: Vec<ValidatorDescr>, //ValidatorDescriptions,
}

#[derive(Eq, PartialEq, PartialOrd, Debug)]
struct IncludedValidatorWeight {
    pub prev_weight_sum: u64,
    pub weight: u64,
}

impl Ord for IncludedValidatorWeight {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.prev_weight_sum.cmp(&other.prev_weight_sum) {
            Ordering::Equal => {
                self.weight.cmp(&other.weight)
            },
            other => other
        }
    }
}

impl ValidatorSet {
    pub fn new(
        utime_since: u32,
        utime_until: u32, 
        main: u16,
        mut list: Vec<ValidatorDescr>
    ) -> Result<Self> {
        if list.is_empty() {
            failure::bail!(BlockError::InvalidArg("`list` can't be empty".to_string()))
        }
        let mut total_weight = 0;
        for i in 0..list.len() {
            list[i].prev_weight_sum = total_weight;
            total_weight = total_weight.checked_add(list[i].weight).ok_or(
                BlockError::InvalidData(format!("Validator's total weight is more than 2^64"))
            )?;
        }
        Ok(ValidatorSet {
            utime_since,
            utime_until, 
            total: Number16(list.len() as u32), 
            main: Number16(main as u32), 
            total_weight,
            list: list, 
        })
    }

    pub fn utime_since(&self) -> u32 {
        self.utime_since
    }

    pub fn utime_until(&self) -> u32 {
        self.utime_until
    }

    pub fn total(&self) -> u16 {
        self.total.0 as u16
    }

    pub fn main(&self) -> u16 {
        self.main.0 as u16
    }

    pub fn total_weight(&self) -> u64 {
        self.total_weight
    }

    pub fn list(&self) -> &Vec<ValidatorDescr> {
        &self.list
    }

    pub fn at_weight(&self, weight_pos: u64) -> &ValidatorDescr {
        debug_assert!(weight_pos < self.total_weight);
        debug_assert!(self.list.len() > 0);
        for i in 0..self.list.len() {
            if self.list[i].prev_weight_sum > weight_pos {
                debug_assert!(i != 0);
                return &self.list[i - 1];
            }
        }
        self.list.last().unwrap()
    }

    pub fn calc_subset(
        &self, 
        cc_config: &CatchainConfig, 
        shard_pfx: u64, 
        workchain_id: i32, 
        cc_seqno: u32,
        _time: UnixTime32
    ) -> Result<(Vec<ValidatorDescr>, u32)> {
        let is_master = (shard_pfx == SHARD_FULL) && (workchain_id == MASTERCHAIN_ID);

        let count = min(
            self.total.0,
            if is_master { self.main.0 } else { cc_config.shard_validators_num }
        ) as usize;

        let subset = if is_master {
            if !cc_config.shuffle_mc_validators {
                self.list[0..count].to_vec()
            } else {
                // shuffle mc validators from the head of the list
                let mut prng = ValidatorSetPRNG::new(shard_pfx, workchain_id, cc_seqno);
                let mut indexes = vec![0; count];
                for i in 0..count {
                    let j = prng.next_ranged(i as u64 + 1) as usize; // number 0 .. i
                    debug_assert!(j <= i);
                    indexes[i] = indexes[j];
                    indexes[j] = i;
                }
                let mut subset = Vec::with_capacity(count);
                for i in 0..count {
                    subset.push(self.list()[indexes[i]].clone());
                }
                subset
            }
        } else {
            let mut subset = Vec::with_capacity(count);
            let mut prng = ValidatorSetPRNG::new(shard_pfx, workchain_id, cc_seqno);
            let mut weights = Vec::<IncludedValidatorWeight>::with_capacity(count);
            let mut weight_remainder = self.total_weight;
            for _ in 0..count {
                debug_assert!(weight_remainder > 0);
                // 1. take pseudo random weight less (or equal) than weight_remainder
                let mut p = prng.next_ranged(weight_remainder);

                // 2. find p which
                //      >= start p value
                //      >= prev_weight_sum of some number of first validators
                for vw in weights.iter() {
                    if p < vw.prev_weight_sum {
                        break;
                    }
                    p += vw.weight;
                }

                // 3. take validator with less weight greater than p
                let next_validator = self.at_weight(p);

                subset.push(ValidatorDescr::with_params(
                    next_validator.public_key.clone(),
                    1, // NB: shardchain validator lists have all weights = 1
                    next_validator.adnl_addr.clone()));
                debug_assert!(weight_remainder >= next_validator.weight);
                weight_remainder -= next_validator.weight;

                // 4. put validator's weight into sorted list of previous weights
                let new_weight = IncludedValidatorWeight {
                    prev_weight_sum: next_validator.prev_weight_sum,
                    weight: next_validator.weight
                };
                let mut idx = 0;
                while idx < weights.len() {
                    if weights[idx] > new_weight {
                        break;
                    }
                    idx += 1;
                }
                debug_assert!(idx == 0 || weights[idx - 1] < new_weight);
                weights.insert(idx, new_weight);
            }
            subset
        };

        let hash_short = Self::calc_subset_hash_short(&subset, cc_seqno)?;

        Ok((subset, hash_short))
    }

    const HASH_SHORT_MAGIC: i32 = -1877581587;

    pub fn calc_subset_hash_short(subset: &Vec<ValidatorDescr>, cc_seqno: u32) -> Result<u32> {
        let mut hasher = crc32::Digest::new(crc32::CASTAGNOLI);
        hasher.write(&Self::HASH_SHORT_MAGIC.to_le_bytes());
        hasher.write(&cc_seqno.to_le_bytes());
        hasher.write(&(subset.len() as u32).to_le_bytes());
        for vd in subset.iter() {
            hasher.write(vd.public_key.key_bytes());
            hasher.write(&vd.weight.to_le_bytes());
            let addr = vd.adnl_addr.as_ref().ok_or(
                BlockError::InvalidData("At least one validator doesn't have address".to_string())
            )?;
            hasher.write(addr.as_slice());
        }
        Ok(hasher.sum32())
    }
}

const VALIDATOR_SET_TAG: u8 = 0x11;
const VALIDATOR_SET_EX_TAG: u8 = 0x12;

impl Serializable for ValidatorSet {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u8(VALIDATOR_SET_EX_TAG)?;
        self.utime_since.write_to(cell)?;
        self.utime_until.write_to(cell)?;
        self.total.write_to(cell)?;
        self.main.write_to(cell)?;

        let mut validators = ValidatorDescriptions::default();
        for (i, v) in self.list.iter().enumerate() {
            validators.set(&(i as u16), v).unwrap();
        }
        self.total_weight.write_to(cell)?;
        validators.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for ValidatorSet {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag = cell.get_next_byte()?;
        if tag != VALIDATOR_SET_TAG && tag != VALIDATOR_SET_EX_TAG {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: "ValidatorSet".to_string()
                }
            )
        }
        self.utime_since.read_from(cell)?;
        self.utime_until.read_from(cell)?;
        self.total.read_from(cell)?;
        self.main.read_from(cell)?;
        let mut validators = ValidatorDescriptions::default();
        if tag == VALIDATOR_SET_TAG {
            validators.read_hashmap_root(cell)?; // Hashmap
        } else {
            self.total_weight = u64::construct_from(cell)?;
            validators.read_from(cell)?; // HashmapE
        }
        self.list.clear();
        let mut total_weight = 0;
        for i in 0..self.total.0 {
            let mut val = validators.get(&(i as u16))?.ok_or(
                BlockError::InvalidData(format!("Validator's hash map doesn't contain validator with index {}", i))
            )?;
            val.prev_weight_sum = total_weight;
            total_weight += val.weight;
            self.list.push(val);
        }
        if self.list.is_empty() {
            failure::bail!(BlockError::InvalidData("list can't be empty".to_string()));
        }
        if tag == VALIDATOR_SET_TAG {
            self.total_weight = self.list.iter().map(|vd| vd.weight).sum();
        } else {
            if self.total_weight != total_weight {
                failure::bail!(
                    BlockError::InvalidData("Calculated total_weight is not equal to the read one while read ValidatorSet".to_string())
                )
            }
        }

        if self.main > self.total {
            fail!(BlockError::InvalidData("main > total while read ValidatorSet".to_string()))
        }
        if self.main < Number16(1) {
            fail!(BlockError::InvalidData("main < 1 while read ValidatorSet".to_string()))
        }
        Ok(())
    }
}

pub struct ValidatorSetPRNG {
    context: [u8; 48],
    bag: [u64; 7],
    cursor: usize,
}

impl ValidatorSetPRNG {
    pub fn new(shard_pfx: u64, workchain_id: i32, cc_seqno: u32) -> Self {
        let seed = [0; 32];
        Self::with_seed(shard_pfx, workchain_id, cc_seqno, &seed)
    }

    pub fn with_seed(shard_pfx: u64, workchain_id: i32, cc_seqno: u32, seed: &[u8; 32]) -> Self {

        // Big endian
        // byte seed[32]
        // u64 shard;
        // i32 workchain;
        // u32 cc_seqno;
        let mut context = [0_u8; 48];
        let mut cur = Cursor::new(&mut context[..]);
        cur.write(seed).unwrap();
        cur.write(&shard_pfx.to_be_bytes()).unwrap();
        cur.write(&workchain_id.to_be_bytes()).unwrap();
        cur.write(&cc_seqno.to_be_bytes()).unwrap();

        ValidatorSetPRNG{
            context,
            bag: [0_u64; 7],
            cursor: 7,
        }
    }

    fn reset(&mut self) -> u64 {
        // calc hash
        let mut hasher = Sha512::new();
        hasher.input(&self.context[..]);
        let mut hash = Cursor::new(hasher.result());

        // increment seed
        for i in (0..32).rev() {
            self.context[i] += 1;
            if self.context[i] != 0 {
                break;
            }
        }

        // read results
        let first_u64 = hash.read_be_u64().unwrap();
        for i in 0..7 {
            self.bag[i] = hash.read_be_u64().unwrap();
        }

        self.cursor = 0;
        first_u64
    }

    pub fn next_u64(&mut self) -> u64 {
        if self.cursor < self.bag.len() {
            let next = self.bag[self.cursor];
            self.cursor += 1;
            next
        } else {
            self.reset()
        }
    }

    pub fn next_ranged(&mut self, range: u64) -> u64 {
        let val = self.next_u64();
        ((range as u128 * val as u128) >> 64) as u64
    }
}

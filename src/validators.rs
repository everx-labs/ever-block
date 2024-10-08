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
    bls::BLS_PUBLIC_KEY_LEN,
    config_params::CatchainConfig,
    define_HashmapE, error,
    error::BlockError,
    fail, sha512_digest,
    shard::{MASTERCHAIN_ID, SHARD_FULL},
    signature::{CryptoSignature, SigPubKey},
    types::{Number16, UnixTime32},
    BuilderData, ByteOrderRead, Cell, Crc32, Deserializable, FastFinalityConfig, IBitstring, KeyId,
    Result, Serializable, ShardDescr, ShardHashes, ShardIdent, SliceData, UInt256,
    MAX_DATA_BITS, MEMPOOL_MAX_LEN,
};

use std::{
    borrow::Cow,
    cmp::{min, Ordering},
    collections::{HashMap, HashSet},
    convert::TryInto,
    io::{Cursor, Write},
    ops::Range,
    sync::Arc,
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
    pub validator_list_hash_short: u32,
    pub catchain_seqno: u32,
    pub nx_cc_updated: bool
}

impl ValidatorInfo {
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
    pub fn new() -> Self { Self::default() }

    pub fn with_params(
        validator_list_hash_short: u32, 
        catchain_seqno: u32
    ) -> Self {
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
validator#73 
    public_key:SigPubKey 
    weight:uint64 
    adnl_addr:bits256
= ValidatorDescr;
validator#93 
    public_key:SigPubKey 
    weight:uint64 
    adnl_addr:bits256
    mc_seq_no_since:u32
= ValidatorDescr;
*/

///
/// ValidatorDescr
/// Has two keys: public_key and adnl_addr
/// Before first election adnl_addr is None and it is calculated from the public key
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ValidatorDescr {
    pub weight: u64,
    pub public_key: SigPubKey,
    /// before first election this filed is None
    pub adnl_addr: Option<UInt256>,
    pub mc_seq_no_since: u32,
    pub bls_public_key: Option<[u8; BLS_PUBLIC_KEY_LEN]>,

    // Total weight of the previous validators in the list.
    // The field is not serialized.
    pub prev_weight_sum: u64,
}

#[allow(clippy::derived_hash_with_manual_eq)]
impl std::hash::Hash for ValidatorDescr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.public_key.as_slice().hash(state);
        if let Some(aa) = &self.adnl_addr {
            aa.hash(state)
        }
    }
}

impl ValidatorDescr {
    pub fn new() -> Self { Self::default() }

    pub const fn with_params(
        public_key: SigPubKey,
        weight: u64,
        adnl_addr: Option<UInt256>, 
        bls_public_key: Option<[u8; BLS_PUBLIC_KEY_LEN]>) -> Self
    {
        ValidatorDescr {
            public_key,
            weight,
            adnl_addr,
            prev_weight_sum: 0,
            mc_seq_no_since: 0,
            bls_public_key,
        }
    }

    pub fn compute_node_id_short(&self) -> UInt256 {
        self.public_key.pub_key().id().data().into()
    }

    pub fn verify_signature(&self, data: &[u8], signature: &CryptoSignature) -> bool {
        self.public_key.verify_signature(data, signature)
    }

    /// returns adnl_addr or calc it from the public key
    pub fn adnl_addr(&self) -> Arc<KeyId> {
        match &self.adnl_addr {
            Some(addr) => KeyId::from_data(*addr.as_array()),
            None => self.public_key.pub_key().id().clone()
        }
    }
}

const VALIDATOR_DESC_TAG: u8 = 0x53;
const VALIDATOR_DESC_ADDR_TAG: u8 = 0x73;
const VALIDATOR_DESC_ADDR_SEQNO_TAG: u8 = 0x93;
const VALIDATOR_DESC_BLS_KEY_TAG: u8 = 0x74;

impl Serializable for ValidatorDescr {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        let tag = if self.mc_seq_no_since != 0 {
            if self.adnl_addr.is_none() {
                fail!("if mc_seq_no_since is not zero ADNL address must be specified too")
            }
            VALIDATOR_DESC_ADDR_SEQNO_TAG
        } else if self.bls_public_key.is_some() {
            VALIDATOR_DESC_BLS_KEY_TAG
        } else if self.adnl_addr.is_some() {
            VALIDATOR_DESC_ADDR_TAG
        } else {
            VALIDATOR_DESC_TAG
        };
        cell.append_u8(tag)?;
        self.public_key.write_to(cell)?;
        self.weight.write_to(cell)?;
        if let Some(adnl_addr) = self.adnl_addr.as_ref() {
            adnl_addr.write_to(cell)?;
        } else if self.bls_public_key.is_some() {
            UInt256::default().write_to(cell)?;
        }
        if self.mc_seq_no_since != 0 {
            self.mc_seq_no_since.write_to(cell)?;
        }
        if let Some(bls_key) = self.bls_public_key.as_ref() {
            cell.append_raw(bls_key, BLS_PUBLIC_KEY_LEN * 8)?;
        }
        Ok(())
    }
}

impl Deserializable for ValidatorDescr {
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        let tag = slice.get_next_byte()?;
        let (public_key, weight, adnl_addr, mc_seq_no_since, bls_public_key);
        match tag {
            VALIDATOR_DESC_TAG => {
                public_key = Deserializable::construct_from(slice)?;
                weight = Deserializable::construct_from(slice)?;
                adnl_addr = None;
                mc_seq_no_since = 0;
                bls_public_key = None;
            }
            VALIDATOR_DESC_ADDR_TAG => {
                public_key = Deserializable::construct_from(slice)?;
                weight = Deserializable::construct_from(slice)?;
                adnl_addr = Some(Deserializable::construct_from(slice)?);
                mc_seq_no_since = 0;
                bls_public_key = None;
            }
            VALIDATOR_DESC_ADDR_SEQNO_TAG => {
                public_key = Deserializable::construct_from(slice)?;
                weight = Deserializable::construct_from(slice)?;
                adnl_addr = Some(Deserializable::construct_from(slice)?);
                mc_seq_no_since = Deserializable::construct_from(slice)?;
                bls_public_key = None;
            }
            VALIDATOR_DESC_BLS_KEY_TAG => {
                log::error!("{}) rembits={}", line!(), slice.remaining_bits());
                public_key = Deserializable::construct_from(slice)?;
                log::error!("{}) rembits={}", line!(), slice.remaining_bits());
                weight = Deserializable::construct_from(slice)?;
                log::error!("{}) rembits={}", line!(), slice.remaining_bits());
                let addr : UInt256 = Deserializable::construct_from(slice)?;
                adnl_addr = if addr.is_zero() { None } else { Some(addr) };
                log::error!("{}) rembits={}", line!(), slice.remaining_bits());
                mc_seq_no_since = 0;
                bls_public_key = Some(slice.get_next_bits(BLS_PUBLIC_KEY_LEN * 8)?.as_slice().try_into()?);
                log::error!("{}) rembits={}", line!(), slice.remaining_bits());
            }
            tag => fail!(Self::invalid_tag(tag as u32))
        }
        Ok(Self {
            public_key,
            weight,
            adnl_addr,
            mc_seq_no_since,
            bls_public_key,
            prev_weight_sum: 0,
        })
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
    cc_seqno: u32, // is never used
    list: Vec<ValidatorDescr>, //ValidatorDescriptions,
}

#[derive(Eq, PartialEq, Debug)]
struct IncludedValidatorWeight {
    pub prev_weight_sum: u64,
    pub weight: u64,
}

impl PartialOrd for IncludedValidatorWeight {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for IncludedValidatorWeight {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.prev_weight_sum.cmp(&other.prev_weight_sum) {
            Ordering::Equal => {
                self.weight.cmp(&other.weight)
            }
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
            fail!(BlockError::InvalidArg("`list` can't be empty".to_string()))
        }
        let mut total_weight = 0;
        for descr in &mut list {
            descr.prev_weight_sum = total_weight;
            total_weight = total_weight.checked_add(descr.weight).ok_or_else(|| 
                BlockError::InvalidData("Validator's total weight is more than 2^64".to_string())
            )?;
        }
        Ok(ValidatorSet {
            utime_since,
            utime_until, 
            total: Number16::from(list.len() as u16),
            main: Number16::from(main),
            total_weight,
            cc_seqno: 0,
            list,
        })
    }

    pub fn with_cc_seqno(
        utime_since: u32,
        utime_until: u32, 
        main: u16,
        cc_seqno: u32,
        list: Vec<ValidatorDescr>
    ) -> Result<Self> {
        Ok(Self {
            cc_seqno,
            ..Self::new(utime_since, utime_until, main, list)?
        })
    }

    pub fn with_values_version_2(
        utime_since: u32,
        utime_until: u32,
        main: u16,
        total_weight: u64,
        list: Vec<ValidatorDescr>
    ) -> Result<Self> {
        Ok(Self {
            total_weight,
            ..Self::new(utime_since, utime_until, main, list)?
        })
    }

    pub fn utime_since(&self) -> u32 {
        self.utime_since
    }

    pub fn utime_until(&self) -> u32 {
        self.utime_until
    }

    pub fn total(&self) -> u16 {
        self.total.as_u16()
    }

    pub fn main(&self) -> u16 {
        self.main.as_u16()
    }

    pub fn total_weight(&self) -> u64 {
        self.total_weight
    }

    pub fn list(&self) -> &[ValidatorDescr] {
        &self.list
    }

    pub fn validator_by_pub_key(&self, pub_key: &[u8; 32]) -> Option<&ValidatorDescr> {
        self.list.iter().find(|item| item.public_key.as_slice() == pub_key)
    }

    pub fn catchain_seqno(&self) -> u32 {
        self.cc_seqno
    }

    pub fn set_catchain_seqno(&mut self, cc_seqno: u32) {
        self.cc_seqno = cc_seqno;
    }

    pub fn cc_seqno(&self) -> u32 {
        self.cc_seqno
    }

    pub fn set_cc_seqno(&mut self, cc_seqno: u32) {
        self.cc_seqno = cc_seqno;
    }

    pub fn at_weight(&self, weight_pos: u64) -> &ValidatorDescr {
        debug_assert!(weight_pos < self.total_weight);
        debug_assert!(!self.list.is_empty());
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

        let subset = if is_master {
            let count = min(self.total.as_usize(), self.main.as_usize());
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
                for index in indexes.iter().take(count) {
                    subset.push(self.list()[*index].clone());
                }
                subset
            }
        } else {
            let mut prng = ValidatorSetPRNG::new(shard_pfx, workchain_id, cc_seqno);
            let full_list = if cc_config.isolate_mc_validators {
                if self.total <= self.main && !(self.main == 0 && self.total == 0) {
                    fail!("Count of validators is too small to make sharde's subset while `isolate_mc_validators` flag is set (total={}, main={})", self.total, self.main)
                }
                let list = self.list[self.main.as_usize()..].to_vec();
                Cow::Owned(
                    Self::new(self.utime_since, self.utime_until, self.main.as_u16(), list)?
                )

            } else {
                Cow::Borrowed(self)
            };
            let count = min(full_list.total(), cc_config.shard_validators_num as u16) as usize;
            let mut subset = Vec::with_capacity(count);
            let mut weights = Vec::<IncludedValidatorWeight>::with_capacity(count);
            let mut weight_remainder = full_list.total_weight();

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
                let next_validator = full_list.at_weight(p);

                subset.push(ValidatorDescr::with_params(
                    next_validator.public_key.clone(),
                    1, // NB: shardchain validator lists have all weights = 1
                    next_validator.adnl_addr.clone(),
                    next_validator.bls_public_key,
                ));
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

        let hash_short = Self::calc_subset_hash_short(subset.as_slice(), cc_seqno)?;

        Ok((subset, hash_short))
    }

    const HASH_SHORT_MAGIC: u32 = 0x901660ED;

    pub fn calc_subset_hash_short(subset: &[ValidatorDescr], cc_seqno: u32) -> Result<u32> {
        let mut hasher = Crc32::new();
        hasher.update(Self::HASH_SHORT_MAGIC.to_le_bytes());
        hasher.update(cc_seqno.to_le_bytes());
        hasher.update((subset.len() as u32).to_le_bytes());
        for vd in subset.iter() {
            hasher.update(vd.public_key.as_slice());
            hasher.update(vd.weight.to_le_bytes());
            if let Some(addr) = vd.adnl_addr.as_ref() {
                hasher.update(addr.as_slice());
            } else {
                hasher.update(UInt256::default().as_slice());
            }
        }
        Ok(hasher.finalize())
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
            validators.set(&(i as u16), v)?;
        }
        self.total_weight.write_to(cell)?;
        validators.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for ValidatorSet {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag = cell.get_next_byte()?;
        if !matches!(tag, VALIDATOR_SET_TAG | VALIDATOR_SET_EX_TAG) {
            fail!(Self::invalid_tag(tag as u32))
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
        for i in 0..self.total.as_u16() {
            let mut val = validators.get(&i)?.ok_or_else(|| 
                BlockError::InvalidData(format!("Validator's hash map doesn't \
                    contain validator with index {}", i)))?;
            val.prev_weight_sum = total_weight;
            total_weight += val.weight;
            self.list.push(val);
        }
        if self.list.is_empty() {
            fail!(BlockError::InvalidData("list can't be empty".to_string()));
        }
        if tag == VALIDATOR_SET_TAG {
            self.total_weight = self.list.iter().map(|vd| vd.weight).sum();
        } else if self.total_weight != total_weight {
            fail!(BlockError::InvalidData("Calculated total_weight is not equal to the read one while read ValidatorSet".to_string()))
        }

        if self.main > self.total {
            fail!(BlockError::InvalidData("main > total while read ValidatorSet".to_string()))
        }
        if self.main < Number16::new(1)? {
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
        cur.write_all(seed).unwrap();
        cur.write_all(&shard_pfx.to_be_bytes()).unwrap();
        cur.write_all(&workchain_id.to_be_bytes()).unwrap();
        cur.write_all(&cc_seqno.to_be_bytes()).unwrap();

        ValidatorSetPRNG{
            context,
            bag: [0_u64; 7],
            cursor: 7,
        }
    }

    fn reset(&mut self) -> u64 {
        // calc hash
        let mut hash = Cursor::new(sha512_digest(self.context));

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

const VALIDATORS_SHARD_STAT_TAG: u8 = 0x1; // 4 bits
const VALIDATORS_STAT_EXPECTED_MAX: usize = 256;

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ValidatorsStat { 
    // Single u16 value for each validator.
    // VALIDATORS_STAT_EXPECTED_MAX values are stored inplace, 
    // if more - SmallVec just reallocates memory using heap.
    values: smallvec::SmallVec::<[u16; VALIDATORS_STAT_EXPECTED_MAX]>
}

impl ValidatorsStat {
    
    pub fn new(validators_count: u16) -> Self {
        ValidatorsStat { values: smallvec::smallvec![0; validators_count as usize] }
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn update<F>(&mut self, validator_index: u16, updater: F) -> Result<()>
    where F: FnOnce(u16) -> u16 {
        if self.values.len() <= validator_index as usize {
            fail!("Invalid validator index: {} (max is {})", validator_index, self.values.len() - 1)
        }
        self.values[validator_index as usize] = updater(self.values[validator_index as usize]);
        Ok(())
    }

    pub fn get(&self, validator_index: u16) -> Result<u16> {
        if self.values.is_empty() {
            fail!("ValidatorsStat is empty")
        }
        self.values.get(validator_index as usize).copied().ok_or_else(|| 
            error!("Invalid validator index: {} (max is {})", validator_index, self.values.len() - 1)
        )
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
}

impl Serializable for ValidatorsStat {
    fn write_to(&self, builder: &mut BuilderData) -> Result<()> {
        builder.append_bits(VALIDATORS_SHARD_STAT_TAG as usize, 4)?;

        // Items are wrote one by one into cell. If the cell is full, 
        // the rest of the items are wrote into the child cell, etc.

        let mut remaining_len = self.values.len();
        if remaining_len == 0 {
            return Ok(());
        }
        let mut stack = vec![builder.clone()];
        loop {
            // Calculate how many items can be written to the current builder
            let builder_fits = stack.last()
                .ok_or_else(|| error!("INTERNAL ERROR: stack is empty"))?
                .bits_free() / 16;

            // If the current builder can fit all remaining items - finish
            if builder_fits >= remaining_len {
                break;
            } else {
                // If not - create one more builder and push it to the stack
                remaining_len = remaining_len.saturating_sub(builder_fits);
                stack.push(BuilderData::new());
            }
        }

        // Write items to the builders from the last (deeper) cell to the first.
        let mut start = self.values.len().saturating_sub(remaining_len);
        while let Some(mut last_builder) = stack.pop() {
            // Fill a builder
            let builder_fits = last_builder.bits_free() / 16;
            for i in start..min(start + builder_fits, self.values.len()) {
                last_builder.append_u16(self.values[i])?;
            }

            // Move start to the start of the next builder (minus one cell of items)
            start = start.saturating_sub(MAX_DATA_BITS / 16);

            if let Some(prev_builder) = stack.last_mut() {
                prev_builder.checked_append_reference(last_builder.into_cell()?)?;
            } else {
                *builder = last_builder;
            }
        }

        Ok(())
    }
}

impl Deserializable for ValidatorsStat {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        let tag = slice.get_next_int(4)? as u8;
        if tag != VALIDATORS_SHARD_STAT_TAG {
            fail!("Invalid tag for ValidatorsShardStat: {}", tag)
        }
        self.values.clear();
        while slice.remaining_bits() > 0 {
            self.values.push(u16::construct_from(slice)?);
        }
        let mut next_cell_opt = slice.checked_drain_reference().ok();
        while let Some(next_cell) = next_cell_opt {
            let mut slice = SliceData::load_cell(next_cell)?;
            while slice.remaining_bits() > 0 {
                self.values.push(u16::construct_from(&mut slice)?);
            }
            next_cell_opt = slice.checked_drain_reference().ok();
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FastFinalityRole {
    Collator,
    MessageProducer,
}

type MempoolSmallVec = smallvec::SmallVec<[u16; MEMPOOL_MAX_LEN]>;

pub fn find_validators(
    shard: &ShardIdent,
    block_range: Range<u32>,
    shards: &ShardHashes,
    common_stat: &ValidatorsStat,
    config: &FastFinalityConfig,
    black_list: &[u16],
    salt: &[u8],
    mempool: Option<&[u16]>, // Mempool nodes to partial change, None for full change
) -> Result<(u16, MempoolSmallVec)> {

    fn intersection(range1: &Range<u32>, range2: &Range<u32>) -> u32 {
        if range1.start >= range2.end || range1.end <= range2.start {
            0
        } else {
            range1.end.min(range2.end).saturating_sub(range1.start.max(range2.start))
        }
    }

    let calc_busyness_fine_for_shard = |shard: &ShardDescr, desired_range: &Range<u32>, validator: u16| -> u32 {
        let mut fine = 0u32;
        if let Some(collators) = shard.collators.as_ref() {
            let ranges = [Some(&collators.current), Some(&collators.next), collators.next2.as_ref()];
            for range in ranges.into_iter().flatten() { 
                let relative_range = range.start.saturating_sub(shard.seq_no)..
                    range.finish.saturating_sub(shard.seq_no);
                if validator == range.collator {
                    fine += intersection(desired_range, &relative_range) * config.busyness_collator_fine as u32;
                }
                for i in &range.mempool {
                    if validator == *i {
                        fine += intersection(desired_range, &relative_range) * config.busyness_msgpool_fine as u32;
                        break;
                    }
                }
            }
        }
        fine
    };

    // Instead of deserializing ShardDescr many times we will cache them
    let mut shards_cache = HashMap::new();
    shards.iterate_shards(|shard, descr| {
        shards_cache.insert(shard.clone(), descr.clone());
        Ok(true)
    })?;
    let shards = shards_cache;

    // Calculate indexes for each validator

    let mut indexes = HashMap::new();
    for (validator, unreliability) in common_stat.values.iter().enumerate().map(|(v, u)| (v as u16, u)) {

        // Find our shard's descr
        let (descr, descr2) = if let Some(descr) = shards.get(shard) {
            (descr, None)
        } else if let Ok(Some(parent)) = shard.merge().map(|s| shards.get(&s)) {
            (parent, None)
        } else {
            let (child_shard1, child_shard2) = shard.split()?;
            let child1 = shards.get(&child_shard1)
                .ok_or_else(|| error!("Shard descr {} is not found", child_shard1))?;
            let child2 = shards.get(&child_shard2)
                .ok_or_else(|| error!("Shard descr {} is not found", child_shard2))?;
            (child1, Some(child2))
        };

        // Familiarity index
        let mut familiarity = 0;
        let mut cur_seqno = 0;
        if descr.collators.is_some() {
            if let Some(descr2) = descr2 {
                let f1 = descr.collators()?.stat.get(validator).unwrap_or(0) as u32;
                let f2 = descr2.collators()?.stat.get(validator).unwrap_or(0) as u32;
                familiarity = (f1 + f2) / 2 * config.familiarity_weight as u32;
                cur_seqno = descr.seq_no.max(descr2.seq_no);
            } else {
                familiarity = descr.collators()?.stat.get(validator).unwrap_or(0) as u32 *
                    config.familiarity_weight as u32;
                cur_seqno = descr.seq_no;
            }
        }

        if cur_seqno > block_range.start {
            fail!("The start of the desired range {} must be greater than last commited seqno {}",
                block_range.start, cur_seqno);
        }

        // Busyness index
        let mut busyness = 0u32;
        let relative_block_range = block_range.start - cur_seqno..block_range.end - cur_seqno;
        for (other_shard, other_descr) in shards.iter() {
            if !other_shard.intersect_with(shard) {
                busyness += calc_busyness_fine_for_shard(other_descr, &relative_block_range, validator);
            }
        }
        busyness *= config.busyness_weight as u32;

        // Unreliability - just take the value from the stat
        let unreliability = *unreliability as u32 * config.unreliability_weight as u32;

        // println!("Validator #{}, familiarity: {}, busyness: {}, unreliability: {}", validator, familiarity, busyness, unreliability);
        indexes.insert(validator, familiarity + busyness + unreliability);
    }

    // Choose candidates
    
    let get_next = |black_list: &mut HashSet<u16>| -> Result<u16> {

        // max index is 100%, min - 0%
        // Choose indexes that are in 0..config.candidates_percentile percent
        let mut min_index = u32::MAX;
        let mut max_index = 0;
        for (validator, index) in indexes.iter() {
            if !black_list.contains(validator) {
                min_index = min_index.min(*index);
                max_index = max_index.max(*index);
            }
        }
        let trashhold = max_index.saturating_sub(min_index) * (config.candidates_percentile as u32);
        let mut candidates = vec!();
        for (validator, index) in &indexes {
            if !black_list.contains(validator) && (index - min_index) * 100 <= trashhold {
                candidates.push(*validator);
            }
        }
        candidates.sort();

        if candidates.is_empty() {
            fail!("No candidates found")
        } else if candidates.len() == 1 {
            black_list.insert(candidates[0]);
            return Ok(candidates[0])
        } else {
            let s = salt.iter().fold(0, |acc, x| acc ^ *x);
            let i = s as usize % candidates.len();
            black_list.insert(candidates[i]);
            return Ok(candidates[i])
        }
    };

    let mut black_list = HashSet::<u16>::from_iter(black_list.iter().cloned());
    let mut new_mempool = MempoolSmallVec::new();

    if let Some(mempool) = mempool {
        if mempool.len() != config.mempool_validators_count as usize {
            fail!("Can't change mempool partially - Old mempool size is not equal to the new one")
        }

        // delete old mempool nodes from the begin
        for i in config.mempool_rotated_count..config.mempool_validators_count {
            new_mempool.push(mempool[i as usize]);
            black_list.insert(mempool[i as usize]);
        }
    }

    let collator = get_next(&mut black_list)?;

    if mempool.is_some() {
        // add new nodes to the end
        for _ in 0..config.mempool_rotated_count {
            new_mempool.push(get_next(&mut black_list)?);
        }
    } else {
        for _ in 0..config.mempool_validators_count {
            new_mempool.push(get_next(&mut black_list)?);
        }
    }

    Ok((collator, new_mempool))
}


#[cfg(test)]
#[path = "tests/test_validators.rs"]
mod tests;

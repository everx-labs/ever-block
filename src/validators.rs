/*
* Copyright 2018-2019 TON DEV SOLUTIONS LTD.
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

use super::*;

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
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        self.validator_list_hash_short.write_to(cell)?;
        self.catchain_seqno.write_to(cell)?;
        self.nx_cc_updated.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for ValidatorInfo {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
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
    validator_list_hash_short: u32,
    catchain_seqno: u32,
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
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        self.validator_list_hash_short.write_to(cell)?;
        self.catchain_seqno.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for ValidatorBaseInfo {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
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
    public_key: SigPubKey, 
    weight: u64 
}

impl ValidatorDescr {
    pub fn new() -> Self {
        ValidatorDescr {
            public_key: SigPubKey::default(),
            weight: 0,
        }
    }

    pub fn with_params(
        public_key: SigPubKey, 
        weight: u64) -> Self 
    {
        ValidatorDescr {
            public_key,
            weight,
        }
    }
}

const VALIDATOR_DESC_TAG: u8 = 0x53;


impl Serializable for ValidatorDescr {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        cell.append_u8(VALIDATOR_DESC_TAG)?;
        self.public_key.write_to(cell)?;
        self.weight.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for ValidatorDescr {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        let tag = cell.get_next_byte()?;
        if tag != VALIDATOR_DESC_TAG {
            bail!(BlockErrorKind::InvalidConstructorTag(tag as u32, "ValidatorDescr".into()))
        }
        self.public_key.read_from(cell)?;
        self.weight.read_from(cell)?;
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
*/

///
/// ValidatorSet
/// 
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatorSet {
    pub utime_since: u32,
    pub utime_until: u32, 
    pub total: Number16, 
    pub main: Number16, 
    pub list: HashmapE,    
    list_key: u16, 
}

impl Default for ValidatorSet {
    fn default() -> Self {
        ValidatorSet {
            utime_since: 0,
            utime_until: 0, 
            total: Number16::default(), 
            main: Number16::default(), 
            list: HashmapE::with_bit_len(16), 
            list_key: 0
        }        
    }
}

impl ValidatorSet {
    pub fn new() -> Self {
        ValidatorSet {
            utime_since: 0,
            utime_until: 0, 
            total: Number16::default(), 
            main: Number16::default(), 
            list: HashmapE::with_bit_len(16), 
            list_key: 0
        }
    }

    pub fn with_params(
        utime_since: u32,
        utime_until: u32, 
        total: Number16, 
        main: Number16) -> Self 
    {
        ValidatorSet {
            utime_since,
            utime_until, 
            total, 
            main, 
            list: HashmapE::with_bit_len(16),
            list_key: 0
        }
    }

    pub fn add_validator_desc(&mut self, validator: ValidatorDescr) {
        self.list_key += 1;
        let key = self.list_key.write_to_new_cell().unwrap();
        self.list.set(key.into(), &validator.write_to_new_cell().unwrap().into()).unwrap();
    }
}

const VALIDATOR_SET_TAG: u8 = 0x11;

impl Serializable for ValidatorSet {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        cell.append_u8(VALIDATOR_SET_TAG)?;
        self.utime_since.write_to(cell)?;
        self.utime_until.write_to(cell)?;
        self.total.write_to(cell)?;
        self.main.write_to(cell)?;
        if self.list.is_empty() {
            bail!(BlockErrorKind::InvalidData("self.list is empty".into()))
        }
        self.list.write_hashmap_root(cell)?;
        Ok(())
    }
}

impl Deserializable for ValidatorSet {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        let tag = cell.get_next_byte()?;
        if tag != VALIDATOR_SET_TAG {
            bail!(BlockErrorKind::InvalidConstructorTag(tag as u32, "ValidatorSet".into()))
        }
        self.utime_since.read_from(cell)?;
        self.utime_until.read_from(cell)?;
        self.total.read_from(cell)?;
        self.main.read_from(cell)?;
        self.list = HashmapE::with_bit_len(16);
        self.list.read_hashmap_root(cell)?;
        self.list_key = self.list.len()? as u16;
        Ok(())
    }
}
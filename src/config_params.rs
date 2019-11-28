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
use self::{BlockResult, BlockErrorKind};


/*
1.6.3. Quick access through the header of masterchain blocks
_ config_addr:uint256
config:^(Hashmap 32 ^Cell) = ConfigParams;
*/
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigParams {
    pub config_addr: UInt256,
    pub config_params: HashmapE // <u32, SliceData>
}

impl Default for ConfigParams {
    fn default() -> ConfigParams {
        ConfigParams {
            config_addr: UInt256::default(),
            config_params: HashmapE::with_bit_len(32)
        }
    }
}

impl ConfigParams {
    /// create new instance ConfigParams
    pub fn new() -> Self {
        Self::default()
    }

    /// get config by index
    pub fn config(&self, index: u32) -> Option<ConfigParamEnum> {
        let key = index.write_to_new_cell().unwrap();
        if let Ok(Some(slice)) = self.config_params.get(key.into()) {
            if let Ok(cell) = slice.reference(0) {
                return ConfigParamEnum::construct_from_slice_and_number(&mut cell.into(), index).ok()
            }
        }
        None
    }

    /// set config
    pub fn set_config(&mut self, config: ConfigParamEnum) -> BlockResult<()> {

        let mut value = BuilderData::new();
        let index = config.write_to_cell(&mut value)?;
        let mut key = BuilderData::default();
        key.append_u32(index).unwrap();
        self.config_params.set(key.into(), &value.into())?;
        Ok(())
    }   
}

impl Deserializable for ConfigParams {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        self.config_addr.read_from(cell)?;
        *self.config_params.data_mut() = Some(cell.checked_drain_reference()?.clone());
        Ok(())
    }
}


impl Serializable for ConfigParams {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        if self.config_params.is_empty() {            
            bail!(BlockErrorKind::InvalidOperation("config_params is empty".into()))
        }
        self.config_addr.write_to(cell)?;
        cell.append_reference_cell(self.config_params.data().unwrap().clone());
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfigParamEnum {
    ConfigParam0(ConfigParam0),
    ConfigParam1(ConfigParam1),
    ConfigParam12(ConfigParam12),
    ConfigParam16(ConfigParam16),
    ConfigParam17(ConfigParam17),
    ConfigParam18(ConfigParam18),
    ConfigParam20(GasLimitsPrices),
    ConfigParam21(GasLimitsPrices),
    ConfigParam22(ConfigParam22),
    ConfigParam23(ConfigParam23),
    ConfigParam24(MsgForwardPrices),
    ConfigParam25(MsgForwardPrices),
    ConfigParam28(CatchainConfig),    
    ConfigParam31(ConfigParam31),
    ConfigParam32(ConfigParam32),
    ConfigParam34(ConfigParam34),
    ConfigParam36(ConfigParam36),
    ConfigParam39(ConfigParam39),
    ConfigParamAny(u32, SliceData),
}

macro_rules! read_config {
    ( $cpname:ident, $cname:ident, $slice:expr ) => {
        {
            let mut c = $cname::default();
            c.read_from($slice)?;
            Ok(ConfigParamEnum::$cpname(c))
        }
    }
}

impl ConfigParamEnum {
    
    /// read config from cell
    pub fn construct_from_slice_and_number(slice: &mut SliceData, index: u32) -> BlockResult<ConfigParamEnum> {
        match index {
            0 => { read_config!(ConfigParam0, ConfigParam0, slice) },
            1 => { read_config!(ConfigParam1, ConfigParam1, slice) },
            12 => { read_config!(ConfigParam12, ConfigParam12, slice) },
            16 => { read_config!(ConfigParam16, ConfigParam16, slice) },
            17 => { read_config!(ConfigParam17, ConfigParam17, slice) },
            18 => { read_config!(ConfigParam18, ConfigParam18, slice) },
            20 => { read_config!(ConfigParam20, GasLimitsPrices, slice) },
            21 => { read_config!(ConfigParam21, GasLimitsPrices, slice) },
            22 => { read_config!(ConfigParam22, ConfigParam22, slice) },
            23 => { read_config!(ConfigParam23, ConfigParam23, slice) },
            24 => { read_config!(ConfigParam24, MsgForwardPrices, slice) },
            25 => { read_config!(ConfigParam25, MsgForwardPrices, slice) },
            28 => { read_config!(ConfigParam28, CatchainConfig, slice) },
            31 => { read_config!(ConfigParam31, ConfigParam31, slice) },
            32 => { read_config!(ConfigParam32, ConfigParam32, slice) },
            34 => { read_config!(ConfigParam34, ConfigParam34, slice) },
            36 => { read_config!(ConfigParam36, ConfigParam36, slice) },
            39 => { read_config!(ConfigParam39, ConfigParam39, slice) },
            index @ _ => Ok(ConfigParamEnum::ConfigParamAny(index, slice.clone())),
        }
    }

    /// Save config to cell
    pub fn write_to_cell(&self, cell: &mut BuilderData) -> BlockResult<u32> {
        match self {
            ConfigParamEnum::ConfigParam0(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(0)},
            ConfigParamEnum::ConfigParam1(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(1)},
            ConfigParamEnum::ConfigParam12(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(12)},
            ConfigParamEnum::ConfigParam16(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(16)},
            ConfigParamEnum::ConfigParam17(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(17)},
            ConfigParamEnum::ConfigParam18(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(18)},
            ConfigParamEnum::ConfigParam20(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(20)},
            ConfigParamEnum::ConfigParam21(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(21)},
            ConfigParamEnum::ConfigParam22(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(22)},
            ConfigParamEnum::ConfigParam23(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(23)},
            ConfigParamEnum::ConfigParam24(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(24)},
            ConfigParamEnum::ConfigParam25(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(25)},
            ConfigParamEnum::ConfigParam28(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(28)},
            ConfigParamEnum::ConfigParam31(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(31)},
            ConfigParamEnum::ConfigParam32(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(32)},
            ConfigParamEnum::ConfigParam34(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(34)},
            ConfigParamEnum::ConfigParam36(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(36)},
            ConfigParamEnum::ConfigParam39(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(39)},
            ConfigParamEnum::ConfigParamAny(index, slice) => { cell.append_reference_cell(slice.into_cell()); Ok(*index)},
        }
    }
}

/*
_ config_addr:bits256 = ConfigParam 0;
*/

///
/// Config Param 0 structure
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ConfigParam0 {
    pub config_addr: UInt256,
}

impl ConfigParam0 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deserializable for ConfigParam0 {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        self.config_addr.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam0 {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        self.config_addr.write_to(cell)?;
        Ok(())
    }
}

/*
_ elector_addr:bits256 = ConfigParam 1;
*/

///
/// Config Param 1 structure
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ConfigParam1 {
    pub elector_addr: UInt256,
}

impl ConfigParam1 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deserializable for ConfigParam1 {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        self.elector_addr.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam1 {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        self.elector_addr.write_to(cell)?;
        Ok(())
    }
}

/*
_ max_validators:(## 16) max_main_validators:(## 16) min_validators:(## 16) 
  { max_validators >= max_main_validators } 
  { max_main_validators >= min_validators } 
  { min_validators >= 1 }
  = ConfigParam 16;
*/

///
/// Config Param 16 structure
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ConfigParam16 {
    pub max_validators: Number16,
    pub max_main_validators: Number16,
    pub min_validators: Number16,
}

impl ConfigParam16 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deserializable for ConfigParam16 {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        self.max_validators.read_from(cell)?;
        self.max_main_validators.read_from(cell)?;
        self.min_validators.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam16 {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        self.max_validators.write_to(cell)?;
        self.max_main_validators.write_to(cell)?;
        self.min_validators.write_to(cell)?;
        Ok(())
    }
}

/*
_ min_stake:Grams max_stake:Grams max_stake_factor:uint32 = ConfigParam 17;
*/

///
/// Config Param 17 structure
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ConfigParam17 {
    pub min_stake: Grams,
    pub max_stake: Grams,
    pub max_stake_factor: u32,
}

impl ConfigParam17 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deserializable for ConfigParam17 {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        self.min_stake.read_from(cell)?;
        self.max_stake.read_from(cell)?;
        self.max_stake_factor.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam17 {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        self.min_stake.write_to(cell)?;
        self.max_stake.write_to(cell)?;
        self.max_stake_factor.write_to(cell)?;
        Ok(())
    }
}

/*
_#cc 
    utime_since:uint32 
    bit_price_ps:uint64 
    cell_price_ps:uint64 
    mc_bit_price_ps:uint64 
    mc_cell_price_ps:uint64 
= StoragePrices;

_ (Hashmap 32 StoragePrices) = ConfigParam 18;
*/

///
/// StoragePrices structure
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct StoragePrices {
    pub utime_since: u32,
    pub bit_price_ps: u64,
    pub cell_price_ps: u64,
    pub mc_bit_price_ps: u64,
    pub mc_cell_price_ps: u64,
}

impl StoragePrices {
    pub fn new() -> Self {
        Self::default()
    }
}

const STORAGE_PRICES_TAG: u8 = 0xCC;

impl Deserializable for StoragePrices {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        let tag = cell.get_next_byte()?;
        if tag != STORAGE_PRICES_TAG {
            bail!(BlockErrorKind::InvalidConstructorTag(tag as u32, "StoragePrices".into()))
        }
        self.utime_since.read_from(cell)?;
        self.bit_price_ps.read_from(cell)?;
        self.cell_price_ps.read_from(cell)?;
        self.mc_bit_price_ps.read_from(cell)?;
        self.mc_cell_price_ps.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for StoragePrices {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        cell.append_u8(STORAGE_PRICES_TAG)?;
        self.utime_since.write_to(cell)?;
        self.bit_price_ps.write_to(cell)?;
        self.cell_price_ps.write_to(cell)?;
        self.mc_bit_price_ps.write_to(cell)?;
        self.mc_cell_price_ps.write_to(cell)?;
        Ok(())
    }
}

///
/// ConfigParam 18 struct
/// 
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigParam18 {
    map: HashmapE,
    index: u32
}

impl Default for ConfigParam18 {
    fn default() -> Self {
        ConfigParam18 {
            map: HashmapE::with_bit_len(32),
            index: 0,
        }
    }
}

impl ConfigParam18 {
    /// new instance of ConfigParam18
    pub fn new() -> Self {
        Self::default()
    }

    /// get length
    pub fn len(&self) -> BlockResult<usize> {
        self.map.len().map_err(|e| BlockError::from(e))
    } 

    /// get value by index
    pub fn get(&self, index: u32) -> BlockResult<StoragePrices> {
        let key = index.write_to_new_cell().unwrap().into();
        let mut s = self.map.get(key).map_err(|e| BlockError::from(e))?
            .ok_or(BlockErrorKind::InvalidIndex(index as usize))?;
        StoragePrices::construct_from(&mut s)
    }

    /// insert value
    pub fn insert(&mut self, sp: StoragePrices) {
        self.index += 1;
        let key = self.index.write_to_new_cell().unwrap();
        self.map.set(key.into(), &sp.write_to_new_cell().unwrap().into()).unwrap();
    }
}


impl Deserializable for ConfigParam18 {
    fn read_from(&mut self, slice: &mut SliceData) -> BlockResult<()> {
        self.map.read_hashmap_root(slice)?;
        self.index = self.map.len()? as u32;
        Ok(())
    }
}

impl Serializable for ConfigParam18 {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        if self.map.is_empty() {
            bail!(BlockErrorKind::InvalidOperation("self.map is empty".into()))
        }
        self.map.write_hashmap_root(cell)?;
        Ok(())
    }
}

/*
gas_prices#dd 
    gas_price:uint64 
    gas_limit:uint64 
    gas_credit:uint64 
    block_gas_limit:uint64 
    freeze_due_limit:uint64 
    delete_due_limit:uint64 
= GasLimitsPrices; 

gas_prices_ext#de
  gas_price:uint64
  gas_limit:uint64
  special_gas_limit:uint64
  gas_credit:uint64
  block_gas_limit:uint64
  freeze_due_limit:uint64
  delete_due_limit:uint64 
  = GasLimitsPrices;
*/

///
/// GasLimitsPrices
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct GasLimitsPrices {
    pub gas_price: u64,
    pub gas_limit: u64,
    pub special_gas_limit: Option<u64>, // not good solution - maybe autogeneration later
    pub gas_credit: u64,
    pub block_gas_limit: u64,
    pub freeze_due_limit: u64,
    pub delete_due_limit: u64,
}

impl GasLimitsPrices {
    pub fn new() -> Self {
        Self::default()
    }
}

const GAS_LIMIT_PRICES_TAG: u8 = 0xDD;
const GAS_LIMIT_PRICES_EXT_TAG: u8 = 0xDE;

impl Deserializable for GasLimitsPrices {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        let tag = cell.get_next_byte()?;
        if tag != GAS_LIMIT_PRICES_TAG && tag != GAS_LIMIT_PRICES_EXT_TAG {
            bail!(BlockErrorKind::InvalidConstructorTag(tag as u32, "GasLimitsPrices".into()))
        }
        self.gas_price.read_from(cell)?;
        self.gas_limit.read_from(cell)?;
        self.special_gas_limit = match tag {
            GAS_LIMIT_PRICES_EXT_TAG => Some(cell.get_next_u64()?),
            _ => None
        };
        self.gas_credit.read_from(cell)?;
        self.block_gas_limit.read_from(cell)?;
        self.freeze_due_limit.read_from(cell)?;
        self.delete_due_limit.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for GasLimitsPrices {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        cell.append_u8(match self.special_gas_limit {
            Some(_) => GAS_LIMIT_PRICES_EXT_TAG,
            None => GAS_LIMIT_PRICES_TAG
        })?;
        self.gas_price.write_to(cell)?;
        self.gas_limit.write_to(cell)?;
        if let Some(limit) = self.special_gas_limit {
            limit.write_to(cell)?
        }
        self.gas_credit.write_to(cell)?;
        self.block_gas_limit.write_to(cell)?;
        self.freeze_due_limit.write_to(cell)?;
        self.delete_due_limit.write_to(cell)?;
        Ok(())
    }
}

/*
config_mc_gas_prices#_ GasLimitsPrices = ConfigParam 20;
*/
/*
config_gas_prices#_ GasLimitsPrices = ConfigParam 21;
*/


/*

// msg_fwd_fees = (lump_price + ceil((bit_price * msg.bits + cell_price * msg.cells)/2^16)) nanograms
// ihr_fwd_fees = ceil((msg_fwd_fees * ihr_price_factor)/2^16) nanograms
// bits in the root cell of a message are not included in msg.bits (lump_price pays for them)
msg_forward_prices#ea 
    lump_price:uint64 
    bit_price:uint64 
    cell_price:uint64
    ihr_price_factor:uint32 
    first_frac:uint16 
    next_frac:uint16 
= MsgForwardPrices;

*/

///
/// MsgForwardPrices
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct MsgForwardPrices {
    pub lump_price: u64,
    pub bit_price: u64,
    pub cell_price: u64,
    pub ihr_price_factor: u32,
    pub first_frac: u16,
    pub next_frac: u16,
}

impl MsgForwardPrices {
    pub fn new() -> Self {
        Self::default()
    }
}

const MSG_FWD_PRICES_TAG: u8 = 0xEA;

impl Deserializable for MsgForwardPrices {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        let tag = cell.get_next_byte()?;
        if tag != MSG_FWD_PRICES_TAG {
            bail!(BlockErrorKind::InvalidConstructorTag(tag as u32, "MsgForwardPrices".into()))
        }
        self.lump_price.read_from(cell)?;
        self.bit_price.read_from(cell)?;
        self.cell_price.read_from(cell)?;
        self.ihr_price_factor.read_from(cell)?;
        self.first_frac.read_from(cell)?;
        self.next_frac.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for MsgForwardPrices {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        cell.append_u8(MSG_FWD_PRICES_TAG)?;
        self.lump_price.write_to(cell)?;
        self.bit_price.write_to(cell)?;
        self.cell_price.write_to(cell)?;
        self.ihr_price_factor.write_to(cell)?;
        self.first_frac.write_to(cell)?;
        self.next_frac.write_to(cell)?;
        Ok(())
    }
}

/*
// used for messages to/from masterchain
config_mc_fwd_prices#_ MsgForwardPrices = ConfigParam 24;
// used for all other messages
config_fwd_prices#_ MsgForwardPrices = ConfigParam 25;

*/


/*
catchain_config#c1 
    mc_catchain_lifetime:uint32 
    shard_catchain_lifetime:uint32 
    shard_validators_lifetime:uint32 
    shard_validators_num:uint32 
= CatchainConfig;
*/

///
/// MsgForwardPrices
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct CatchainConfig {
    pub mc_catchain_lifetime: u32,
    pub shard_catchain_lifetime: u32,
    pub shard_validators_lifetime: u32,
    pub shard_validators_num: u32,
}

impl CatchainConfig {
    pub fn new() -> Self {
        Self::default()
    }
}

const CATCHAIN_CONFIG_TAG: u8 = 0xC1;

impl Deserializable for CatchainConfig {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        let tag = cell.get_next_byte()?;
        if tag != CATCHAIN_CONFIG_TAG {
            bail!(BlockErrorKind::InvalidConstructorTag(tag as u32, "CatchainConfig".into()))
        }
        self.mc_catchain_lifetime.read_from(cell)?;
        self.shard_catchain_lifetime.read_from(cell)?;
        self.shard_validators_lifetime.read_from(cell)?;
        self.shard_validators_num.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for CatchainConfig {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        cell.append_u8(CATCHAIN_CONFIG_TAG)?;
        self.mc_catchain_lifetime.write_to(cell)?;
        self.shard_catchain_lifetime.write_to(cell)?;
        self.shard_validators_lifetime.write_to(cell)?;
        self.shard_validators_num.write_to(cell)?;
        Ok(())
    }
}

/*
 _ CatchainConfig = ConfigParam 28;
 */


/*
_ fundamental_smc_addr:(HashmapE 256 True) = ConfigParam 31;
*/

///
/// ConfigParam 31;
/// 
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigParam31 {
    pub fundamental_smc_addr: HashmapE,
}

impl Default for ConfigParam31{
    fn default() -> Self {
        ConfigParam31 {
            fundamental_smc_addr: HashmapE::with_bit_len(256)
        }
    }
}

impl ConfigParam31 {
    pub fn new() -> Self {
        ConfigParam31::default()
    }

    pub fn add_address(&mut self, address: UInt256) {
        self.fundamental_smc_addr.set(
            address.write_to_new_cell().unwrap().into(),
            &SliceData::default()
        ).unwrap();
    }
}

impl Deserializable for ConfigParam31 {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        self.fundamental_smc_addr.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam31 {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        self.fundamental_smc_addr.write_to(cell)?;
        Ok(())
    }
}


macro_rules! define_configparams {
    ( $cpname:ident, $pname:ident ) => {
        ///
        /// $cpname structure
        /// 
        #[derive(Clone, Debug, Eq, PartialEq, Default)]
        pub struct $cpname {
            pub $pname: ValidatorSet,
        }

        impl $cpname {
            /// create new instance of $cpname
            pub fn new() -> Self {
                $cpname::default()
            }
        }

        impl Deserializable for $cpname {
            fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
                self.$pname.read_from(cell)?;
                Ok(())
            }
        }

        impl Serializable for $cpname {
            fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
                self.$pname.write_to(cell)?;
                Ok(())
            }
        }
    }
}

/*
_ prev_validators:ValidatorSet = ConfigParam 32;
*/
define_configparams!(ConfigParam32, prev_validators);
/*
_ cur_validators:ValidatorSet = ConfigParam 34;
*/
define_configparams!(ConfigParam34, cur_validators);
/*
_ next_validators:ValidatorSet = ConfigParam 36;
*/
define_configparams!(ConfigParam36, next_validators);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkchainFormat {
    Basic(WorkchainFormat1),
    Extended(WorkchainFormat0),
}


impl Deserializable for WorkchainFormat {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        *self = match cell.get_next_bit()? {
            true => {
                let mut val = WorkchainFormat1::default();
                val.read_from(cell)?;
                WorkchainFormat::Basic(val)
            }
            false => {
                let mut val = WorkchainFormat0::default();
                val.read_from(cell)?;
                WorkchainFormat::Extended(val)
            }
        };
        Ok(())
    }
}

impl Serializable for WorkchainFormat {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        match self {
            WorkchainFormat::Basic(ref val) => {
                cell.append_bit_one()?;
                val.write_to(cell)?;
            },
            WorkchainFormat::Extended(val) => {
                cell.append_bit_zero()?;
                val.write_to(cell)?;
            }
        }
        Ok(())
    }
}

/*
wfmt_basic#1 
	vm_version:int32 
	vm_mode:uint64 
= WorkchainFormat 1;
*/

///
/// Workchain format basic
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct WorkchainFormat1 {
    pub vm_version: i32,
    pub vm_mode: u64,
}

impl WorkchainFormat1 {
    ///
    /// Create empty intance of WorkchainFormat1
    /// 
    pub fn new() -> Self {
        WorkchainFormat1::default()
    }

    ///
    /// Create new instance of WorkchainFormat1
    /// 
    pub fn with_params(vm_version: i32, vm_mode: u64) -> Self {
        WorkchainFormat1 {
            vm_version,
            vm_mode
        }
    }
}


impl Deserializable for WorkchainFormat1 {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        self.vm_version.read_from(cell)?;
        self.vm_mode.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for WorkchainFormat1 {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        self.vm_version.write_to(cell)?;
        self.vm_mode.write_to(cell)?;
        Ok(())
    }
}


/*
wfmt_ext#0 
	min_addr_len:(## 12) 
	max_addr_len:(## 12) 
	addr_len_step:(## 12)
  { min_addr_len >= 64 } { min_addr_len <= max_addr_len } 
  { max_addr_len <= 1023 } { addr_len_step <= 1023 }
  workchain_type_id:(## 32) { workchain_type_id >= 1 }
= WorkchainFormat 0;
*/

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkchainFormat0 {
    min_addr_len: u16,  //use 12 bit
    max_addr_len: u16,  //use 12 bit
    addr_len_step: u16, // use 12 bit
    workchain_type_id: u32
}

impl Default for WorkchainFormat0 {
    fn default() -> Self {
        WorkchainFormat0 {
            min_addr_len: 64,
            max_addr_len: 64,
            addr_len_step: 0,
            workchain_type_id: 1
        }
    }
}

impl WorkchainFormat0 {
    ///
    /// Create empty new instance of WorkchainFormat0
    /// 
    pub fn new() -> Self {
        WorkchainFormat0::default()
    }

    ///
    /// Create new instance of WorkchainFormat0
    /// 
    pub fn with_params(min_addr_len: u16, max_addr_len: u16, addr_len_step: u16, workchain_type_id: u32 ) -> BlockResult<WorkchainFormat0> {
        if min_addr_len >= 64 && min_addr_len <= max_addr_len &&
           max_addr_len <= 1023 && addr_len_step <= 1023 &&
           workchain_type_id >= 1 {
               Ok(
                   WorkchainFormat0 {
                        min_addr_len,
                        max_addr_len,
                        addr_len_step,
                        workchain_type_id, 
                   }
               )
           }
        else {
            block_err!(BlockErrorKind::InvalidData("min_addr_len >= 64 && min_addr_len <= max_addr_len\
            && max_addr_len <= 1023 && addr_len_step <= 1023 && workchain_type_id >= 1".to_string()))
        }
    }

    ///
    /// Getter for min_addr_len
    /// 
    pub fn min_addr_len(&self) -> u16 {
        self.min_addr_len
    }

    ///
    /// Setter for min_addr_len
    /// 
    pub fn set_min_addr_len(&mut self, min_addr_len: u16) -> BlockResult<()> {
        if min_addr_len >= 64 && min_addr_len <= 1023 {
            self.min_addr_len = min_addr_len;
            Ok(())
        } else {
            block_err!(BlockErrorKind::InvalidData("should: min_addr_len >= 64 && min_addr_len <= 1023".to_string()))
        }
    }    

    ///
    /// Getter for min_addr_len
    /// 
    pub fn max_addr_len(&self) -> u16 {
        self.max_addr_len
    }

    ///
    /// Setter for max_addr_len
    /// 
    pub fn set_max_addr_len(&mut self, max_addr_len: u16) -> BlockResult<()> {
        if max_addr_len >= 64 && max_addr_len <= 1024 && self.min_addr_len <= max_addr_len {
            self.max_addr_len = max_addr_len;
            Ok(())
        } else {
            block_err!(BlockErrorKind::InvalidData("should: max_addr_len >= 64 && max_addr_len <= 1024 && self.min_addr_len <= max_addr_len".to_string()))
        }
    }        

    ///
    /// Getter for addr_len_step
    /// 
    pub fn addr_len_step(&self) -> u16 {
        self.addr_len_step
    }

    ///
    /// Setter for min_addr_len
    /// 
    pub fn set_addr_len_step(&mut self, addr_len_step: u16) -> BlockResult<()> {
        if addr_len_step <= 1024 {
            self.addr_len_step = addr_len_step;
            Ok(())
        } else {
            block_err!(BlockErrorKind::InvalidData("should: addr_len_step <= 1024".to_string()))
        }
    }       

    ///
    /// Getter for workchain_type_id
    /// 
    pub fn workchain_type_id(&self) -> u32 {
        self.workchain_type_id
    }

    ///
    /// Setter for min_addr_len
    /// 
    pub fn set_workchain_type_id(&mut self, workchain_type_id: u32) -> BlockResult<()> {
        if workchain_type_id >= 1 {
            self.workchain_type_id = workchain_type_id;
            Ok(())
        } else {
            block_err!(BlockErrorKind::InvalidData("should: workchain_type_id >= 1".to_string()))
        }
    } 
}

impl Deserializable for WorkchainFormat0 {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        let mut val = Number12::default();
        val.read_from(cell)?;
        self.min_addr_len = val.0 as u16;
        val.read_from(cell)?;
        self.max_addr_len = val.0 as u16;
        val.read_from(cell)?;
        self.addr_len_step = val.0 as u16;
        let mut val = Number32::default();
        val.read_from(cell)?;
        self.workchain_type_id = val.0;
        if self.min_addr_len >= 64 && self.min_addr_len <= self.max_addr_len &&
           self.max_addr_len <= 1023 && self.addr_len_step <= 1023 &&
           self.workchain_type_id > 1 {
                Ok(())
        } else {
            block_err!(BlockErrorKind::InvalidData("should: min_addr_len >= 64 && min_addr_len <= max_addr_len && max_addr_len <= 1023 && addr_len_step <= 1023".to_string()))
        }
    }
}

impl Serializable for WorkchainFormat0 {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        if self.min_addr_len >= 64 && self.min_addr_len <= self.max_addr_len &&
           self.max_addr_len <= 1023 && self.addr_len_step <= 1023 &&
           self.workchain_type_id >= 1 {
                let min = Number12(self.min_addr_len as u32);
                let max = Number12(self.max_addr_len as u32);
                let len = Number12(self.addr_len_step as u32);
                let id = Number32(self.workchain_type_id);
                min.write_to(cell)?;
                max.write_to(cell)?;
                len.write_to(cell)?;
                id.write_to(cell)?;
                Ok(())
        } else {
            block_err!(BlockErrorKind::InvalidData("should: min_addr_len >= 64 && min_addr_len <= max_addr_len && max_addr_len <= 1023 && addr_len_step <= 1023".to_string()))
        }
    }
}



/*
workchain#a5 
	enabled_since:uint32 
	min_split:(## 8)
	 max_split:(## 8)
  { min_split <= max_split } { max_split <= 60 }
  basic:(## 1) 
	active:Bool 
	accept_msgs:Bool 
	flags:(## 13) { flags = 0 }
  zerostate_root_hash:bits256 
	zerostate_file_hash:bits256
  version:uint32 
	format:(WorkchainFormat basic)
= WorkchainDescr;
*/

///
/// WorkchainDescr structure
/// 
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkchainDescr {
    pub enabled_since: u32,
    min_split: u8,
    max_split: u8,
    //basic: bool, // depends on format 
    pub active: bool,
    pub accept_msgs: bool,
    pub flags: u16, // 13 bit
    pub zerostate_root_hash: UInt256,
    pub zerostate_file_hash: UInt256,
    pub version: u32,
    pub format: WorkchainFormat
}

impl Default for WorkchainDescr {
    fn default() -> Self {
        WorkchainDescr {
            enabled_since: 0,
            min_split: 0,
            max_split: 0,
            //basic: bool, // depends on format 
            active: false,
            accept_msgs: false,
            flags: 0,
            zerostate_root_hash: UInt256::from([0;32]),
            zerostate_file_hash: UInt256::from([0;32]),
            version: 0,
            format: WorkchainFormat::Basic(WorkchainFormat1::default()),
        }        
    }
}

impl WorkchainDescr {
    ///
    /// Create empty instance of WorkchainDescr
    /// 
    pub fn new() -> Self {
        WorkchainDescr::default()
    }

    ///
    /// Getter for min_split
    /// 
    pub fn min_split(&self) -> u8 {
        self.min_split
    }

    ///
    /// Setter for min_split
    /// 
    pub fn set_min_split(&mut self, min_split: u8) -> BlockResult<()> {
        if min_split <= 60 {
            self.min_split = min_split;
            Ok(())
        } else {
            block_err!(BlockErrorKind::InvalidData("should: min_split <= max_split && max_split <= 60".to_string()))
        }
    } 

    ///
    /// Getter for max_split
    /// 
    pub fn max_split(&self) -> u8 {
        self.max_split
    }

    ///
    /// Setter for max_split
    /// 
    pub fn set_max_split(&mut self, max_split: u8) -> BlockResult<()> {
        if self.min_split <= max_split && max_split <= 60 {
            self.max_split = max_split;
            Ok(())
        } else {
            block_err!(BlockErrorKind::InvalidData("should: min_split <= max_split && max_split <= 60".to_string()))
        }
    } 

}



impl Deserializable for WorkchainDescr {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        self.enabled_since.read_from(cell)?;
        let mut min = Number8::default();
        min.read_from(cell)?;
        self.min_split = min.0 as u8;
        let mut max = Number8::default();
        max.read_from(cell)?;
        self.max_split = max.0 as u8;
        let basic = cell.get_next_bit()?;
        self.active = cell.get_next_bit()?;
        self.accept_msgs = cell.get_next_bit()?;
        let mut flags = Number13::default();
        flags.read_from(cell)?;
        self.flags = flags.0 as u16;
        self.zerostate_root_hash.read_from(cell)?;
        self.zerostate_file_hash.read_from(cell)?;
        self.version.read_from(cell)?;
        self.format = match basic {
            true => {
                let mut val = WorkchainFormat1::default();
                val.read_from(cell)?;
                WorkchainFormat::Basic(val)
            },
            false => {
                let mut val = WorkchainFormat0::default();
                val.read_from(cell)?;
                WorkchainFormat::Extended(val)
            }
        };

        Ok(())
    }
}

impl Serializable for WorkchainDescr {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        if self.min_split <= self.max_split && self.max_split <= 60 {

            self.enabled_since.write_to(cell)?;

            let min = Number8(self.min_split as u32);
            min.write_to(cell)?;

            let max = Number8(self.max_split as u32);
            max.write_to(cell)?;

            if let WorkchainFormat::Basic(_) = self.format {
                cell.append_bit_one()?;
            } else {
                cell.append_bit_zero()?;
            }

            if self.active {
                cell.append_bit_one()?;
            } else {
                cell.append_bit_zero()?;
            }

            if self.accept_msgs {
                cell.append_bit_one()?;
            } else {
                cell.append_bit_zero()?;
            }

            let flags = Number13(self.flags as u32);
            flags.write_to(cell)?;
            self.zerostate_root_hash.write_to(cell)?;
            self.zerostate_file_hash.write_to(cell)?;
            self.version.write_to(cell)?;
            self.format.write_to(cell)?;

            Ok(())
        } else {
            block_err!(BlockErrorKind::InvalidData("should: min_split <= max_split && max_split <= 60".to_string()))
        }
    }
}


/*
_ workchains:(HashmapE 32 WorkchainDescr) = ConfigParam 12;
*/

///
/// ConfigParam 12 struct
/// 
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigParam12 {
    pub workchains: HashmapE,
}

impl Default for ConfigParam12 {
    fn default() -> Self {
        ConfigParam12 {
            workchains: HashmapE::with_bit_len(32),
        }
    }
}

impl ConfigParam12 {
    /// new instance of ConfigParam18
    pub fn new() -> Self {
        Self::default()
    }

    /// get length
    pub fn len(&self) -> BlockResult<usize> {
        Ok(self.workchains.len()?)
    } 

    /// get value by index
    pub fn get(&self, workchain_id: i32) -> BlockResult<WorkchainDescr> {
        self.workchains.get(workchain_id.write_to_new_cell().unwrap().into())
            .map(|ref mut s| -> BlockResult<WorkchainDescr> 
                {
                    let mut sp = WorkchainDescr::default(); 
                    if let Some(s) = s {
                       sp.read_from(s)?
                    } else {
                        return block_err!(BlockErrorKind::NotFound("WorkchainDescr".to_string()));
                    }; 
                    Ok(sp)
                }).unwrap()
    }

    /// insert value
    pub fn insert(&mut self, workchain_id: i32, sp: &WorkchainDescr) {
        let key = workchain_id.write_to_new_cell().unwrap();
        self.workchains.set(key.into(), &sp.write_to_new_cell().unwrap().into()).unwrap();
    }
}


impl Deserializable for ConfigParam12 {
    fn read_from(&mut self, slice: &mut SliceData) -> BlockResult<()> {
        self.workchains = HashmapE::with_bit_len(32);
        self.workchains.read_from(slice)?;
        Ok(())
    }
}

impl Serializable for ConfigParam12 {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        self.workchains.write_to(cell)?;
        Ok(())
    }
}

// validator_temp_key#3
//     adnl_addr:bits256
//     temp_public_key:SigPubKey
//     seqno:#
//     valid_until:uint32
// = ValidatorTempKey;

const VALIDATOR_TEMP_KEY_TAG: u8 = 0x3;

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct ValidatorTempKey {
    adnl_addr: UInt256,
    temp_public_key: SigPubKey,
    seqno: u32,
    valid_until: u32
}

impl ValidatorTempKey {
    /// new instance of ValidatorTempKey
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_params(adnl_addr: UInt256, temp_public_key: SigPubKey, seqno: u32, valid_until: u32) 
        -> Self {
        Self {
            adnl_addr,
            temp_public_key,
            seqno,
            valid_until 
        }
    }

    pub fn set_adnl_addr(&mut self, adnl_addr: UInt256) {
        self.adnl_addr = adnl_addr
    }

    pub fn adnl_addr(&self) -> &UInt256 {
        &self.adnl_addr
    }

    pub fn set_key(&mut self, temp_public_key: SigPubKey) {
        self.temp_public_key = temp_public_key
    }

    pub fn temp_public_key(&self) -> &SigPubKey {
        &self.temp_public_key
    }

    pub fn set_seqno(&mut self, seqno: u32) {
        self.seqno = seqno
    }

    pub fn seqno(&self) -> u32 {
        self.seqno
    }

    pub fn set_valid_until(&mut self, valid_until: u32) {
        self.valid_until = valid_until
    }

    pub fn valid_until(&self) -> u32 {
        self.valid_until
    }
}

impl Deserializable for ValidatorTempKey {
    fn read_from(&mut self, slice: &mut SliceData) -> BlockResult<()> {
        let tag = slice.get_next_byte()?; // TODO what is tag length in bits???
        if tag != VALIDATOR_TEMP_KEY_TAG {
            bail!(BlockErrorKind::InvalidConstructorTag(tag as u32, "ValidatorTempKey".into()))
        }
        self.adnl_addr.read_from(slice)?;
        self.temp_public_key.read_from(slice)?;
        self.seqno.read_from(slice)?;
        self.valid_until.read_from(slice)?;
        Ok(())
    }
}

impl Serializable for ValidatorTempKey {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        cell.append_u8(VALIDATOR_TEMP_KEY_TAG)?; // TODO what is tag length in bits???
        self.adnl_addr.write_to(cell)?;
        self.temp_public_key.write_to(cell)?;
        self.seqno.write_to(cell)?;
        self.valid_until.write_to(cell)?;
        Ok(())
    }
}


// signed_temp_key#4
//     key:^ValidatorTempKey
//     signature:CryptoSignature
// = ValidatorSignedTempKey;

const VALIDATOR_SIGNED_TEMP_KEY_TAG: u8 = 0x4;

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct ValidatorSignedTempKey {
    key: ValidatorTempKey,
    signature: CryptoSignature
}

impl ValidatorSignedTempKey {

    /// new instance of 
    pub fn with_key_and_signature(key: ValidatorTempKey, signature: CryptoSignature) -> Self {
        Self {key, signature}
    }

    pub fn key(&self) -> &ValidatorTempKey {
        &self.key
    }

    pub fn signature(&self) -> &CryptoSignature {
        &self.signature
    }
}

impl Deserializable for ValidatorSignedTempKey {
    fn read_from(&mut self, slice: &mut SliceData) -> BlockResult<()> {
        let tag = slice.get_next_byte()?; // TODO what is tag length in bits???
        if tag != VALIDATOR_SIGNED_TEMP_KEY_TAG {
            bail!(BlockErrorKind::InvalidConstructorTag(tag as u32, "ValidatorSignedTempKey".into()))
        }
        self.signature.read_from(slice)?;
        self.key.read_from(&mut slice.checked_drain_reference()?.into())?;
        Ok(())
    }
}

impl Serializable for ValidatorSignedTempKey {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        cell.append_u8(VALIDATOR_SIGNED_TEMP_KEY_TAG)?; // TODO what is tag length in bits???
        self.signature.write_to(cell)?;
        cell.append_reference(self.key.write_to_new_cell()?);
        Ok(())
    }
}

///
/// ConfigParam 39 struct
/// 
// _ (HashmapE 256 ValidatorSignedTempKey) = ConfigParam 39;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigParam39 {
    pub validator_keys: HashmapE,
}

impl Default for ConfigParam39 {
    fn default() -> Self {
        ConfigParam39 {
            validator_keys: HashmapE::with_bit_len(256),
        }
    }
}

impl ConfigParam39 {
    /// new instance of ConfigParam39
    pub fn new() -> Self {
        Self::default()
    }

    /// get length
    pub fn len(&self) -> BlockResult<usize> {
        Ok(self.validator_keys.len()?)
    } 

    /// get value by key
    pub fn get(&self, key: UInt256) -> BlockResult<ValidatorSignedTempKey> {
        self.validator_keys.get(key.write_to_new_cell().unwrap().into())
            .map(|ref mut s| -> BlockResult<ValidatorSignedTempKey> 
                {
                    let mut sp = ValidatorSignedTempKey::default(); 
                    if let Some(s) = s {
                       sp.read_from(s)?
                    } else {
                        return block_err!(BlockErrorKind::NotFound("ValidatorSignedTempKey".to_string()));
                    }; 
                    Ok(sp)
                }).unwrap()
    }

    /// insert value
    pub fn insert(&mut self, key: &UInt256, validator_key: &ValidatorSignedTempKey) {
        let key = key.write_to_new_cell().unwrap();
        self.validator_keys.set(key.into(), &validator_key.write_to_new_cell().unwrap().into()).unwrap();
    }
}

impl Deserializable for ConfigParam39 {
    fn read_from(&mut self, slice: &mut SliceData) -> BlockResult<()> {
        self.validator_keys = HashmapE::with_bit_len(256);
        self.validator_keys.read_from(slice)?;
        Ok(())
    }
}

impl Serializable for ConfigParam39 {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        self.validator_keys.write_to(cell)?;
        Ok(())
    }
}


///
///  struct ParamLimits
/// 
// param_limits#c3
//     underload:#
//     soft_limit:#
//     { underload <= soft_limit }
//     hard_limit:#
//     { soft_limit <= hard_limit }
// = ParamLimits;

const PARAM_LIMITS_TAG: u8 = 0xc3;

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct ParamLimits {
    underload: u32,
    soft_limit: u32,
    hard_limit: u32
}

impl ParamLimits {
    /// new instance of ParamLimits
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_limits(underload: u32, soft_limit: u32, hard_limit: u32) -> BlockResult<Self> {
        if underload > soft_limit { 
            bail!(BlockErrorKind::InvalidArg("`underload` have to be less or equal `soft_limit`".into())); 
        }
        if soft_limit > hard_limit { 
            bail!(BlockErrorKind::InvalidArg("`soft_limit` have to be less or equal `hard_limit`".into())); 
        }
        Ok(ParamLimits{ underload, soft_limit, hard_limit })
    }

    pub fn underload(&self) -> u32 {
        self.underload
    }

    pub fn set_underload(&mut self, underload: u32) -> BlockResult<()>{
        if underload > self.soft_limit { 
            bail!(BlockErrorKind::InvalidArg("`underload` have to be less or equal `soft_limit`".into())); 
        }
        self.underload = underload;
        Ok(())
    }

    pub fn soft_limit(&self) -> u32 {
        self.soft_limit
    }

    pub fn set_soft_limit(&mut self, soft_limit: u32) -> BlockResult<()>{
        if soft_limit > self.hard_limit { 
            bail!(BlockErrorKind::InvalidArg("`soft_limit` have to be less or equal `hard_limit`".into())); 
        }
        self.soft_limit = soft_limit;
        Ok(())
    }

    pub fn hard_limit(&self) -> u32 {
        self.hard_limit
    }

    pub fn set_hard_limit(&mut self, hard_limit: u32) -> BlockResult<()>{
        if self.soft_limit > hard_limit { 
            bail!(BlockErrorKind::InvalidArg("`hard_limit` have to be larger or equal `soft_limit`".into())); 
        }
        self.hard_limit = hard_limit;
        Ok(())
    }
}

impl Deserializable for ParamLimits {
    fn read_from(&mut self, slice: &mut SliceData) -> BlockResult<()> {
        let tag = slice.get_next_byte()?;
        if tag != PARAM_LIMITS_TAG {
            bail!(BlockErrorKind::InvalidConstructorTag(tag as u32, "ParamLimits".into()))
        }
        self.underload.read_from(slice)?;
        self.soft_limit.read_from(slice)?;
        self.hard_limit.read_from(slice)?;
        if self.underload > self.soft_limit {
            bail!(BlockErrorKind::InvalidData("`underload` have to be less or equal `soft_limit`".into())); 
        }
        if self.soft_limit > self.hard_limit {
            bail!(BlockErrorKind::InvalidData("`soft_limit` have to be less or equal `hard_limit`".into())); 
        }
        Ok(())
    }
}

impl Serializable for ParamLimits {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        cell.append_u8(PARAM_LIMITS_TAG)?;
        self.underload.write_to(cell)?;
        self.soft_limit.write_to(cell)?;
        self.hard_limit.write_to(cell)?;
        Ok(())
    }
}

///
///  struct BlockLimits
/// 
// block_limits#5d
//     bytes:ParamLimits
//     gas:ParamLimits
//     lt_delta:ParamLimits
// = BlockLimits;

const BLOCK_LIMITS_TAG: u8 = 0x5d;

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct BlockLimits {
    bytes: ParamLimits,
    gas: ParamLimits,
    lt_delta: ParamLimits
}

impl BlockLimits {
    /// new instance of ConfigParam39
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_limits(bytes: ParamLimits, gas: ParamLimits, lt_delta: ParamLimits) -> Self {
        Self { bytes, gas, lt_delta }
    }

    pub fn bytes(&self) -> &ParamLimits {
        &self.bytes
    }

    pub fn bytes_mut(&mut self) -> &mut ParamLimits {
        &mut self.bytes
    }

    pub fn gas(&self) -> &ParamLimits {
        &self.gas
    }

    pub fn gas_mut(&mut self) -> &mut ParamLimits {
        &mut self.gas
    }

    pub fn lt_delta(&self) -> &ParamLimits {
        &self.lt_delta
    }

    pub fn lt_delta_mut(&mut self) -> &mut ParamLimits {
        &mut self.lt_delta
    }
}

impl Deserializable for BlockLimits {
    fn read_from(&mut self, slice: &mut SliceData) -> BlockResult<()> {
        let tag = slice.get_next_byte()?;
        if tag != BLOCK_LIMITS_TAG {
            bail!(BlockErrorKind::InvalidConstructorTag(tag as u32, "BlockLimits".into()))
        }
        self.bytes.read_from(slice)?;
        self.gas.read_from(slice)?;
        self.lt_delta.read_from(slice)?;
        Ok(())
    }
}

impl Serializable for BlockLimits {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        cell.append_u8(BLOCK_LIMITS_TAG)?;
        self.bytes.write_to(cell)?;
        self.gas.write_to(cell)?;
        self.lt_delta.write_to(cell)?;
        Ok(())
    }
}

type ConfigParam22 = BlockLimits;
type ConfigParam23 = BlockLimits;

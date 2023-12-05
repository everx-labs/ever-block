/*
* Copyright (C) 2019-2023 EverX. All Rights Reserved.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use ton_types::{
    BuilderData, Cell, error,
    fail,
    HashmapE, HashmapType, IBitstring, Result, SliceData, UInt256, HashmapIterator,
};

use crate::{
    define_HashmapE,
    error::BlockError,
    hashmapaug::HashmapAugType,
    shard::ShardIdent,
    shard_accounts::ShardAccounts,
    signature::{CryptoSignature, SigPubKey},
    types::{ChildCell, ExtraCurrencyCollection, Grams, Number8, Number12, Number16, Number13, Number32},
    validators::{ValidatorDescr, ValidatorSet},
    Serializable, Deserializable,
};

#[cfg(test)]
#[path = "tests/test_config_params.rs"]
mod tests;

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
        Self::new()
    }
}

impl ConfigParams {
    /// create new instance ConfigParams
    pub const fn new() -> Self {
        Self {
            config_addr: UInt256::default(),
            config_params: HashmapE::with_bit_len(32)
        }
    }

    pub const fn with_root(data: Cell) -> Self {
        Self {
            config_addr: UInt256::ZERO,
            config_params: HashmapE::with_hashmap(32, Some(data))
        }
    }

    pub const fn with_address_and_root(config_addr: UInt256, data: Cell) -> Self {
        Self {
            config_addr,
            config_params: HashmapE::with_hashmap(32, Some(data))
        }
    }

    pub const fn with_address_and_params(config_addr: UInt256, data: Option<Cell>) -> Self {
        Self {
            config_addr,
            config_params: HashmapE::with_hashmap(32, data)
        }
    }

    /// get config by index
    pub fn config(&self, index: u32) -> Result<Option<ConfigParamEnum>> {
        let key = SliceData::load_bitstring(index.write_to_new_cell()?)?;
        if let Some(slice) = self.config_params.get(key)? {
            if let Some(cell) = slice.reference_opt(0) {
                return Ok(Some(ConfigParamEnum::construct_from_slice_and_number(&mut SliceData::load_cell(cell)?, index)?));
            }
        }
        Ok(None)
    }

    /// get config by index
    pub fn config_present(&self, index: u32) -> Result<bool> {
        let key = SliceData::load_bitstring(index.write_to_new_cell()?)?;
        if let Some(slice) = self.config_params.get(key)? {
            if slice.remaining_references() != 0 {
                return Ok(true)
            }
        }
        Ok(false)
    }

    /// set config
    pub fn set_config(&mut self, config: ConfigParamEnum) -> Result<()> {

        let mut value = BuilderData::new();
        let index = config.write_to_cell(&mut value)?;
        let key = SliceData::load_bitstring(index.write_to_new_cell()?)?;
        self.config_params.set_builder(key, &value)?;
        Ok(())
    }

    pub fn get_smc_tick_tock(&self, smc_addr: &UInt256, accounts: &ShardAccounts) -> Result<usize> {
        let account = match accounts.get(smc_addr)? {
            Some(shard_account) => shard_account.read_account()?,
            None => fail!("Tick-tock smartcontract not found")
        };
        Ok(account.get_tick_tock().map(|tick_tock| tick_tock.as_usize()).unwrap_or_default())
    }

    pub fn special_ticktock_smartcontracts(&self, tick_tock: usize, accounts: &ShardAccounts) -> Result<Vec<(UInt256, usize)>> {
        let mut vec = Vec::new();
        self.fundamental_smc_addr()?.iterate_keys(|key: UInt256| {
            let tt = self.get_smc_tick_tock(&key, accounts)?;
            if (tick_tock & tt) != 0 {
                vec.push((key, tt))
            }
            Ok(true)
        })?;
        Ok(vec)
    }

    //
    // Wrappers
    //
    pub fn config_address(&self) -> Result<UInt256> {
        match self.config(0)? {
            Some(ConfigParamEnum::ConfigParam0(param)) => Ok(param.config_addr),
            _ => fail!("no config smc address in config")
        }
    }
    pub fn elector_address(&self) -> Result<UInt256> {
        match self.config(1)? {
            Some(ConfigParamEnum::ConfigParam1(param)) => Ok(param.elector_addr),
            _ => fail!("no elector address in config")
        }
    }
    pub fn minter_address(&self) -> Result<UInt256> {
        let addr = match self.config(2)? {
            Some(ConfigParamEnum::ConfigParam2(param)) => param.minter_addr,
            _ => match self.config(0)? {
                Some(ConfigParamEnum::ConfigParam0(param)) => param.config_addr,
                _ => fail!("no minter address in config")
            }
        };
        Ok(addr)
    }
    pub fn fee_collector_address(&self) -> Result<UInt256> {
        let addr = match self.config(3)? {
            Some(ConfigParamEnum::ConfigParam3(param)) => param.fee_collector_addr,
            _ => match self.config(1)? {
                Some(ConfigParamEnum::ConfigParam1(param)) => param.elector_addr,
                _ => fail!("no fee collector address in config")
            }
        };
        Ok(addr)
    }
    // TODO 4 dns_root_addr
    pub fn mint_prices(&self) -> Result<ConfigParam6> {
        match self.config(6)? {
            Some(ConfigParamEnum::ConfigParam6(cp)) => Ok(cp),
            _ => fail!("no config 6 (mint prices)")
        }
    }
    pub fn to_mint(&self) -> Result<ExtraCurrencyCollection> {
        match self.config(7)? {
            Some(ConfigParamEnum::ConfigParam7(cp)) => Ok(cp.to_mint),
            _ => fail!("no config 7 (to mint)")
        }
    }
    pub fn get_global_version(&self) -> Result<GlobalVersion> {
        match self.config(8)? {
            Some(ConfigParamEnum::ConfigParam8(gb)) => Ok(gb.global_version),
            _ => fail!("no global version in config")
        }
    }
    pub fn mandatory_params(&self) -> Result<MandatoryParams> {
        match self.config(9)? {
            Some(ConfigParamEnum::ConfigParam9(mp)) => Ok(mp.mandatory_params),
            _ => fail!("no mandatory params in config")
        }
    }
    // TODO 11 ConfigVotingSetup
    pub fn workchains(&self) -> Result<Workchains> {
        match self.config(12)? {
            Some(ConfigParamEnum::ConfigParam12(param)) => Ok(param.workchains),
            _ => fail!("Workchains not found in config")
        }
    }
    // TODO 13 compliant pricing
    pub fn block_create_fees(&self, masterchain: bool) -> Result<Grams> {
        match self.config(14)? {
            Some(ConfigParamEnum::ConfigParam14(param)) => if masterchain {
                Ok(param.block_create_fees.masterchain_block_fee)
            } else {
                Ok(param.block_create_fees.basechain_block_fee)
            }
            _ => fail!("no block create fee parameter")
        }
    }
    pub fn elector_params(&self) -> Result<ConfigParam15> {
        match self.config(15)? {
            Some(ConfigParamEnum::ConfigParam15(param)) => Ok(param),
            _ => fail!("no elector params in config")
        }
    }
    pub fn validators_count(&self) -> Result<ConfigParam16> {
        match self.config(16)? {
            Some(ConfigParamEnum::ConfigParam16(param)) => Ok(param),
            _ => fail!("no elector params in config")
        }
    }
    pub fn stakes_config(&self) -> Result<ConfigParam17> {
        match self.config(17)? {
            Some(ConfigParamEnum::ConfigParam17(param)) => Ok(param),
            _ => fail!("no stakes params in config")
        }
    }
    // TODO 16 validators count
    // TODO 17 stakes config
    pub fn storage_prices(&self) -> Result<ConfigParam18> {
        match self.config(18)? {
            Some(ConfigParamEnum::ConfigParam18(param)) => Ok(param),
            _ => fail!("Storage prices not found")
        }
    }
    pub fn gas_prices(&self, is_masterchain: bool) -> Result<GasLimitsPrices> {
        if is_masterchain {
            if let Some(ConfigParamEnum::ConfigParam20(param)) = self.config(20)? {
                return Ok(param)
            }
        } else if let Some(ConfigParamEnum::ConfigParam21(param)) = self.config(21)? {
            return Ok(param)
        }
        fail!("Gas prices not found")
    }
    pub fn block_limits(&self, masterchain: bool) -> Result<BlockLimits> {
        if masterchain {
            if let Some(ConfigParamEnum::ConfigParam22(param)) = self.config(22)? {
                return Ok(param)
            }
        } else if let Some(ConfigParamEnum::ConfigParam23(param)) = self.config(23)? {
            return Ok(param)
        }
        fail!("BlockLimits not found")
    }
    pub fn fwd_prices(&self, is_masterchain: bool) -> Result<MsgForwardPrices> {
        if is_masterchain {
            if let Some(ConfigParamEnum::ConfigParam24(param)) = self.config(24)? {
                return Ok(param)
            }
        } else if let Some(ConfigParamEnum::ConfigParam25(param)) = self.config(25)? {
            return Ok(param)
        }
        fail!("Forward prices not found")
    }
    pub fn catchain_config(&self) -> Result<CatchainConfig> {
        match self.config(28)? {
            Some(ConfigParamEnum::ConfigParam28(ccc)) => Ok(ccc),
            _ => fail!("no CatchainConfig in config_params")
        }
    }
    pub fn consensus_config(&self) -> Result<ConsensusConfig> {
        match self.config(29)? {
            Some(ConfigParamEnum::ConfigParam29(ConfigParam29{ consensus_config})) => Ok(consensus_config),
            _ => fail!("no ConsensusConfig in config_params")
        }
    }
    // TODO 29 consensus config
    pub fn fundamental_smc_addr(&self) -> Result<FundamentalSmcAddresses> {
        match self.config(31)? {
            Some(ConfigParamEnum::ConfigParam31(param)) => Ok(param.fundamental_smc_addr),
            _ => fail!("fundamental_smc_addr not found in config")
        }
    }
    pub fn delector_parameters(&self) -> Result<DelectorParams> {
        match self.config(30)? {
            Some(ConfigParamEnum::ConfigParam30(param)) => Ok(param),
            _ => fail!("delector parameters not found in config")
        }
    }
    pub fn prev_validator_set(&self) -> Result<ValidatorSet> {
        let vset = match self.config(33)? {
            Some(ConfigParamEnum::ConfigParam33(param)) => param.prev_temp_validators,
            _ => match self.config(32)? {
                Some(ConfigParamEnum::ConfigParam32(param)) => param.prev_validators,
                _ => ValidatorSet::default()
            }
        };
        Ok(vset)
    }
    pub fn prev_validator_set_present(&self) -> Result<bool> {
        Ok(self.config_present(33)? || self.config_present(32)?)
    }
    pub fn validator_set(&self) -> Result<ValidatorSet> {
        let vset = match self.config(35)? {
            Some(ConfigParamEnum::ConfigParam35(param)) => param.cur_temp_validators,
            _ => match self.config(34)? {
                Some(ConfigParamEnum::ConfigParam34(param)) => param.cur_validators,
                _ => fail!("no validator set in config")
            }
        };
        Ok(vset)
    }
    pub fn next_validator_set(&self) -> Result<ValidatorSet> {
        let vset = match self.config(37)? {
            Some(ConfigParamEnum::ConfigParam37(param)) => param.next_temp_validators,
            _ => match self.config(36)? {
                Some(ConfigParamEnum::ConfigParam36(param)) => param.next_validators,
                _ => ValidatorSet::default()
            }
        };
        Ok(vset)
    }
    pub fn next_validator_set_present(&self) -> Result<bool> {
        Ok(self.config_present(37)? || self.config_present(36)?)
    }
    pub fn read_cur_validator_set_and_cc_conf(&self) -> Result<(ValidatorSet, CatchainConfig)> {
        Ok((
            self.validator_set()?,
            self.catchain_config()?
        ))
    }
    pub fn copyleft_config(&self) -> Result<ConfigCopyleft> {
        match self.config(42)? {
            Some(ConfigParamEnum::ConfigParam42(cp)) => Ok(cp),
            _ => fail!("no config 42 (copyleft)")
        }
    }
    pub fn suspended_addresses(&self) -> Result<Option<SuspendedAddresses>> {
        match self.config(44)? {
            Some(ConfigParamEnum::ConfigParam44(sa)) => Ok(Some(sa)),
            None => Ok(None),
            _ =>  fail!("wrong config 44 (suspended addresses)")
        }
    }
    // TODO 39 validator signed temp keys
}

#[derive(Clone, Copy, Debug)]
#[repr(u64)]
pub enum GlobalCapabilities {
    CapNone                   = 0,
    CapIhrEnabled             = 0x0000_0000_0001,
    CapCreateStatsEnabled     = 0x0000_0000_0002,
    CapBounceMsgBody          = 0x0000_0000_0004,
    CapReportVersion          = 0x0000_0000_0008,
    CapSplitMergeTransactions = 0x0000_0000_0010,
    CapShortDequeue           = 0x0000_0000_0020,
    CapMbppEnabled            = 0x0000_0000_0040,
    CapFastStorageStat        = 0x0000_0000_0080,
    CapInitCodeHash           = 0x0000_0000_0100,
    CapOffHypercube           = 0x0000_0000_0200,
    CapMycode                 = 0x0000_0000_0400,
    CapSetLibCode             = 0x0000_0000_0800,
    CapFixTupleIndexBug       = 0x0000_0000_1000,
    CapRemp                   = 0x0000_0000_2000,
    CapDelections             = 0x0000_0000_4000,
    CapFullBodyInBounced      = 0x0000_0001_0000,
    CapStorageFeeToTvm        = 0x0000_0002_0000,
    CapCopyleft               = 0x0000_0004_0000,
    CapIndexAccounts          = 0x0000_0008_0000,
    #[cfg(feature = "gosh")]
    CapDiff                   = 0x0000_0010_0000,
    CapsTvmBugfixes2022       = 0x0000_0020_0000, // popsave, exception handler, loops
    CapWorkchains             = 0x0000_0040_0000,
    CapStcontNewFormat        = 0x0000_0080_0000, // support old format continuation serialization
    CapFastStorageStatBugfix  = 0x0000_0100_0000, // calc cell datasize using fast storage stat
    CapResolveMerkleCell      = 0x0000_0200_0000,
    #[cfg(feature = "signature_with_id")]
    CapSignatureWithId        = 0x0000_0400_0000, // use some predefined id during signature check
    CapBounceAfterFailedAction= 0x0000_0800_0000,
    #[cfg(feature = "groth")]
    CapGroth16                = 0x0000_1000_0000,
    CapFeeInGasUnits          = 0x0000_2000_0000, // all fees in config are in gas units
    CapBigCells               = 0x0000_4000_0000,
    CapSuspendedList          = 0x0000_8000_0000,
    CapFastFinality           = 0x0001_0000_0000,
    CapTvmV19                 = 0x0002_0000_0000, // TVM v1.9.x improvemements
    CapSmft                   = 0x0004_0000_0000,
    CapCommonMessage          = 0x0008_0000_0000,
}

impl ConfigParams {
    pub fn get_lt_align(&self) -> u64 {
        1_000_000
    }
    pub fn get_max_lt_growth_fast_finality(&self) -> u64 {
        100 * self.get_lt_align() - 1
    }
    pub fn get_max_lt_growth(&self) -> u64 {
        10 * self.get_lt_align() - 1
    }
    pub fn get_next_block_lt(&self, prev_block_lt: u64) -> u64 {
        (prev_block_lt / self.get_lt_align() + 1) * self.get_lt_align()
    }
    pub fn has_capabilities(&self) -> bool {
        match self.get_global_version() {
            Ok(gb) => gb.capabilities != 0,
            Err(_) => false
        }
    }
    pub fn has_capability(&self, capability: GlobalCapabilities) -> bool {
        match self.get_global_version() {
            Ok(gb) => gb.has_capability(capability),
            Err(_) => false
        }
    }
    pub fn capabilities(&self) -> u64 {
        match self.get_global_version() {
            Ok(gb) => gb.capabilities,
            Err(_) => 0
        }
    }
    pub fn global_version(&self) -> u32 {
        self.get_global_version().map_or(0, |gb| gb.version)
    }
}

impl ConfigParams {
    pub fn compute_validator_set_cc(&self, shard: &ShardIdent, at: u32, cc_seqno: u32, cc_seqno_delta: &mut u32) -> Result<Vec<ValidatorDescr>> {
        let (vset, ccc) = self.read_cur_validator_set_and_cc_conf()?;
        if (*cc_seqno_delta & 0xfffffffe) != 0 {
            fail!("seqno_delta>1 is not implemented yet");
        }
        *cc_seqno_delta += cc_seqno;
        vset.calc_subset(&ccc, shard.shard_prefix_with_tag(), shard.workchain_id(), *cc_seqno_delta, at.into())
            .map(|(set, _hash)| {
                set
            })
    }
    pub fn compute_validator_set(&self, shard: &ShardIdent, _at: u32, cc_seqno: u32) -> Result<Vec<ValidatorDescr>> {
        let (vset, ccc) = self.read_cur_validator_set_and_cc_conf()?;
        vset.calc_subset(&ccc, shard.shard_prefix_with_tag(), shard.workchain_id(), cc_seqno, _at.into())
            .map(|(set, _seq_no)| set)
    }
}

const MANDATORY_CONFIG_PARAMS: [u32; 9] = [18, 20, 21, 22, 23, 24, 25, 28, 34];

impl ConfigParams {
    pub fn valid_config_data(&self, relax_par0: bool, mparams: Option<MandatoryParams>) -> Result<bool> {
        if !relax_par0 {
            match self.config(0) {
                Ok(Some(ConfigParamEnum::ConfigParam0(param))) if param.config_addr == self.config_addr => (),
                _ => return Ok(false)
            }
        }
        // porting from Durov's code
        // previously was not 9 parameter in config params
        for index in &MANDATORY_CONFIG_PARAMS {
            if self.config(*index)?.is_none() {
                log::error!(target: "block", "configuration parameter #{} \
                    (hardcoded as mandatory) is missing)", index);
                return Ok(false)
            }
        }
        let result = match self.config(9) {
            Ok(Some(ConfigParamEnum::ConfigParam9(param))) => self.config_params_present(Some(param.mandatory_params))?,
            _ => {
                log::error!(target: "block", "invalid mandatory parameters dictionary while checking \
                    existence of all mandatory configuration parameters");
                false
            }
        };
        Ok(result && self.config_params_present(mparams)?)
    }
    fn config_params_present(&self, params: Option<MandatoryParams>) -> Result<bool> {
        match params {
            Some(params) => params.iterate_keys(|index: u32| match self.config(index) {
                Ok(Some(_)) => Ok(true),
                _ => {
                    log::error!(target: "block", "configuration parameter #{} \
                        (declared as mandatory in configuration parameter #9) is missing)", index);
                    Ok(false)
                }
            }),
            None => Ok(true)
        }
    }
    // when these parameters change, the block must be marked as a key block
    pub fn important_config_parameters_changed(&self, other: &ConfigParams, coarse: bool) -> Result<bool> {
        if self.config_params == other.config_params {
            return Ok(false)
        }
        if coarse {
            return Ok(true)
        }
        // for now, all parameters are "important"
        // at least the parameters affecting the computations of validator sets must be considered important
        // ...
        Ok(true)
    }
}

impl Deserializable for ConfigParams {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.config_addr.read_from(cell)?;
        *self.config_params.data_mut() = Some(cell.checked_drain_reference()?);
        Ok(())
    }
}


impl Serializable for ConfigParams {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.checked_append_reference(self.config_params.data().cloned().unwrap_or_default())?;
        self.config_addr.write_to(cell)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfigParamEnum {
    ConfigParam0(ConfigParam0),
    ConfigParam1(ConfigParam1),
    ConfigParam2(ConfigParam2),
    ConfigParam3(ConfigParam3),
    ConfigParam4(ConfigParam4),
    ConfigParam5(ConfigParam5),
    ConfigParam6(ConfigParam6),
    ConfigParam7(ConfigParam7),
    ConfigParam8(ConfigParam8),
    ConfigParam9(ConfigParam9),
    ConfigParam10(ConfigParam10),
    ConfigParam11(ConfigParam11),
    ConfigParam12(ConfigParam12),
    ConfigParam13(ConfigParam13),
    ConfigParam14(ConfigParam14),
    ConfigParam15(ConfigParam15),
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
    ConfigParam29(ConfigParam29),
    ConfigParam30(DelectorParams),
    ConfigParam31(ConfigParam31),
    ConfigParam32(ConfigParam32),
    ConfigParam33(ConfigParam33),
    ConfigParam34(ConfigParam34),
    ConfigParam35(ConfigParam35),
    ConfigParam36(ConfigParam36),
    ConfigParam37(ConfigParam37),
    ConfigParam39(ConfigParam39),
    ConfigParam40(ConfigParam40),
    ConfigParam42(ConfigCopyleft),
    ConfigParam44(SuspendedAddresses),
    ConfigParamAny(u32, SliceData),
}

macro_rules! read_config {
    ( $cpname:ident, $cname:ident, $slice:expr ) => {
        {
            let c = $cname::construct_from($slice)?;
            Ok(ConfigParamEnum::$cpname(c))
        }
    }
}

impl ConfigParamEnum {
    
    pub fn construct_from_cell_and_number(cell: Cell, index: u32) -> Result<ConfigParamEnum> {
        Self::construct_from_slice_and_number(&mut SliceData::load_cell(cell)?, index)
    }

    /// read config from cell
    pub fn construct_from_slice_and_number(slice: &mut SliceData, index: u32) -> Result<ConfigParamEnum> {
        match index {
            0 => { read_config!(ConfigParam0, ConfigParam0, slice) },
            1 => { read_config!(ConfigParam1, ConfigParam1, slice) },
            2 => { read_config!(ConfigParam2, ConfigParam2, slice) },
            3 => { read_config!(ConfigParam3, ConfigParam3, slice) },
            4 => { read_config!(ConfigParam4, ConfigParam4, slice) },
            5 => { read_config!(ConfigParam5, ConfigParam5, slice) },
            6 => { read_config!(ConfigParam6, ConfigParam6, slice) },
            7 => { read_config!(ConfigParam7, ConfigParam7, slice) },
            8 => { read_config!(ConfigParam8, ConfigParam8, slice) },
            9 => { read_config!(ConfigParam9, ConfigParam9, slice) },
            10 => { read_config!(ConfigParam10, ConfigParam10, slice) },
            11 => { read_config!(ConfigParam11, ConfigParam11, slice) },
            12 => { read_config!(ConfigParam12, ConfigParam12, slice) },
            13 => { read_config!(ConfigParam13, ConfigParam13, slice) },
            14 => { read_config!(ConfigParam14, ConfigParam14, slice) },
            15 => { read_config!(ConfigParam15, ConfigParam15, slice) },
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
            29 => { read_config!(ConfigParam29, ConfigParam29, slice) },
            30 => { read_config!(ConfigParam30, DelectorParams, slice) },
            31 => { read_config!(ConfigParam31, ConfigParam31, slice) },
            32 => { read_config!(ConfigParam32, ConfigParam32, slice) },
            33 => { read_config!(ConfigParam33, ConfigParam33, slice) },
            34 => { read_config!(ConfigParam34, ConfigParam34, slice) },
            35 => { read_config!(ConfigParam35, ConfigParam35, slice) },
            36 => { read_config!(ConfigParam36, ConfigParam36, slice) },
            37 => { read_config!(ConfigParam37, ConfigParam37, slice) },
            39 => { read_config!(ConfigParam39, ConfigParam39, slice) },
            40 => { read_config!(ConfigParam40, ConfigParam40, slice) },
            42 => { read_config!(ConfigParam42, ConfigCopyleft, slice) },
            44 => { read_config!(ConfigParam44, SuspendedAddresses, slice) },
            index => Ok(ConfigParamEnum::ConfigParamAny(index, slice.clone())),
        }
    }

    /// Save config to cell
    pub fn write_to_cell(&self, cell: &mut BuilderData) -> Result<u32> {
        match self {
            ConfigParamEnum::ConfigParam0(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(0)},
            ConfigParamEnum::ConfigParam1(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(1)},
            ConfigParamEnum::ConfigParam2(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(2)},
            ConfigParamEnum::ConfigParam3(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(3)},
            ConfigParamEnum::ConfigParam4(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(4)},
            ConfigParamEnum::ConfigParam5(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(5)},
            ConfigParamEnum::ConfigParam6(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(6)},
            ConfigParamEnum::ConfigParam7(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(7)},
            ConfigParamEnum::ConfigParam8(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(8)},
            ConfigParamEnum::ConfigParam9(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(9)},
            ConfigParamEnum::ConfigParam10(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(10)},
            ConfigParamEnum::ConfigParam11(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(11)},
            ConfigParamEnum::ConfigParam12(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(12)},
            ConfigParamEnum::ConfigParam13(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(13)},
            ConfigParamEnum::ConfigParam14(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(14)},
            ConfigParamEnum::ConfigParam15(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(15)},
            ConfigParamEnum::ConfigParam16(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(16)},
            ConfigParamEnum::ConfigParam17(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(17)},
            ConfigParamEnum::ConfigParam18(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(18)},
            ConfigParamEnum::ConfigParam20(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(20)},
            ConfigParamEnum::ConfigParam21(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(21)},
            ConfigParamEnum::ConfigParam22(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(22)},
            ConfigParamEnum::ConfigParam23(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(23)},
            ConfigParamEnum::ConfigParam24(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(24)},
            ConfigParamEnum::ConfigParam25(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(25)},
            ConfigParamEnum::ConfigParam28(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(28)},
            ConfigParamEnum::ConfigParam29(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(29)},
            ConfigParamEnum::ConfigParam30(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(30)},
            ConfigParamEnum::ConfigParam31(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(31)},
            ConfigParamEnum::ConfigParam32(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(32)},
            ConfigParamEnum::ConfigParam33(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(33)},
            ConfigParamEnum::ConfigParam34(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(34)},
            ConfigParamEnum::ConfigParam35(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(35)},
            ConfigParamEnum::ConfigParam36(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(36)},
            ConfigParamEnum::ConfigParam37(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(37)},
            ConfigParamEnum::ConfigParam39(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(39)},
            ConfigParamEnum::ConfigParam40(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(40)},
            ConfigParamEnum::ConfigParam42(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(42)},
            ConfigParamEnum::ConfigParam44(ref c) => { cell.checked_append_reference(c.serialize()?)?; Ok(44)},
            ConfigParamEnum::ConfigParamAny(index, slice) => { 
                cell.checked_append_reference(slice.clone().into_cell())?; 
                Ok(*index)
            },
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
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.config_addr.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam0 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
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
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.elector_addr.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam1 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.elector_addr.write_to(cell)?;
        Ok(())
    }
}

///
/// Config Param 2 structure
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ConfigParam2 {
    pub minter_addr: UInt256,
}

impl ConfigParam2 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deserializable for ConfigParam2 {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.minter_addr.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam2 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.minter_addr.write_to(cell)?;
        Ok(())
    }
}

///
/// Config Param 3 structure
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ConfigParam3 {
    pub fee_collector_addr: UInt256,
}

impl ConfigParam3 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deserializable for ConfigParam3 {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.fee_collector_addr.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam3 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.fee_collector_addr.write_to(cell)?;
        Ok(())
    }
}

///
/// Config Param 4 structure
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ConfigParam4 {
    pub dns_root_addr: UInt256,
}

impl ConfigParam4 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deserializable for ConfigParam4 {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.dns_root_addr.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam4 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.dns_root_addr.write_to(cell)?;
        Ok(())
    }
}

///
/// Config Param 5 structure
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ConfigParam5 {
    pub owner_addr: UInt256,
}

impl Deserializable for ConfigParam5 {
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        let owner_addr = Deserializable::construct_from(slice)?;
        Ok(ConfigParam5 { owner_addr })
    }
}

impl Serializable for ConfigParam5 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.owner_addr.write_to(cell)?;
        Ok(())
    }
}

///
/// Config Param 6 structure
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ConfigParam6 {
    pub mint_new_price: Grams,
    pub mint_add_price: Grams,
}

impl ConfigParam6 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deserializable for ConfigParam6 {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.mint_new_price.read_from(cell)?;
        self.mint_add_price.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam6 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.mint_new_price.write_to(cell)?;
        self.mint_add_price.write_to(cell)?;
        Ok(())
    }
}

///
/// Config Param 7 structure
/// 
#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct ConfigParam7 {
    pub to_mint: ExtraCurrencyCollection,
}

impl ConfigParam7 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deserializable for ConfigParam7 {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.to_mint.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam7 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.to_mint.write_to(cell)?;
        Ok(())
    }
}

///
/// Config Param 8 structure
/// 
// capabilities#c4 version:uint32 capabilities:uint64 = GlobalVersion;
// _ GlobalVersion = ConfigParam 8;  // all zero if absent

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct GlobalVersion {
    pub version: u32,
    pub capabilities: u64,
}

impl GlobalVersion {
    pub const fn new() -> Self {
        GlobalVersion {
            version: 0,
            capabilities: 0
        }
    }
    pub fn has_capability(&self, capability: GlobalCapabilities) -> bool {
        (self.capabilities & (capability as u64)) != 0
    }
}

const GLOBAL_VERSION_TAG: u8 = 0xC4;

impl Deserializable for GlobalVersion {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag = cell.get_next_byte()?;
        if tag != GLOBAL_VERSION_TAG {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: std::any::type_name::<Self>().to_string()
                }
            )
        }
        self.version.read_from(cell)?;
        self.capabilities.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for GlobalVersion {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u8(GLOBAL_VERSION_TAG)?;
        self.version.write_to(cell)?;
        self.capabilities.write_to(cell)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ConfigParam8 {
    pub global_version: GlobalVersion
}

impl ConfigParam8 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deserializable for ConfigParam8 {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.global_version.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam8 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.global_version.write_to(cell)?;
        Ok(())
    }
}

// _ mandatory_params:(Hashmap 32 True) = ConfigParam 9;

define_HashmapE!{MandatoryParams, 32, ()}

///
/// Config Param 9 structure
/// 
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConfigParam9 {
    pub mandatory_params: MandatoryParams,
}

impl ConfigParam9 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deserializable for ConfigParam9 {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.mandatory_params.read_hashmap_root(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam9 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.mandatory_params.write_hashmap_root(cell)?;
        Ok(())
    }
}

///
/// Config Param 10 structure
/// 
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConfigParam10 {
    pub critical_params: MandatoryParams,
}

impl ConfigParam10 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deserializable for ConfigParam10 {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.critical_params.read_hashmap_root(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam10 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.critical_params.write_hashmap_root(cell)?;
        Ok(())
    }
}

///
/// Config Param 14 structure
/// 
// block_grams_created#6b masterchain_block_fee:Grams basechain_block_fee:Grams
//   = BlockCreateFees;
// _ BlockCreateFees = ConfigParam 14;

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct BlockCreateFees {
    pub masterchain_block_fee: Grams,
    pub basechain_block_fee: Grams,
}

impl BlockCreateFees {
    pub fn new() -> Self {
        Self::default()
    }
}

const BLOCK_CREATE_FEES: u8 = 0x6b;

impl Deserializable for BlockCreateFees {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag = cell.get_next_byte()?;
        if tag != BLOCK_CREATE_FEES {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: std::any::type_name::<Self>().to_string()
                }
            )
        }
        self.masterchain_block_fee.read_from(cell)?;
        self.basechain_block_fee.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for BlockCreateFees {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u8(BLOCK_CREATE_FEES)?;
        self.masterchain_block_fee.write_to(cell)?;
        self.basechain_block_fee.write_to(cell)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ConfigParam14 {
    pub block_create_fees: BlockCreateFees
}

impl ConfigParam14 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deserializable for ConfigParam14 {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.block_create_fees.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam14 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.block_create_fees.write_to(cell)?;
        Ok(())
    }
}

///
/// Config Param 15 structure
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ConfigParam15 {
    pub validators_elected_for: u32,
    pub elections_start_before: u32,
    pub elections_end_before: u32,
    pub stake_held_for: u32,
}

impl ConfigParam15 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deserializable for ConfigParam15 {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.validators_elected_for.read_from(cell)?;
        self.elections_start_before.read_from(cell)?;
        self.elections_end_before.read_from(cell)?;
        self.stake_held_for.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam15 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.validators_elected_for.write_to(cell)?;
        self.elections_start_before.write_to(cell)?;
        self.elections_end_before.write_to(cell)?;
        self.stake_held_for.write_to(cell)?;
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
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.max_validators.read_from(cell)?;
        self.max_main_validators.read_from(cell)?;
        self.min_validators.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam16 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.max_validators.write_to(cell)?;
        self.max_main_validators.write_to(cell)?;
        self.min_validators.write_to(cell)?;
        Ok(())
    }
}

/*
_ 
    min_stake: Grams 
    max_stake: Grams 
    min_total_stake: Grams 
    max_stake_factor: uint32 
= ConfigParam 17;
*/

///
/// Config Param 17 structure
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ConfigParam17 {
    pub min_stake: Grams,
    pub max_stake: Grams,
    pub min_total_stake: Grams,
    pub max_stake_factor: u32,
}

impl ConfigParam17 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deserializable for ConfigParam17 {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.min_stake.read_from(cell)?;
        self.max_stake.read_from(cell)?;
        self.min_total_stake.read_from(cell)?;
        self.max_stake_factor.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam17 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.min_stake.write_to(cell)?;
        self.max_stake.write_to(cell)?;
        self.min_total_stake.write_to(cell)?;
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
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag = cell.get_next_byte()?;
        if tag != STORAGE_PRICES_TAG {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: std::any::type_name::<Self>().to_string()
                }
            )
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
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u8(STORAGE_PRICES_TAG)?;
        self.utime_since.write_to(cell)?;
        self.bit_price_ps.write_to(cell)?;
        self.cell_price_ps.write_to(cell)?;
        self.mc_bit_price_ps.write_to(cell)?;
        self.mc_cell_price_ps.write_to(cell)?;
        Ok(())
    }
}

define_HashmapE!(ConfigParam18Map, 32, StoragePrices);

///
/// ConfigParam 18 struct
/// 
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConfigParam18 {
    pub map: ConfigParam18Map,
}

impl ConfigParam18 {
    /// get length
    pub fn len(&self) -> Result<usize> {
        self.map.len()
    } 

    /// determine is empty
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    } 

    /// get value by index
    pub fn get(&self, index: u32) -> Result<StoragePrices> {
        self.map.get(&index)?.ok_or_else(|| error!(BlockError::InvalidIndex(index as usize)))
    }

    /// insert value
    pub fn insert(&mut self, sp: &StoragePrices) -> Result<()> {
        let index = match self.map.0.get_max(false, &mut 0)? {
            Some((key, _value)) => SliceData::load_bitstring(key)?.get_next_u32()? + 1,
            None => 0
        };
        self.map.set(&index, sp)
    }
}


impl Deserializable for ConfigParam18 {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        self.map.read_hashmap_root(slice)?;
        Ok(())
    }
}

impl Serializable for ConfigParam18 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        if self.map.is_empty() {
            fail!(BlockError::InvalidOperation("self.map is empty".to_string()))
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

gas_flat_pfx#d1
  flat_gas_limit:uint64
  flat_gas_price:uint64
  other:GasLimitsPrices
= GasLimitsPrices;
*/

///
/// GasLimitsPrices
/// 
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GasLimitsPrices {
    pub gas_price: u64,
    pub gas_limit: u64,
    pub special_gas_limit: u64,
    pub gas_credit: u64,
    pub block_gas_limit: u64,
    pub freeze_due_limit: u64,
    pub delete_due_limit: u64,
    pub flat_gas_limit: u64,
    pub flat_gas_price: u64,
    pub max_gas_threshold: u128,
}

impl GasLimitsPrices {
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate gas fee by gas used value
    pub fn calc_gas_fee(&self, gas_used: u64) -> u128 {
        // There is a flat_gas_limit value which is the minimum gas value possible and has fixed price.
        // If actual gas value is less then flat_gas_limit then flat_gas_price paid.
        // If actual gas value is bigger then flat_gas_limit then flat_gas_price paid for first 
        // flat_gas_limit gas and remaining value costs gas_price
        if gas_used <= self.flat_gas_limit {
            self.flat_gas_price as u128
        } else {
            // gas_price is pseudo value (shifted by 16 as forward and storage price)
            // after calculation divide by 0xffff with ceil rounding
            self.flat_gas_price as u128 + (((gas_used - self.flat_gas_limit) as u128 * self.gas_price as u128 + 0xffff) >> 16)
        }
    }

    /// Get gas price in nanograms
    pub fn get_real_gas_price(&self) -> u64 {
        self.gas_price >> 16
    }

    /// Calculate gas by grams balance
    pub fn calc_gas(&self, value: u128) -> u64 {
        if value >= self.max_gas_threshold {
            return self.gas_limit
        }
        if value < self.flat_gas_price as u128 {
            return 0
        }
        let res = ((value - self.flat_gas_price as u128) << 16) / (self.gas_price as u128);
        self.flat_gas_limit + res as u64
    }

    /// Calculate max gas threshold
    pub fn calc_max_gas_threshold(&self) -> u128 {
        let mut result = self.flat_gas_price as u128;
        if self.gas_limit > self.flat_gas_limit {
            result += ((self.gas_price as u128) * ((self.gas_limit - self.flat_gas_limit) as u128)) >> 16;
        }
        result
    }
}

const GAS_PRICES_TAG: u8 = 0xDD;
const GAS_PRICES_EXT_TAG: u8 = 0xDE;
const GAS_FLAT_PFX_TAG: u8 = 0xD1;

impl Deserializable for GasLimitsPrices {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.flat_gas_limit = 0;
        self.flat_gas_price = 0;
        self.special_gas_limit = 0;
        loop {
            match cell.get_next_byte()? {
                GAS_PRICES_TAG => {
                    self.gas_price.read_from(cell)?;
                    self.gas_limit.read_from(cell)?;
                    self.gas_credit.read_from(cell)?;
                    self.block_gas_limit.read_from(cell)?;
                    self.freeze_due_limit.read_from(cell)?;
                    self.delete_due_limit.read_from(cell)?;
                    break;
                }
                GAS_PRICES_EXT_TAG => {
                    self.gas_price.read_from(cell)?;
                    self.gas_limit.read_from(cell)?;
                    self.special_gas_limit.read_from(cell)?;
                    self.gas_credit.read_from(cell)?;
                    self.block_gas_limit.read_from(cell)?;
                    self.freeze_due_limit.read_from(cell)?;
                    self.delete_due_limit.read_from(cell)?;
                    break;
                }
                GAS_FLAT_PFX_TAG => {
                    self.flat_gas_limit.read_from(cell)?;
                    self.flat_gas_price.read_from(cell)?;
                }
                tag => {
                    fail!(
                        BlockError::InvalidConstructorTag {
                            t: tag as u32,
                            s: std::any::type_name::<Self>().to_string()
                        }
                    )
                }
            }
        }
        self.max_gas_threshold = self.calc_max_gas_threshold();
        Ok(())
    }
}

impl Serializable for GasLimitsPrices {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u8(GAS_FLAT_PFX_TAG)?;
        self.flat_gas_limit.write_to(cell)?;
        self.flat_gas_price.write_to(cell)?;
        cell.append_u8(GAS_PRICES_EXT_TAG)?;
        self.gas_price.write_to(cell)?;
        self.gas_limit.write_to(cell)?;
        self.special_gas_limit.write_to(cell)?;
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
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag = cell.get_next_byte()?;
        if tag != MSG_FWD_PRICES_TAG {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: std::any::type_name::<Self>().to_string()
                }
            )
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
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
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

catchain_config_new#c2
    flags: (## 7) 
    { flags = 0 } 
    shuffle_mc_validators: Bool
    mc_catchain_lifetime: uint3
    shard_catchain_lifetime: uint32
    shard_validators_lifetime: uint32 
    shard_validators_num: uint32 
= CatchainConfig;
*/

///
/// MsgForwardPrices
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct CatchainConfig {
    pub isolate_mc_validators: bool,
    pub shuffle_mc_validators: bool,
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

const CATCHAIN_CONFIG_TAG_1: u8 = 0xC1;
const CATCHAIN_CONFIG_TAG_2: u8 = 0xC2;

impl Deserializable for CatchainConfig {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag = cell.get_next_byte()?;
        if (tag != CATCHAIN_CONFIG_TAG_1) && (tag != CATCHAIN_CONFIG_TAG_2) {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: std::any::type_name::<Self>().to_string()
                }
            )
        }
        if tag == CATCHAIN_CONFIG_TAG_2 {
            let flags = u8::construct_from(cell)?;
            self.isolate_mc_validators = flags & 0b10 != 0;
            self.shuffle_mc_validators = flags & 0b01 != 0;
            if flags >> 2 != 0 {
                fail!(BlockError::InvalidArg("`flags` should be zero".to_string()))
            }
        }
        self.mc_catchain_lifetime.read_from(cell)?;
        self.shard_catchain_lifetime.read_from(cell)?;
        self.shard_validators_lifetime.read_from(cell)?;
        self.shard_validators_num.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for CatchainConfig {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u8(CATCHAIN_CONFIG_TAG_2)?;
        cell.append_bits(0, 6)?;
        cell.append_bit_bool(self.isolate_mc_validators)?;
        cell.append_bit_bool(self.shuffle_mc_validators)?;
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

consensus_config#d6
    round_candidates:# { round_candidates >= 1 }
    next_candidate_delay_ms:uint32
    consensus_timeout_ms:uint32
    fast_attempts:uint32
    attempt_duration:uint32
    catchain_max_deps:uint32
    max_block_bytes:uint32
    max_collated_bytes:uint32
= ConsensusConfig;

consensus_config_new#d7
    flags: (## 7)
    { flags = 0 }
    new_catchain_ids: Bool
    round_candidates: (## 8) { round_candidates >= 1 }
    next_candidate_delay_ms: uint32 
    consensus_timeout_ms: uint32
    fast_attempts: uint32
    attempt_duration: uint32
    catchain_max_deps: uint32
    max_block_bytes: uint32
    max_collated_bytes: uint32 
= ConsensusConfig;
*/

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ConsensusConfig {
    pub new_catchain_ids: bool,
    pub round_candidates: u32,
    pub next_candidate_delay_ms: u32,
    pub consensus_timeout_ms: u32,
    pub fast_attempts: u32,
    pub attempt_duration: u32,
    pub catchain_max_deps: u32,
    pub max_block_bytes: u32,
    pub max_collated_bytes: u32,
}

impl ConsensusConfig {
    pub fn new() -> Self {
        Self::default()
    }
}

const CONSENSUS_CONFIG_TAG_1: u8 = 0xD6;
const CONSENSUS_CONFIG_TAG_2: u8 = 0xD7;

impl Deserializable for ConsensusConfig {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag = cell.get_next_byte()?;
        if (tag != CONSENSUS_CONFIG_TAG_1) && (tag != CONSENSUS_CONFIG_TAG_2) {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: std::any::type_name::<Self>().to_string()
                }
            )
        }
        if tag == CONSENSUS_CONFIG_TAG_1 {
            self.round_candidates.read_from(cell)?;
        } else {
            let flags = u8::construct_from(cell)?;
            self.new_catchain_ids = flags == 1;
            if flags >> 1 != 0 {
                fail!(BlockError::InvalidArg("`flags` should be zero".to_string()))
            }
            self.round_candidates = u8::construct_from(cell)? as u32;
            if self.round_candidates == 0 {
                fail!(BlockError::InvalidArg("`round_candidates` should be positive".to_string()))
            }
        }
        self.next_candidate_delay_ms.read_from(cell)?;
        self.consensus_timeout_ms.read_from(cell)?;
        self.fast_attempts.read_from(cell)?;
        self.attempt_duration.read_from(cell)?;
        self.catchain_max_deps.read_from(cell)?;
        self.max_block_bytes.read_from(cell)?;
        self.max_collated_bytes.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConsensusConfig {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        if self.round_candidates == 0 {
            fail!(BlockError::InvalidArg("`round_candidates` should be positive".to_string()))
        }
        cell.append_u8(CONSENSUS_CONFIG_TAG_2)?;
        cell.append_u8(self.new_catchain_ids as u8)?;
        (self.round_candidates as u8).write_to(cell)?;
        self.next_candidate_delay_ms.write_to(cell)?;
        self.consensus_timeout_ms.write_to(cell)?;
        self.fast_attempts.write_to(cell)?;
        self.attempt_duration.write_to(cell)?;
        self.catchain_max_deps.write_to(cell)?;
        self.max_block_bytes.write_to(cell)?;
        self.max_collated_bytes.write_to(cell)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ConfigParam29 {
    pub consensus_config: ConsensusConfig,
}

impl ConfigParam29 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deserializable for ConfigParam29 {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.consensus_config.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam29 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.consensus_config.write_to(cell)?;
        Ok(())
    }
}

/*
_ fundamental_smc_addr:(HashmapE 256 True) = ConfigParam 31;
*/

define_HashmapE!{FundamentalSmcAddresses, 256, ()}

impl IntoIterator for &FundamentalSmcAddresses {
    type Item = <HashmapIterator<HashmapE> as std::iter::Iterator>::Item;
    type IntoIter = HashmapIterator<HashmapE>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

///
/// ConfigParam 30;
/// 

const DELECTOR_PARAMS_TAG: u8 = 0x1;
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DelectorParams {
    pub delections_step: u32,
    pub validator_init_code_hash: UInt256,
    pub staker_init_code_hash: UInt256,
}

impl DelectorParams {
    pub const fn new() -> Self {
        Self {
            delections_step: 0,
            validator_init_code_hash: UInt256::new(),
            staker_init_code_hash: UInt256::new(),
        }
    }
}

impl Deserializable for DelectorParams {
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        let tag = slice.get_next_byte()?;
        if tag != DELECTOR_PARAMS_TAG {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: std::any::type_name::<Self>().to_string()
                }
            )
        }
        let delections_step = Deserializable::construct_from(slice)?;
        let validator_init_code_hash = Deserializable::construct_from(slice)?;
        let staker_init_code_hash = Deserializable::construct_from(slice)?;
        Ok(Self {
            delections_step,
            validator_init_code_hash,
            staker_init_code_hash,
        })
    }
}

impl Serializable for DelectorParams {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        DELECTOR_PARAMS_TAG.write_to(cell)?; // tag
        self.delections_step.write_to(cell)?;
        self.validator_init_code_hash.write_to(cell)?;
        self.staker_init_code_hash.write_to(cell)?;
        Ok(())
    }
}

///
/// ConfigParam 31;
/// 
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConfigParam31 {
    pub fundamental_smc_addr: FundamentalSmcAddresses,
}

impl ConfigParam31 {
    pub const fn new() -> Self {
        Self {
            fundamental_smc_addr: FundamentalSmcAddresses::new()
        }
    }

    pub fn add_address(&mut self, address: UInt256) {
        self.fundamental_smc_addr.set(&address, &()).unwrap();
    }
}

impl Deserializable for ConfigParam31 {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.fundamental_smc_addr.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam31 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
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
            fn construct_from(slice: &mut SliceData) -> Result<Self> {
                let $pname = ValidatorSet::construct_from(slice)?;
                Ok(Self { $pname })
            }
        }

        impl Serializable for $cpname {
            fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
                self.$pname.write_to(cell)?;
                Ok(())
            }
        }
    }
}

// _ prev_validators:ValidatorSet = ConfigParam 32;
define_configparams!(ConfigParam32, prev_validators);

// _ prev_temp_validators: ValidatorSet = ConfigParam 33;
define_configparams!(ConfigParam33, prev_temp_validators);

// _ cur_validators:ValidatorSet = ConfigParam 34;
define_configparams!(ConfigParam34, cur_validators);

// _ cur_temp_validators: ValidatorSet = ConfigParam 35;
define_configparams!(ConfigParam35, cur_temp_validators);

//_ next_validators:ValidatorSet = ConfigParam 36;
define_configparams!(ConfigParam36, next_validators);

// _ next_temp_validators: ValidatorSet = ConfigParam 37;
define_configparams!(ConfigParam37, next_temp_validators);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkchainFormat {
    Basic(WorkchainFormat1),
    Extended(WorkchainFormat0),
}

impl Default for WorkchainFormat {
    fn default() -> Self {
        WorkchainFormat::Basic(WorkchainFormat1::default())
    }
}

impl Deserializable for WorkchainFormat {
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        slice.get_next_bits(3)?;
        match slice.get_next_bit()? {
            true => {
                Ok(WorkchainFormat::Basic(WorkchainFormat1::construct_from(slice)?))
            }
            false => {
                Ok(WorkchainFormat::Extended(WorkchainFormat0::construct_from(slice)?))
            }
        }
    }
}

impl Serializable for WorkchainFormat {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_bits(0, 3)?;
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
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        let vm_version = Deserializable::construct_from(slice)?;
        let vm_mode = Deserializable::construct_from(slice)?;
        Ok(Self { vm_version, vm_mode })
    }
}

impl Serializable for WorkchainFormat1 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
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
    min_addr_len: Number12,
    max_addr_len: Number12,
    addr_len_step: Number12,
    workchain_type_id: Number32,
}

impl Default for WorkchainFormat0 {
    fn default() -> Self {
        WorkchainFormat0::new()
    }
}

impl WorkchainFormat0 {
    ///
    /// Create empty new instance of WorkchainFormat0
    /// 
    pub fn new() -> Self {
        Self {
            min_addr_len: Number12::from(64),
            max_addr_len: Number12::from(64),
            addr_len_step: Number12::from(0),
            workchain_type_id: Number32::from(1)
        }
    }

    ///
    /// Create new instance of WorkchainFormat0
    /// 
    pub fn with_params(min_addr_len: u16, max_addr_len: u16, addr_len_step: u16, workchain_type_id: u32 ) -> Result<WorkchainFormat0> {
        if min_addr_len >= 64 && min_addr_len <= max_addr_len &&
           max_addr_len <= 1023 && addr_len_step <= 1023 &&
           workchain_type_id >= 1 {
               Ok(
                   WorkchainFormat0 {
                        min_addr_len: Number12::new(min_addr_len as u32)?,
                        max_addr_len: Number12::new(max_addr_len as u32)?,
                        addr_len_step: Number12::new(addr_len_step as u32)?,
                        workchain_type_id: Number32::new(workchain_type_id)?,
                   }
               )
           }
        else {
            fail!(
                BlockError::InvalidData(
                    "min_addr_len >= 64 && min_addr_len <= max_addr_len \
                     && max_addr_len <= 1023 && addr_len_step <= 1023 \
                     && workchain_type_id >= 1".to_string()
                )
            )
        }
    }

    ///
    /// Getter for min_addr_len
    /// 
    pub fn min_addr_len(&self) -> u16 {
        self.min_addr_len.as_u16()
    }

    ///
    /// Setter for min_addr_len
    /// 
    pub fn set_min_addr_len(&mut self, min_addr_len: u16) -> Result<()> {
        if (64..=1023).contains(&min_addr_len) {
            self.min_addr_len = Number12::new(min_addr_len as u32)?;
            Ok(())
        } else {
            fail!(
                BlockError::InvalidData(
                    "should: min_addr_len >= 64 && min_addr_len <= 1023".to_string()
                )
            )
        }
    }    

    ///
    /// Getter for min_addr_len
    /// 
    pub fn max_addr_len(&self) -> u16 {
        self.max_addr_len.as_u16()
    }

    ///
    /// Setter for max_addr_len
    /// 
    pub fn set_max_addr_len(&mut self, max_addr_len: u16) -> Result<()> {
        if (64..=1024).contains(&max_addr_len) && self.min_addr_len <= max_addr_len as u32 {
            self.max_addr_len = Number12::new(max_addr_len as u32)?;
            Ok(())
        } else {
            fail!(
                BlockError::InvalidData(
                    "should: max_addr_len >= 64 && max_addr_len <= 1024 \
                     && self.min_addr_len <= max_addr_len".to_string()
                )
            )
        }
    }        

    ///
    /// Getter for addr_len_step
    /// 
    pub fn addr_len_step(&self) -> u16 {
        self.addr_len_step.as_u16()
    }

    ///
    /// Setter for min_addr_len
    /// 
    pub fn set_addr_len_step(&mut self, addr_len_step: u16) -> Result<()> {
        if addr_len_step <= 1024 {
            self.addr_len_step = Number12::new(addr_len_step as u32)?;
            Ok(())
        } else {
            fail!(
                BlockError::InvalidData("should: addr_len_step <= 1024".to_string())
            )
        }
    }       

    ///
    /// Getter for workchain_type_id
    /// 
    pub fn workchain_type_id(&self) -> u32 {
        self.workchain_type_id.as_u32()
    }

    ///
    /// Setter for min_addr_len
    /// 
    pub fn set_workchain_type_id(&mut self, workchain_type_id: u32) -> Result<()> {
        if workchain_type_id >= 1 {
            self.workchain_type_id = Number32::new(workchain_type_id)?;
            Ok(())
        } else {
            fail!(
                BlockError::InvalidData("should: workchain_type_id >= 1".to_string())
            )
        }
    } 
}

impl Deserializable for WorkchainFormat0 {
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        let min_addr_len = Number12::construct_from(slice)?;
        let max_addr_len = Number12::construct_from(slice)?;
        let addr_len_step = Number12::construct_from(slice)?;
        let workchain_type_id = Number32::construct_from(slice)?;
        if min_addr_len >= 64 && min_addr_len <= max_addr_len &&
           max_addr_len <= 1023 && addr_len_step <= 1023 &&
           workchain_type_id >= 1 {
                Ok(Self {
                    min_addr_len,
                    max_addr_len,
                    addr_len_step,
                    workchain_type_id,
                })
        } else {
            fail!(
                BlockError::InvalidData(
                    "should: min_addr_len >= 64 && min_addr_len <= max_addr_len \
                     && max_addr_len <= 1023 && addr_len_step <= 1023".to_string()
                )
            )
        }
    }
}

impl Serializable for WorkchainFormat0 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        if self.min_addr_len >= 64 && self.min_addr_len <= self.max_addr_len &&
           self.max_addr_len <= 1023 && self.addr_len_step <= 1023 &&
           self.workchain_type_id >= 1 {
                self.min_addr_len.write_to(cell)?;
                self.max_addr_len.write_to(cell)?;
                self.addr_len_step.write_to(cell)?;
                self.workchain_type_id.write_to(cell)?;
                Ok(())
        } else {
            fail!(
                BlockError::InvalidData(
                    "should: min_addr_len >= 64 && min_addr_len <= max_addr_len \
                     && max_addr_len <= 1023 && addr_len_step <= 1023".to_string()
                )
            )
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
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorkchainDescr {
    pub enabled_since: u32,
    actual_min_split: u8,
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
    pub fn set_min_split(&mut self, min_split: u8) -> Result<()> {
        if min_split <= 60 {
            self.min_split = min_split;
            Ok(())
        } else {
            fail!(
                BlockError::InvalidData(
                    "should: min_split <= max_split && max_split <= 60".to_string()
                )
            )
        }
    } 

    ///
    /// Getter for actual_min_split
    /// 
    pub fn actual_min_split(&self) -> u8 {
        self.actual_min_split
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
    pub fn set_max_split(&mut self, max_split: u8) -> Result<()> {
        if self.min_split <= max_split && max_split <= 60 {
            self.max_split = max_split;
            Ok(())
        } else {
            fail!(
                BlockError::InvalidData(
                    "should: min_split <= max_split && max_split <= 60".to_string()
                )
            )
        }
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn basic(&self) -> bool {
        matches!(self.format, WorkchainFormat::Basic(_))
    }

}

const WORKCHAIN_DESCRIPTOR_TAG : u8 = 0xA6;

impl Deserializable for WorkchainDescr {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag = cell.get_next_byte()?;
        if tag != WORKCHAIN_DESCRIPTOR_TAG {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: std::any::type_name::<Self>().to_string()
                }
            )
        }
        self.enabled_since.read_from(cell)?;
        let mut min = Number8::default();
        min.read_from(cell)?;
        self.actual_min_split = min.as_u8();
        let mut min = Number8::default();
        min.read_from(cell)?;
        self.min_split = min.as_u8();
        let mut max = Number8::default();
        max.read_from(cell)?;
        self.max_split = max.as_u8();
        cell.get_next_bit()?; // basic
        self.active = cell.get_next_bit()?;
        self.accept_msgs = cell.get_next_bit()?;
        let mut flags = Number13::default();
        flags.read_from(cell)?;
        self.flags = flags.as_u16();
        self.zerostate_root_hash.read_from(cell)?;
        self.zerostate_file_hash.read_from(cell)?;
        self.version.read_from(cell)?;
        self.format.read_from(cell)?;

        Ok(())
    }
}

impl Serializable for WorkchainDescr {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        if self.min_split <= self.max_split && self.max_split <= crate::shard::MAX_SPLIT_DEPTH {

            cell.append_u8(WORKCHAIN_DESCRIPTOR_TAG)?;

            self.enabled_since.write_to(cell)?;

            let min = Number8::new(self.actual_min_split as u32)?;
            min.write_to(cell)?;

            let min = Number8::new(self.min_split as u32)?;
            min.write_to(cell)?;

            let max = Number8::new(self.max_split as u32)?;
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

            let flags = Number13::new(self.flags as u32)?;
            flags.write_to(cell)?;
            self.zerostate_root_hash.write_to(cell)?;
            self.zerostate_file_hash.write_to(cell)?;
            self.version.write_to(cell)?;
            self.format.write_to(cell)?;

            Ok(())
        } else {
            fail!(
                BlockError::InvalidData(
                    "should: min_split <= max_split && max_split <= 60".to_string()
                )
            )
        }
    }
}

/*
cfg_vote_cfg#36
    min_tot_rounds: uint8
    max_tot_rounds: uint8
    min_wins: uint8
    max_losses: uint8
    min_store_sec: uint32
    max_store_sec: uint32
    bit_price: uint32
    cell_price: uint32
= ConfigProposalSetup;

cfg_vote_setup#91
    normal_params: ^ConfigProposalSetup
    critical_params: ^ConfigProposalSetup
= ConfigVotingSetup;

_ ConfigVotingSetup = ConfigParam 11;
*/

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConfigProposalSetup {
    pub min_tot_rounds: u8,
    pub max_tot_rounds: u8,
    pub min_wins: u8,
    pub max_losses: u8,
    pub min_store_sec: u32,
    pub max_store_sec: u32,
    pub bit_price: u32,
    pub cell_price: u32,
}

const CONFIG_PROPOSAL_SETUP_TAG : u8 = 0x36;

impl Deserializable for ConfigProposalSetup {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        let tag = slice.get_next_byte()?;
        if tag != CONFIG_PROPOSAL_SETUP_TAG {
            fail!(BlockError::InvalidConstructorTag {
                t: tag as u32,
                s: std::any::type_name::<Self>().to_string()
            })
        }
        self.min_tot_rounds.read_from(slice)?;
        self.max_tot_rounds.read_from(slice)?;
        self.min_wins.read_from(slice)?;
        self.max_losses.read_from(slice)?;
        self.min_store_sec.read_from(slice)?;
        self.max_store_sec.read_from(slice)?;
        self.bit_price.read_from(slice)?;
        self.cell_price.read_from(slice)?;
        Ok(())
    }
}

impl Serializable for ConfigProposalSetup {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u8(CONFIG_PROPOSAL_SETUP_TAG)?;
        self.min_tot_rounds.write_to(cell)?;
        self.max_tot_rounds.write_to(cell)?;
        self.min_wins.write_to(cell)?;
        self.max_losses.write_to(cell)?;
        self.min_store_sec.write_to(cell)?;
        self.max_store_sec.write_to(cell)?;
        self.bit_price.write_to(cell)?;
        self.cell_price.write_to(cell)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConfigVotingSetup {
    normal_params: ChildCell<ConfigProposalSetup>,
    critical_params: ChildCell<ConfigProposalSetup>,
}

impl ConfigVotingSetup {

    pub fn new(normal_params: &ConfigProposalSetup, critical_params: &ConfigProposalSetup) -> Result<Self> {
        Ok(
            ConfigVotingSetup {
                normal_params: ChildCell::with_struct(normal_params)?,
                critical_params: ChildCell::with_struct(critical_params)?,
            }
        )
    }

    pub fn read_normal_params(&self) -> Result<ConfigProposalSetup> {
        self.normal_params.read_struct()
    }

    pub fn write_normal_params(&mut self, value: &ConfigProposalSetup) -> Result<()> {
        self.normal_params.write_struct(value)
    }

    pub fn normal_params_cell(&self)-> Cell {
        self.normal_params.cell()
    }

    pub fn read_critical_params(&self) -> Result<ConfigProposalSetup> {
        self.critical_params.read_struct()
    }

    pub fn write_critical_params(&mut self, value: &ConfigProposalSetup) -> Result<()> {
        self.critical_params.write_struct(value)
    }

    pub fn critical_params_cell(&self)-> Cell {
        self.critical_params.cell()
    }
}

const CONFIG_VOTING_SETUP_TAG : u8 = 0x91;

impl Deserializable for ConfigVotingSetup {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        let tag = slice.get_next_byte()?;
        if tag != CONFIG_VOTING_SETUP_TAG {
            fail!(BlockError::InvalidConstructorTag {
                t: tag as u32,
                s: std::any::type_name::<Self>().to_string()
            })
        }
        self.normal_params.read_from_reference(slice)?;
        self.critical_params.read_from_reference(slice)?;

        Ok(())
    }
}

impl Serializable for ConfigVotingSetup {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u8(CONFIG_VOTING_SETUP_TAG)?;
        cell.checked_append_reference(self.normal_params.cell())?;
        cell.checked_append_reference(self.critical_params.cell())?;
        Ok(())
    }
}

pub type ConfigParam11 = ConfigVotingSetup;

/*
_ workchains:(HashmapE 32 WorkchainDescr) = ConfigParam 12;
*/
define_HashmapE!{Workchains, 32, WorkchainDescr}

///
/// ConfigParam 12 struct
/// 
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConfigParam12 {
    pub workchains: Workchains,
}

impl ConfigParam12 {
    /// new instance of ConfigParam12
    pub fn new() -> Self {
        Self::default()
    }

    /// get length
    pub fn len(&self) -> Result<usize> {
        self.workchains.len()
    } 

    /// determine is empty
    pub fn is_empty(&self) -> bool {
        self.workchains.is_empty()
    } 

    /// get value by index
    pub fn get(&self, workchain_id: i32) -> Result<Option<WorkchainDescr>> {
        self.workchains.get(&workchain_id)
    }

    /// insert value
    pub fn insert(&mut self, workchain_id: i32, sp: &WorkchainDescr) -> Result<()> {
        self.workchains.set(&workchain_id, sp)
    }
}


impl Deserializable for ConfigParam12 {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        self.workchains.read_from(slice)?;
        Ok(())
    }
}

impl Serializable for ConfigParam12 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.workchains.write_to(cell)?;
        Ok(())
    }
}

///
/// ConfigParam 13 struct
/// 
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConfigParam13 {
    pub cell: Cell,
}

impl Deserializable for ConfigParam13 {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        self.cell = slice.clone().into_cell();
        Ok(())
    }
}

impl Serializable for ConfigParam13 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.checked_append_references_and_data(&SliceData::load_cell_ref(&self.cell)?)?;
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
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        let tag = slice.get_next_byte()?; // TODO what is tag length in bits???
        if tag != VALIDATOR_TEMP_KEY_TAG {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: std::any::type_name::<Self>().to_string()
                }
            )
        }
        self.adnl_addr.read_from(slice)?;
        self.temp_public_key.read_from(slice)?;
        self.seqno.read_from(slice)?;
        self.valid_until.read_from(slice)?;
        Ok(())
    }
}

impl Serializable for ValidatorTempKey {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
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
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        let tag = slice.get_next_byte()?; // TODO what is tag length in bits???
        if tag != VALIDATOR_SIGNED_TEMP_KEY_TAG {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: std::any::type_name::<Self>().to_string()
                }
            )
        }
        self.signature.read_from(slice)?;
        self.key.read_from_reference(slice)?;
        Ok(())
    }
}

impl Serializable for ValidatorSignedTempKey {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u8(VALIDATOR_SIGNED_TEMP_KEY_TAG)?; // TODO what is tag length in bits???
        self.signature.write_to(cell)?;
        cell.checked_append_reference(self.key.serialize()?)?;
        Ok(())
    }
}

///
/// ConfigParam 39 struct
/// 
// _ (HashmapE 256 ValidatorSignedTempKey) = ConfigParam 39;
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConfigParam39 {
    pub validator_keys: ValidatorKeys,
}

define_HashmapE!(ValidatorKeys, 256, ValidatorSignedTempKey);

impl ConfigParam39 {
    pub fn new() -> Self {
        Default::default()
    }
    /// get length
    pub fn len(&self) -> Result<usize> {
        self.validator_keys.len()
    } 

    /// determine is empty
    pub fn is_empty(&self) -> bool {
        self.validator_keys.is_empty()
    } 

    /// get value by key
    pub fn get(&self, key: &UInt256) -> Result<ValidatorSignedTempKey> {
        self
            .validator_keys
            .get(key)?
            .ok_or_else(|| error!(BlockError::InvalidArg(format!("{:x}", key))))
    }

    /// insert value
    pub fn insert(&mut self, key: &UInt256, validator_key: &ValidatorSignedTempKey) -> Result<()> {
        self.validator_keys.set(key, validator_key)
    }
}

impl Deserializable for ConfigParam39 {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        self.validator_keys.read_from(slice)?;
        Ok(())
    }
}

impl Serializable for ConfigParam39 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.validator_keys.write_to(cell)?;
        Ok(())
    }
}

///
/// ConfigParam 40 struct
///

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ConfigParam40 {
    pub slashing_config: SlashingConfig,
}

impl ConfigParam40 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deserializable for ConfigParam40 {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.slashing_config.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam40 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.slashing_config.write_to(cell)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlashingConfig {
    pub slashing_period_mc_blocks_count : u32, //number of MC blocks for one slashing iteration
    pub resend_mc_blocks_count : u32, //number of MC blocks to resend slashing messages in case they have not been delivered
    pub min_samples_count : u32, //minimal number of samples to compute statistics parameters
    pub collations_score_weight : u32, //weight for collations score in total score
    pub signing_score_weight : u32, //weight for signing score in total score
    pub min_slashing_protection_score : u32, //minimal score to protect from any slashing [0..100]
    pub z_param_numerator : u32, //numerator for Z param of confidence interval
    pub z_param_denominator : u32, //numerator for Z param of confidence interval
}

impl SlashingConfig {
    pub fn new() -> Self {
        Self {
            slashing_period_mc_blocks_count : 100,
            resend_mc_blocks_count : 4,
            min_samples_count : 30,
            collations_score_weight : 0,
            signing_score_weight : 1,
            min_slashing_protection_score : 70,
            z_param_numerator : 2326, //98% confidence
            z_param_denominator : 1000,
        }
    }
}

impl Default for SlashingConfig {
    fn default() -> SlashingConfig {
        Self::new()
    }
}

const SLASHING_VERSION1_TAG: u8 = 1;

impl Deserializable for SlashingConfig {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        match cell.get_next_byte()? {
            SLASHING_VERSION1_TAG => {
                self.slashing_period_mc_blocks_count.read_from(cell)?;
                self.resend_mc_blocks_count.read_from(cell)?;
                self.min_samples_count.read_from(cell)?;
                self.collations_score_weight.read_from(cell)?;
                self.signing_score_weight.read_from(cell)?;
                self.min_slashing_protection_score.read_from(cell)?;
                self.z_param_numerator.read_from(cell)?;
                self.z_param_denominator.read_from(cell)?;
            }
            tag => {
                fail!(
                    BlockError::InvalidConstructorTag {
                        t: tag as u32,
                        s: std::any::type_name::<Self>().to_string()
                    }
                )
            }
        }
        Ok(())
    }
}

impl Serializable for SlashingConfig {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u8(SLASHING_VERSION1_TAG)?;
        self.slashing_period_mc_blocks_count.write_to(cell)?;
        self.resend_mc_blocks_count.write_to(cell)?;
        self.min_samples_count.write_to(cell)?;
        self.collations_score_weight.write_to(cell)?;
        self.signing_score_weight.write_to(cell)?;
        self.min_slashing_protection_score.write_to(cell)?;
        self.z_param_numerator.write_to(cell)?;
        self.z_param_denominator.write_to(cell)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub enum ParamLimitIndex {
    Underload = 0,
    Normal,
    Soft,
    Medium,
    Hard
}

const LIMIT_COUNT: usize = 4;

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
    // [ unerload , soft, (soft+hard)/2, hard ]
    limits: [u32; LIMIT_COUNT],
}

impl ParamLimits {

    pub fn with_limits(underload: u32, soft: u32, hard: u32) -> Result<Self> {
        let mut limits = [0u32; LIMIT_COUNT];
        Self::set_limits(&mut limits, underload, soft, hard)?;
        Ok(Self{limits})
    }

    pub fn classify(&self, value: u32) -> ParamLimitIndex {
        if value >= self.medium() {
            if value >= self.hard_limit() {
                ParamLimitIndex::Hard
            } else {
                ParamLimitIndex::Medium
            }
        } else if value >= self.underload() {
            if value >= self.soft_limit() {
                ParamLimitIndex::Soft
            } else {
                ParamLimitIndex::Normal
            }
        } else {
            ParamLimitIndex::Underload
        }
    }

    pub fn fits(&self, level: ParamLimitIndex, value: u32) -> bool {
        // *level*         *checks*
        // Underload       value < unerload
        // Normal          value < soft
        // Soft            value < medium
        // Medium          value < hard
        // Hard            always true
        level == ParamLimitIndex::Hard || value < self.limits[level as usize]
    }

    pub fn fits_normal(&self, value: u32, percent: u32) -> bool {
        value * 100 < self.soft_limit() * percent
    }

    pub fn underload(&self) -> u32 {
        self.limits[ParamLimitIndex::Underload as usize]
    }

    pub fn soft_limit(&self) -> u32 {
        self.limits[ParamLimitIndex::Soft as usize - 1]
    }

    pub fn medium(&self) -> u32 {
        self.limits[ParamLimitIndex::Medium as usize - 1]
    }

    pub fn hard_limit(&self) -> u32 {
        self.limits[ParamLimitIndex::Hard as usize - 1]
    }

    fn compute_medium_limit(soft: u32, hard: u32) -> u32 {
        soft + ((hard - soft) >> 1)
    }

    fn set_limits(
        limits: &mut [u32; LIMIT_COUNT], 
        underload: u32, 
        soft: u32, 
        hard: u32
    ) -> Result<()> {
        if underload > soft {
            fail!(
                BlockError::InvalidArg(
                    "underload have to be less or equal to soft limit".to_string()
                )
            )
        }
        if soft > hard {
            fail!(
                BlockError::InvalidArg(
                   "soft limit have to be less or equal to hard one".to_string()
                )
            )
        }
        limits[ParamLimitIndex::Underload as usize] = underload;
        limits[ParamLimitIndex::Soft as usize - 1] = soft;
        limits[ParamLimitIndex::Medium as usize - 1] = Self::compute_medium_limit(soft, hard);
        limits[ParamLimitIndex::Hard as usize - 1] = hard;
        Ok(())
    }

}

impl Deserializable for ParamLimits {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        let tag = slice.get_next_byte()?;
        if tag != PARAM_LIMITS_TAG {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: std::any::type_name::<Self>().to_string()
                }
            )
        }
        let underload = u32::construct_from(slice)?;
        let soft = u32::construct_from(slice)?;
        let hard = u32::construct_from(slice)?;
        Self::set_limits(&mut self.limits, underload, soft, hard)
    }
}

impl Serializable for ParamLimits {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u8(PARAM_LIMITS_TAG)?;
        self.underload().write_to(cell)?;
        self.soft_limit().write_to(cell)?;
        self.hard_limit().write_to(cell)?;
        Ok(())
    }
}

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
    lt_delta: ParamLimits,
}

impl BlockLimits {

    pub fn with_limits(bytes: ParamLimits, gas: ParamLimits, lt_delta: ParamLimits) -> Self {
        Self { bytes, gas, lt_delta }
    }

    pub fn bytes(&self) -> &ParamLimits {
        &self.bytes
    }

    pub fn gas(&self) -> &ParamLimits {
        &self.gas
    }

    pub fn lt_delta(&self) -> &ParamLimits {
        &self.lt_delta
    }

    pub fn fits(&self, level: ParamLimitIndex, bytes: u32, gas: u32, lt_delta: u32) -> bool {
        self.gas.fits(level, gas) &&
        self.bytes.fits(level, bytes) &&
        self.lt_delta.fits(level, lt_delta)
    }

    pub fn fits_normal(&self, bytes: u32, gas: u32, lt_delta: u32, percent: u32) -> bool {
        self.gas.fits_normal(gas, percent) &&
        self.bytes.fits_normal(bytes, percent) &&
        self.lt_delta.fits_normal(lt_delta, percent)
    }
}

impl Deserializable for BlockLimits {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        let tag = slice.get_next_byte()?;
        if tag != BLOCK_LIMITS_TAG {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: std::any::type_name::<Self>().to_string()
                }
            )
        }
        self.bytes.read_from(slice)?;
        self.gas.read_from(slice)?;
        self.lt_delta.read_from(slice)?;
        Ok(())
    }
}

impl Serializable for BlockLimits {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u8(BLOCK_LIMITS_TAG)?;
        self.bytes.write_to(cell)?;
        self.gas.write_to(cell)?;
        self.lt_delta.write_to(cell)?;
        Ok(())
    }
}

type ConfigParam22 = BlockLimits;
type ConfigParam23 = BlockLimits;

const COPYLEFT_TAG: u8 = 0x9A;

///
/// ConfigParam 42 struct
///
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ConfigCopyleft {
    pub copyleft_reward_threshold: Grams,
    pub license_rates: LicenseRates,
}

define_HashmapE!(LicenseRates, 8, u8);

impl ConfigCopyleft {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deserializable for ConfigCopyleft {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag = cell.get_next_byte()?;
        if tag != COPYLEFT_TAG {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: std::any::type_name::<Self>().to_string()
                }
            )
        }
        self.copyleft_reward_threshold.read_from(cell)?;
        self.license_rates.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigCopyleft {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u8(COPYLEFT_TAG)?;
        self.copyleft_reward_threshold.write_to(cell)?;
        self.license_rates.write_to(cell)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct SuspendedAddressesKey {
    pub workchain_id: i32,
    pub address: UInt256,
}
impl SuspendedAddressesKey {
    pub fn new(workchain_id: i32, address: UInt256) -> Self {
        Self { workchain_id, address }
    }
}
impl Serializable for SuspendedAddressesKey {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_i32(self.workchain_id)?;
        cell.append_raw(self.address.as_slice(), 256)?;
        Ok(())
    }
}
impl Deserializable for SuspendedAddressesKey {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        self.workchain_id = slice.get_next_i32()?;
        self.address = UInt256::construct_from(slice)?;
        Ok(())
    }
}
define_HashmapE!{SuspendedAddresses, 288, ()}
impl SuspendedAddresses {
    pub fn is_suspended(&self, wc: i32, addr: UInt256) -> Result<bool> {
        let key = SuspendedAddressesKey::new(wc, addr);
        Ok(self.get(&key)?.is_some())
    }
    pub fn add_suspended_address(&mut self, wc: i32, addr: UInt256) -> Result<()> {
        let key = SuspendedAddressesKey::new(wc, addr);
        self.set(&key, &())
    }
}

#[cfg(test)]
pub(crate) fn dump_config(params: &HashmapE) {
    params.iterate_slices(|ref mut key, ref mut slice| -> Result<bool> {
        let key = key.get_next_u32()?;
        match ConfigParamEnum::construct_from_slice_and_number(&mut SliceData::load_cell(slice.reference(0)?)?, key)? {
            ConfigParamEnum::ConfigParam31(ref mut cfg) => {
                println!("\tConfigParam31.fundamental_smc_addr");
                cfg.fundamental_smc_addr.iterate_keys(|addr: UInt256| -> Result<bool> {
                    println!("\t\t{}", addr);
                    Ok(true)
                })?;
            }
            ConfigParamEnum::ConfigParam34(ref mut cfg) => {
                println!("\tConfigParam34.cur_validators");
                for validator in cfg.cur_validators.list() {
                    println!("\t\t{:?}", validator);
                };
            }
            x => println!("\t{:?}", x)
        }
        Ok(true)
    }).unwrap();
}

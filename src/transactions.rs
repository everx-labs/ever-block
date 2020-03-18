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

use super::*;
use super::types::{InRefValue};
use std::sync::Arc;
use {AccountId, UInt256};
use ton_types::{BuilderData, IBitstring, SliceData};
use ton_types::dictionary::{HashmapE, HashmapType};


/*
acst_unchanged$0 = AccStatusChange;  // x -> x
acst_frozen$10 = AccStatusChange;    // init -> frozen
acst_deleted$11 = AccStatusChange;   // frozen -> deleted
*/
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AccStatusChange {
    Unchanged,
    Frozen,
    Deleted,
}

impl Default for AccStatusChange {
    fn default() -> Self {
        AccStatusChange::Unchanged
    }
}

impl Serializable for AccStatusChange {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        let (tag, bits_count) = match self {
            AccStatusChange::Unchanged => (0b0, 1),
            AccStatusChange::Frozen => (0b10, 2),
            AccStatusChange::Deleted => (0b11, 2),
        };
        cell.append_bits(tag, bits_count)?;
        Ok(())
    }
}

impl Deserializable for AccStatusChange {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        *self = match cell.get_next_bit()? {
            false => AccStatusChange::Unchanged,
            true => match cell.get_next_bit()? {
                false => AccStatusChange::Frozen,
                true => AccStatusChange::Deleted,
            }
        };
        Ok(())
    }
}

/*
cskip_no_state$00 = ComputeSkipReason;
cskip_bad_state$01 = ComputeSkipReason;
cskip_no_gas$10 = ComputeSkipReason;
*/
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ComputeSkipReason {
    NoState,
    BadState,
    NoGas,
}

impl Default for ComputeSkipReason {
    fn default() -> Self {
        ComputeSkipReason::NoState
    }
}

impl Serializable for ComputeSkipReason {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        let tag = match self {
            ComputeSkipReason::NoState => 0b0,
            ComputeSkipReason::BadState => 0b01,
            ComputeSkipReason::NoGas => 0b10,
        };
        cell.append_bits(tag, 2)?;
        Ok(())
    }
}

impl Deserializable for ComputeSkipReason {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        *self = match cell.get_next_bits(2)?[0] {
            0b00000000 => ComputeSkipReason::NoState,
            0b01000000 => ComputeSkipReason::BadState,
            0b10000000 => ComputeSkipReason::NoGas,
            tag => failure::bail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: "ComputeSkipReason".to_string()
                }
            )
        };
        Ok(())
    }
}

/*
tr_phase_compute_skipped$0
    reason:ComputeSkipReason
= TrComputePhase;

tr_phase_compute_vm$1
    success:Bool
    msg_state_used:Bool
    account_activated:Bool
    gas_fees:Gram // is it spec's typo? I think should be "Grams"
    _:^[
        gas_used:(VarUInteger 7)
        gas_limit:(VarUInteger 7)
        gas_credit:(Maybe (VarUInteger 3))
        mode:int8
        exit_code:int32
        exit_arg:(Maybe int32)
        vm_steps:uint32
        vm_init_state_hash:uint256
        vm_final_state_hash:uint256
    ]
  = TrComputePhase;
*/
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TrComputePhaseSkipped {
    pub reason: ComputeSkipReason
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TrComputePhaseVm {
    pub success: bool,
    pub msg_state_used: bool,
    pub account_activated: bool,
    pub gas_fees: Grams,
    pub gas_used: VarUInteger7,
    pub gas_limit: VarUInteger7,
    pub gas_credit: Option<VarUInteger3>,
    pub mode: i8,
    pub exit_code: i32,
    pub exit_arg: Option<i32>,
    pub vm_steps: u32,
    pub vm_init_state_hash: UInt256,
    pub vm_final_state_hash: UInt256
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TrComputePhase {
    Skipped(TrComputePhaseSkipped),
    Vm(TrComputePhaseVm)
}

impl TrComputePhase {
    pub fn get_vmphase_mut(&mut self) -> Option<&mut TrComputePhaseVm> {
        match self {
            TrComputePhase::Vm(ref mut vm_ref) => return Some(vm_ref),
            _ => None,
        }
    }

    /// Set flag, that account is activated. Use 'msg_used' parameter
    /// to indicate that inbound message is used for this activation.
    pub fn activated(&mut self, msg_used: bool) {
        match self {
            TrComputePhase::Vm(ref mut phase_ref) => {
                phase_ref.account_activated = true;
                phase_ref.msg_state_used = msg_used;
            },
            _ => {
                let mut vm_phase = TrComputePhaseVm::default();
                vm_phase.account_activated = true;
                vm_phase.msg_state_used = msg_used;
                *self = TrComputePhase::Vm(vm_phase);
            },
        }
    }
}

impl Default for TrComputePhase {
    fn default() -> Self {
        TrComputePhase::Skipped(TrComputePhaseSkipped{ reason: ComputeSkipReason::NoState })
    }
}

impl Serializable for TrComputePhase {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        if let TrComputePhase::Skipped(s) = self {
            cell.append_bit_zero()?; // tr_phase_compute_skipped$0
            s.reason.write_to(cell)?; // reason:ComputeSkipReason

        } else if let TrComputePhase::Vm(v) = self {
            cell.append_bit_one()? // tr_phase_compute_vm$1
                .append_bit_bool(v.success)? // success:Bool
                .append_bit_bool(v.msg_state_used)? // msg_state_used:Bool
                .append_bit_bool(v.account_activated)?; // account_activated:Bool

            v.gas_fees.write_to(cell)?; // gas_fees:Gram

            // fields below are serialized into separate cell
            let mut sep_cell = BuilderData::new();
            v.gas_used.write_to(&mut sep_cell)?; // gas_used:(VarUInteger 7)
            v.gas_limit.write_to(&mut sep_cell)?;// gas_limit:(VarUInteger 7)
            v.gas_credit.write_maybe_to(&mut sep_cell)?; // gas_credit:(Maybe (VarUInteger 3))
            v.mode.write_to(&mut sep_cell)?; // mode:int8
            v.exit_code.write_to(&mut sep_cell)?; // exit_code:int32
            v.exit_arg.write_maybe_to(&mut sep_cell)?; // exit_arg:(Maybe int32)
            v.vm_steps.write_to(&mut sep_cell)?; // vm_steps:uint32
            v.vm_init_state_hash.write_to(&mut sep_cell)?; // vm_init_state_hash:uint256
            v.vm_final_state_hash.write_to(&mut sep_cell)?; // vm_final_state_hash:uint256

            cell.append_reference(sep_cell);
        }

        Ok(())
    }
}

impl Deserializable for TrComputePhase {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {

        if !cell.get_next_bit()? {
            // tr_phase_compute_skipped$0clear
            let mut s = TrComputePhaseSkipped::default();
            s.reason.read_from(cell)?;// reason:ComputeSkipReason
            *self = TrComputePhase::Skipped(s);
        } else {
            // tr_phase_compute_vm$1
            let mut v = TrComputePhaseVm::default();

            v.success = cell.get_next_bit()?; // success:Bool
            v.msg_state_used = cell.get_next_bit()?; // msg_state_used:Bool
            v.account_activated = cell.get_next_bit()?; // account_activated:Bool
            v.gas_fees.read_from(cell)?; // gas_fees:Gram

            // fields below are serialized into separate cell
            let mut sep_cell = cell.checked_drain_reference()?.into();

            v.gas_used.read_from(&mut sep_cell)?; // gas_used:(VarUInteger 7)
            v.gas_limit.read_from(&mut sep_cell)?; // gas_limit:(VarUInteger 7)
            v.gas_credit = VarUInteger3::read_maybe_from(&mut sep_cell)?; // gas_credit:(Maybe (VarUInteger 3))
            v.mode = sep_cell.get_next_byte()? as i8; // mode:int8
            v.exit_code = sep_cell.get_next_u32()? as i32; // exit_code:int32
            v.exit_arg = i32::read_maybe_from(&mut sep_cell)?; // exit_arg:(Maybe int32)
            v.vm_steps = sep_cell.get_next_u32()?; // vm_steps:uint32
            v.vm_init_state_hash.read_from(&mut sep_cell)?; // vm_init_state_hash:uint256
            v.vm_final_state_hash.read_from(&mut sep_cell)?; // vm_final_state_hash:uint256

            *self = TrComputePhase::Vm(v);
        }
        Ok(())
    }
}

/*
tr_phase_storage$_
  storage_fees_collected:Grams
  storage_fees_due:(Maybe Grams)
  status_change:AccStatusChange
= TrStoragePhase;
*/
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TrStoragePhase {
    pub storage_fees_collected: Grams,
    pub storage_fees_due: Option<Grams>,
    pub status_change: AccStatusChange
}

impl TrStoragePhase {
    pub fn with_params(collected: Grams, due: Option<Grams>, status: AccStatusChange) -> Self {
        TrStoragePhase {
            storage_fees_collected: collected,
            storage_fees_due: due,
            status_change: status
        }
    }
}

impl Serializable for TrStoragePhase {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.storage_fees_collected.write_to(cell)?;
        self.storage_fees_due.write_maybe_to(cell)?;
        self.status_change.write_to(cell)?;

        Ok(())
    }
}

impl Deserializable for TrStoragePhase {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.storage_fees_collected.read_from(cell)?;
        self.storage_fees_due = Grams::read_maybe_from(cell)?;
        self.status_change.read_from(cell)?;

        Ok(())
    }
}

/*
tr_phase_bounce_negfunds$00 = TrBouncePhase;

tr_phase_bounce_nofunds$01
  msg_size:StorageUsed
  req_fwd_fees:Grams
= TrBouncePhase;

tr_phase_bounce_ok$1
  msg_size:StorageUsed
  msg_fees:Grams
  fwd_fees:Grams
= TrBouncePhase;
*/
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TrBouncePhaseNofunds {
    pub msg_size: StorageUsedShort,
    pub req_fwd_fees: Grams,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TrBouncePhaseOk {
    pub msg_size: StorageUsedShort,
    pub msg_fees: Grams,
    pub fwd_fees: Grams,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TrBouncePhase {
    Negfunds,
    Nofunds(TrBouncePhaseNofunds),
    Ok(TrBouncePhaseOk),
}

impl Default for TrBouncePhase {
    fn default() -> Self {
        TrBouncePhase::Negfunds
    }
}

impl Serializable for TrBouncePhase {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        match self {
            TrBouncePhase::Negfunds => {
                //tr_phase_bounce_negfunds$00
                cell.append_bits(0b00, 2)?;
            },
            TrBouncePhase::Nofunds(bp) => {
                // tr_phase_bounce_nofunds$01
                cell.append_bits(0b01, 2)?;
                bp.msg_size.write_to(cell)?; // msg_size:StorageUsed
                bp.req_fwd_fees.write_to(cell)?; // req_fwd_fees:Grams
            },
            TrBouncePhase::Ok(bp) => {
                // tr_phase_bounce_ok$1
                cell.append_bit_one()?;
                bp.msg_size.write_to(cell)?; // msg_size:StorageUsed
                bp.msg_fees.write_to(cell)?; // msg_fees:Grams
                bp.fwd_fees.write_to(cell)?; // fwd_fees:Grams
            },
        };
        Ok(())
    }
}

impl Deserializable for TrBouncePhase {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        if cell.get_next_bit()? {
            // tr_phase_bounce_ok$1
            let mut bp = TrBouncePhaseOk::default();
            bp.msg_size.read_from(cell)?; // msg_size:StorageUsed
            bp.msg_fees.read_from(cell)?; // msg_fees:Grams
            bp.fwd_fees.read_from(cell)?; // fwd_fees:Grams
            *self = TrBouncePhase::Ok(bp);
        } else {
            if cell.get_next_bit()? {
                // tr_phase_bounce_nofunds$01
                let mut bp = TrBouncePhaseNofunds::default();
                bp.msg_size.read_from(cell)?; // msg_size:StorageUsed
                bp.req_fwd_fees.read_from(cell)?; // req_fwd_fees:Grams
                *self = TrBouncePhase::Nofunds(bp);
            } else {
                //tr_phase_bounce_negfunds$00
                *self = TrBouncePhase::Negfunds;
            }
        }
        Ok(())
    }
}

impl TrBouncePhaseOk {
    pub fn with_params(msg_size: StorageUsedShort, msg_fees: Grams, fwd_fees: Grams) -> Self {
        TrBouncePhaseOk {
            msg_size: msg_size,
            msg_fees: msg_fees,
            fwd_fees: fwd_fees,
        }
    }
}

impl TrBouncePhaseNofunds {
    pub fn with_params(msg_size: StorageUsedShort, req_fwd_fees: Grams) -> Self {
        TrBouncePhaseNofunds {
            msg_size: msg_size,
            req_fwd_fees: req_fwd_fees,
        }
    }
}

/*
tr_phase_credit$_
    due_fees_collected:(Maybe Grams)
    credit:CurrencyCollection
= TrCreditPhase;
*/
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrCreditPhase {
    pub due_fees_collected: Option<Grams>,
    pub credit: CurrencyCollection,
}

impl TrCreditPhase {
    pub fn with_params(collected: Option<Grams>, credit: CurrencyCollection) -> Self {
        TrCreditPhase {
            due_fees_collected: collected,
            credit: credit,
        }
    }
}

impl Default for TrCreditPhase {
    fn default() -> Self {
        TrCreditPhase {
            due_fees_collected: Option::None,
            credit: CurrencyCollection::default(),
        }
    }
}

impl Serializable for TrCreditPhase {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.due_fees_collected.write_maybe_to(cell)?;
        self.credit.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for TrCreditPhase {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.due_fees_collected = Grams::read_maybe_from(cell)?;
        self.credit.read_from(cell)?;
        Ok(())
    }
}

/*
tick$0 = TickTock;
tock$1 = TickTock;
There are two kinds of TickTock: in transaction and in messages.
*/
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum TransactionTickTock {
    Tick,
    Tock
}

impl Default for TransactionTickTock {
    fn default() -> Self {
        TransactionTickTock::Tick
    }
}

impl Serializable for TransactionTickTock {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        match self {
            TransactionTickTock::Tick => cell.append_bit_zero()?,
            TransactionTickTock::Tock => cell.append_bit_one()?,
        };
        Ok(())
    }
}

impl Deserializable for TransactionTickTock {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        if cell.get_next_bit()? {
            *self =  TransactionTickTock::Tock
        } else {
            *self =  TransactionTickTock::Tick
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TrActionPhase {
    pub success: bool,
    pub valid: bool,
    pub no_funds: bool,
    pub status_change: AccStatusChange,
    pub total_fwd_fees: Option<Grams>,
    pub total_action_fees: Option<Grams>,
    pub result_code: i32,
    pub result_arg: Option<i32>,
    pub tot_actions: i16,
    pub spec_actions: i16,
    pub skipped_actions: i16,
    pub msgs_created: i16,
    pub action_list_hash: UInt256,
    pub tot_msg_size: StorageUsedShort,
}

impl Serializable for TrActionPhase {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_bit_bool(self.success)? // success:Bool
            .append_bit_bool(self.valid)? // valid:Bool
            .append_bit_bool(self.no_funds)?; // no_funds:Bool
        self.status_change.write_to(cell)?; // status_change:AccStatusChange
        self.total_fwd_fees.write_maybe_to(cell)?; // total_fwd_fees:(Maybe Grams)
        self.total_action_fees.write_maybe_to(cell)?; // total_action_fees:(Maybe Grams)
        self.result_code.write_to(cell)?; // result_code:int32
        self.result_arg.write_maybe_to(cell)?; // result_arg:(Maybe int32)
        self.tot_actions.write_to(cell)?; // tot_actions:uint16
        self.spec_actions.write_to(cell)?; // spec_actions:uint16
        self.skipped_actions.write_to(cell)?; // skipped_actions: uint16
        self.msgs_created.write_to(cell)?; // msgs_created:uint16
        self.action_list_hash.write_to(cell)?; // action_list_hash:uint256
        self.tot_msg_size.write_to(cell)?; // tot_msg_size:StorageUsed
        Ok(())
    }
}

impl Deserializable for TrActionPhase {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.success = cell.get_next_bit()?; // success:Bool
        self.valid = cell.get_next_bit()?; // valid:Bool
        self.no_funds = cell.get_next_bit()?; // no_funds:Bool
        self.status_change.read_from(cell)?; // status_change:AccStatusChange
        self.total_fwd_fees = Grams::read_maybe_from(cell)?; // total_fwd_fees:(Maybe Grams)
        self.total_action_fees = Grams::read_maybe_from(cell)?; // total_action_fees:(Maybe Grams)
        self.result_code.read_from(cell)?; // result_code:int32
        self.result_arg = i32::read_maybe_from(cell)?; // result_arg:(Maybe int32)
        self.tot_actions.read_from(cell)?; // tot_actions:uint16
        self.spec_actions.read_from(cell)?; // spec_actions:uint16
        self.skipped_actions.read_from(cell)?; // skipped_actions: uint16
        self.msgs_created.read_from(cell)?; // msgs_created:uint16
        self.action_list_hash.read_from(cell)?; // action_list_hash:uint256
        self.tot_msg_size.read_from(cell)?; // tot_msg_size:StorageUsed
        Ok(())
    }
}

/*
split_merge_info$_
  cur_shard_pfx_len:(## 6)
  acc_split_depth:(##6)
  this_addr:uint256
  sibling_addr:uint256
= SplitMergeInfo;
*/

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SplitMergeInfo {
    pub cur_shard_pfx_len: u8,
    pub acc_split_depth:  u8,
    pub this_addr: UInt256,
    pub sibling_addr: UInt256,
}

impl Serializable for SplitMergeInfo {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        if 0 != self.cur_shard_pfx_len & 0b11000000 {
            failure::bail!(
                BlockError::InvalidData("self.cur_shard_pfx_len is too long".to_string())
            )
        } else {
            cell.append_bits(self.cur_shard_pfx_len as usize, 6)?;
        }
        if 0 != self.acc_split_depth & 0b11000000 {
            failure::bail!(
                BlockError::InvalidData("self.acc_split_depth is too long".to_string()) 
            )
        } else {
            cell.append_bits(self.acc_split_depth as usize, 6)?;
        }
        self.this_addr.write_to(cell)?;
        self.sibling_addr.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for SplitMergeInfo {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.cur_shard_pfx_len = cell.get_next_bits(6)?[0] >> 2;
        self.acc_split_depth = cell.get_next_bits(6)?[0] >> 2;
        self.this_addr.read_from(cell)?;
        self.sibling_addr.read_from(cell)?;
        Ok(())
    }
}


/*
trans_ord$0000
    storage_ph:(Maybe TrStoragePhase)
    credit_ph:(Maybe TrCreditPhase)
    compute_ph:TrComputePhase
    action:(Maybe ^TrActionPhase)
    aborted:Boolean
    bounce:(Maybe TrBouncePhase)
    destroyed:Boolean
*/
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TransactionDescrOrdinary {
    pub credit_first: bool,
    pub storage_ph: Option<TrStoragePhase>,
    pub credit_ph: Option<TrCreditPhase>,
    pub compute_ph: TrComputePhase,
    pub action: Option<TrActionPhase>,
    pub aborted: bool,
    pub bounce: Option<TrBouncePhase>,
    pub destroyed: bool
}

impl Serializable for TransactionDescrOrdinary {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        // constructor tag is written in TransactionDescr::write_to
        cell.append_bit_bool(self.credit_first)?;
        self.storage_ph.write_maybe_to(cell)?;
        self.credit_ph.write_maybe_to(cell)?;
        self.compute_ph.write_to(cell)?;
        cell.append_bit_bool(self.action.is_some())?;
        cell.append_bit_bool(self.aborted)?;
        self.bounce.write_maybe_to(cell)?;
        cell.append_bit_bool(self.destroyed)?;

        if let Some(a) = &self.action {
            cell.append_reference(a.write_to_new_cell()?);
        }

        Ok(())
    }
}

impl Deserializable for TransactionDescrOrdinary {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        // constructor tag is read in TransactionDescr::write_to
        self.credit_first = cell.get_next_bit()?;
        self.storage_ph = TrStoragePhase::read_maybe_from(cell)?;
        self.credit_ph = TrCreditPhase::read_maybe_from(cell)?;
        self.compute_ph.read_from(cell)?;
        self.action = if cell.get_next_bit()? {
            let mut ap = TrActionPhase::default();
            ap.read_from(&mut cell.checked_drain_reference()?.into())?;
            Option::Some(ap)
        } else {
            Option::None
        };
        self.aborted = cell.get_next_bit()?;
        self.bounce = TrBouncePhase::read_maybe_from(cell)?;
        self.destroyed = cell.get_next_bit()?;
        Ok(())
    }
}

/*
trans_storage$0001
    storage_ph:TrStoragePhase
constructor tag is written and read in TransactionDescr::write_to
*/
type TransactionDescrStorage = TrStoragePhase;

/*
trans_tick_tock$001
    tt:TickTock
    storage:TrStoragePhase
    compute_ph:TrComputePhase
    action:(Maybe ^TrActionPhase)
    aborted:Boolean
    destroyed:Boolean
*/
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TransactionDescrTickTock {
    pub tt: TransactionTickTock,
    pub storage: TrStoragePhase,
    pub compute_ph: TrComputePhase,
    pub action: Option<TrActionPhase>,
    pub aborted: bool,
    pub destroyed: bool,
}

impl TransactionDescrTickTock {
    pub fn tick() -> Self {
        let mut descr = Self::default();
        descr.tt = TransactionTickTock::Tick;
        descr
    }
    pub fn tock() -> Self {
        let mut descr = Self::default();
        descr.tt = TransactionTickTock::Tock;
        descr
    }
}

impl Serializable for TransactionDescrTickTock {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.tt.write_to(cell)?;
        self.storage.write_to(cell)?;
        self.compute_ph.write_to(cell)?;
        cell.append_bit_bool(self.action.is_some())?;
        cell.append_bit_bool(self.aborted)?;
        cell.append_bit_bool(self.destroyed)?;

        if let Some(a) = &self.action {
            cell.append_reference(a.write_to_new_cell()?);
        }

        Ok(())
    }
}

impl Deserializable for TransactionDescrTickTock {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        // self.tt.read_from(cell)?;
        self.storage.read_from(cell)?;
        self.compute_ph.read_from(cell)?;
        self.action = if cell.get_next_bit()? {
            let mut ap = TrActionPhase::default();
            ap.read_from(&mut cell.checked_drain_reference()?.into())?;
            Option::Some(ap)
        } else {
            Option::None
        };
        self.aborted = cell.get_next_bit()?;
        self.destroyed = cell.get_next_bit()?;
        Ok(())
    }
}

/*
trans_split_prepare$0100
    split_info:SplitMergeInfo
    compute_ph:TrComputePhase
    action:(Maybe ^TrActionPhase)
    aborted:Boolean
    destroyed:Boolean
*/
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TransactionDescrSplitPrepare {
    pub split_info: SplitMergeInfo,
    pub compute_ph: TrComputePhase,
    pub action: Option<TrActionPhase>,
    pub aborted: bool,
    pub destroyed: bool,
}

impl Serializable for TransactionDescrSplitPrepare {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.split_info.write_to(cell)?;
        self.compute_ph.write_to(cell)?;
        cell.append_bit_bool(self.action.is_some())?;
        cell.append_bit_bool(self.aborted)?;
        cell.append_bit_bool(self.destroyed)?;

        if let Some(a) = &self.action {
            cell.append_reference(a.write_to_new_cell()?);
        }

        Ok(())
    }
}

impl Deserializable for TransactionDescrSplitPrepare {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.split_info.read_from(cell)?;
        self.compute_ph.read_from(cell)?;
        self.action = if cell.get_next_bit()? {
            let mut ap = TrActionPhase::default();
            ap.read_from(&mut cell.checked_drain_reference()?.into())?;
            Option::Some(ap)
        } else {
            Option::None
        };
        self.aborted = cell.get_next_bit()?;
        self.destroyed = cell.get_next_bit()?;
        Ok(())
    }
}

/*
trans_split_install$0101
    split_info:SplitMergeInfo
    prepare_transaction:^Transaction
    installed:Boolean
*/
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TransactionDescrSplitInstall {
    pub split_info: SplitMergeInfo,
    pub prepare_transaction: Arc<Transaction>,
    pub installed: bool,
}

impl Serializable for TransactionDescrSplitInstall {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.split_info.write_to(cell)?;
        cell.append_bit_bool(self.installed)?;
        cell.append_reference(self.prepare_transaction.write_to_new_cell()?);
        Ok(())
    }
}

impl Deserializable for TransactionDescrSplitInstall {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.split_info.read_from(cell)?;
        self.installed = cell.get_next_bit()?;

        let tr = Transaction::construct_from(&mut cell.checked_drain_reference()?.into())?;
        self.prepare_transaction = Arc::new(tr);

        Ok(())
    }
}

/*
trans_merge_prepare$0110
    split_info:SplitMergeInfo
    storage_ph:TrStoragePhase
    aborted:Boolean
*/
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TransactionDescrMergePrepare {
    pub split_info: SplitMergeInfo,
    pub storage_ph: TrStoragePhase,
    pub aborted: bool,
}

impl Serializable for TransactionDescrMergePrepare {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.split_info.write_to(cell)?;
        self.storage_ph.write_to(cell)?;
        cell.append_bit_bool(self.aborted)?;
        Ok(())
    }
}

impl Deserializable for TransactionDescrMergePrepare {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.split_info.read_from(cell)?;
        self.storage_ph.read_from(cell)?;
        self.aborted = cell.get_next_bit()?;
        Ok(())
    }
}

/*
trans_merge_install$0111
    split_info:SplitMergeInfo
    prepare_transaction:^Transaction
    credit_ph:(Maybe TrCreditPhase)
    compute_ph:TrComputePhase
    action:(Maybe ^TrActionPhase)
    aborted:Boolean
    destroyed:Boolean
*/
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TransactionDescrMergeInstall {
    pub split_info: SplitMergeInfo,
    pub prepare_transaction: Arc<Transaction>,
    pub credit_ph: Option<TrCreditPhase>,
    pub compute_ph: TrComputePhase,
    pub action: Option<TrActionPhase>,
    pub aborted: bool,
    pub destroyed: bool,
}

impl Serializable for TransactionDescrMergeInstall {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.split_info.write_to(cell)?;
        cell.append_reference(self.prepare_transaction.write_to_new_cell()?);
        self.credit_ph.write_maybe_to(cell)?;
        self.compute_ph.write_to(cell)?;
        cell.append_bit_bool(self.action.is_some())?;
        cell.append_bit_bool(self.aborted)?;
        cell.append_bit_bool(self.destroyed)?;

        if let Some(a) = &self.action {
            cell.append_reference(a.write_to_new_cell()?);
        }

        Ok(())
    }
}

impl Deserializable for TransactionDescrMergeInstall {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.split_info.read_from(cell)?;

        let tr = Transaction::construct_from(&mut cell.checked_drain_reference()?.into())?;
        self.prepare_transaction = Arc::new(tr);

        self.credit_ph = TrCreditPhase::read_maybe_from(cell)?;
        self.compute_ph.read_from(cell)?;
        self.action = if cell.get_next_bit()? {
            let mut ap = TrActionPhase::default();
            ap.read_from(&mut cell.checked_drain_reference()?.into())?;
            Option::Some(ap)
        } else {
            Option::None
        };
        self.aborted = cell.get_next_bit()?;
        self.destroyed = cell.get_next_bit()?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransactionDescr {
    Ordinary(TransactionDescrOrdinary),
    Storage(TransactionDescrStorage),
    TickTock(TransactionDescrTickTock),
    SplitPrepare(TransactionDescrSplitPrepare),
    SplitInstall(TransactionDescrSplitInstall),
    MergePrepare(TransactionDescrMergePrepare),
    MergeInstall(TransactionDescrMergeInstall)
}

impl Default for TransactionDescr {
    fn default() -> Self {
        TransactionDescr::Ordinary(TransactionDescrOrdinary::default())
    }
}

impl TransactionDescr {
    pub fn is_aborted(&self) -> bool {
        match self {
            TransactionDescr::Ordinary(ref desc) => { desc.aborted },
            TransactionDescr::TickTock(ref desc) => { desc.aborted },
            TransactionDescr::SplitPrepare(ref desc) => { desc.aborted },
            TransactionDescr::MergePrepare(ref desc) => { desc.aborted },
            TransactionDescr::MergeInstall(ref desc) => { desc.aborted },
            _ => false,
        }
    }

    pub fn compute_phase_ref(&self) -> Option<&TrComputePhase> {
        match self {
            TransactionDescr::Ordinary(ref desc) => Some(&desc.compute_ph),
            TransactionDescr::TickTock(ref desc) => Some(&desc.compute_ph),
            TransactionDescr::SplitPrepare(ref desc) => Some(&desc.compute_ph),
            TransactionDescr::MergeInstall(ref desc) => Some(&desc.compute_ph),
            _ => None,
        }
    }

    pub fn is_credit_first(&self) -> Option<bool> {
        match self {
            TransactionDescr::Ordinary(ref tr) => Some(tr.credit_first),
            _ => None,
        }
    }

    fn append_to_storage_used(&mut self, cell: &Cell) {
        match self {
            TransactionDescr::Ordinary(ref mut desc) => {
                if let Some(ref mut bounce) = desc.bounce {
                    match bounce {
                        TrBouncePhase::Nofunds(ref mut no_funds) => { no_funds.msg_size.append(cell);},
                        TrBouncePhase::Ok(ref mut ok) => { ok.msg_size.append(cell);},
                        _ => (),
                    };
                }
                if let Some(ref mut action) = desc.action {
                    action.tot_msg_size.append(cell);
                }
            },
            TransactionDescr::TickTock(ref mut desc) => {
                if let Some(ref mut action) = desc.action {
                    action.tot_msg_size.append(cell);
                }
            },
            TransactionDescr::SplitPrepare(ref mut desc) => {
                if let Some(ref mut action) = desc.action {
                    action.tot_msg_size.append(cell);
                }
            },
            TransactionDescr::MergeInstall(ref mut desc) => {
                if let Some(ref mut action) = desc.action {
                    action.tot_msg_size.append(cell);
                }
            },
            _ => (),
        }
    }

    ///
    /// mark the transaction as aborted
    ///
    pub fn mark_as_aborted(&mut self) {
        match self {
            TransactionDescr::Ordinary(ref mut desc) => { desc.aborted = true; },
            TransactionDescr::TickTock(ref mut desc) => { desc.aborted = true; },
            TransactionDescr::SplitPrepare(ref mut desc) => { desc.aborted = true; },
            TransactionDescr::MergePrepare(ref mut desc) => { desc.aborted = true; },
            TransactionDescr::MergeInstall(ref mut desc) => { desc.aborted = true; },
            _ => (),
        };
    }
}

impl Serializable for TransactionDescr {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        match self {
            TransactionDescr::Ordinary(o) => {
                cell.append_bits(0b0000, 4)?;
                o.write_to(cell)?;
            },
            TransactionDescr::Storage(s) => {
                cell.append_bits(0b0001, 4)?;
                s.write_to(cell)?;
            },
            TransactionDescr::TickTock(tt) => {
                cell.append_bits(0b001, 3)?;
                tt.write_to(cell)?;
            },
            TransactionDescr::SplitPrepare(sp) => {
                cell.append_bits(0b0100, 4)?;
                sp.write_to(cell)?;
            },
            TransactionDescr::SplitInstall(si) => {
                cell.append_bits(0b0101, 4)?;
                si.write_to(cell)?;
            },
            TransactionDescr::MergePrepare(mp) => {
                cell.append_bits(0b0110, 4)?;
                mp.write_to(cell)?;
            },
            TransactionDescr::MergeInstall(mi) => {
                cell.append_bits(0b0111, 4)?;
                mi.write_to(cell)?;
            }
        }
        Ok(())
    }
}

impl Deserializable for TransactionDescr {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        match cell.get_next_bits(4)?[0] {
            0b0000_0000 => {
                let mut o = TransactionDescrOrdinary::default();
                o.read_from(cell)?;
                *self = TransactionDescr::Ordinary(o);
            }
            0b0001_0000 => {
                let mut s = TransactionDescrStorage::default();
                s.read_from(cell)?;
                *self = TransactionDescr::Storage(s);
            }
            0b0010_0000 => {
                let mut tt = TransactionDescrTickTock::tick();
                tt.read_from(cell)?;
                *self = TransactionDescr::TickTock(tt);
            }
            0b0011_0000 => {
                let mut tt = TransactionDescrTickTock::tock();
                tt.read_from(cell)?;
                *self = TransactionDescr::TickTock(tt);
            }
            0b0100_0000 => {
                let mut sp = TransactionDescrSplitPrepare::default();
                sp.read_from(cell)?;
                *self = TransactionDescr::SplitPrepare(sp);
            }
            0b0101_0000 => {
                let mut si = TransactionDescrSplitInstall::default();
                si.read_from(cell)?;
                *self = TransactionDescr::SplitInstall(si);
            }
            0b0110_0000 => {
                let mut mp = TransactionDescrMergePrepare::default();
                mp.read_from(cell)?;
                *self = TransactionDescr::MergePrepare(mp);
            }
            0b0111_0000 => {
                let mut mi = TransactionDescrMergeInstall::default();
                mi.read_from(cell)?;
                *self = TransactionDescr::MergeInstall(mi);
            }
            tag => failure::bail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: "TransactionDescr".to_string()
                }
            )
        }
        Ok(())
    }
}


/*
update_hashes#72 {X:Type} old_hash:bits256 new_hash:bits256
  = HASH_UPDATE X;
*/
const HASH_UPDATE_TAG: u8 = 0x72;
#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct HashUpdate {
    pub old_hash: UInt256,
    pub new_hash: UInt256,
}

impl HashUpdate {
    // Creates new instance of HashUpdate with given hashes
    pub fn with_hashes(old_hash: UInt256, new_hash: UInt256) -> Self {
        HashUpdate {old_hash, new_hash}
    }
}

impl Serializable for HashUpdate {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u8(HASH_UPDATE_TAG)?;
        self.old_hash.write_to(cell)?;
        self.new_hash.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for HashUpdate {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag = cell.get_next_byte()?;
        if tag != HASH_UPDATE_TAG {
            failure::bail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: "HashUpdate".to_string()
                }
            )
        }
        self.old_hash.read_from(cell)?;
        self.new_hash.read_from(cell)?;
        Ok(())
    }
}

struct U15(i16);

impl Serializable for U15 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_bits(self.0 as usize, 15)?;
        Ok(())
    }
}

impl Deserializable for U15 {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        self.0 = slice.get_next_int(15)? as i16;
        Ok(())
    }
}

define_HashmapE!{OutMessages, 15, InRefValue<Message>}

pub type TransactionId = UInt256;

/*
transaction$0111 
    account_addr:bits256 
    lt:uint64 
    prev_trans_hash:bits256
    prev_trans_lt:uint64
    now:uint32
    outmsg_cnt:uint15
    orig_status:AccountStatus
    end_status:AccountStatus
    ^[  in_msg:(Maybe ^(Message Any)) 
        out_msgs:(HashmapE 15 ^(Message Any)) ]
    total_fees:CurrencyCollection
    state_update:^(HASH_UPDATE Account)
    description:^TransactionDescr 
= Transaction;
*/
#[derive(Debug, Clone)]
pub struct Transaction {
    pub account_addr: AccountId,
    pub lt: u64,
    pub prev_trans_hash: UInt256,
	pub prev_trans_lt: u64,
	pub now: u32,
    pub outmsg_cnt: i16,
    pub orig_status: AccountStatus,
    pub end_status: AccountStatus,
    pub in_msg: Option<ChildCell<Message>>,
    pub out_msgs: OutMessages,
    pub total_fees: CurrencyCollection,
    pub state_update: ChildCell<HashUpdate>,
    pub description: ChildCell<TransactionDescr>,
}

impl Transaction {

    /// create new transaction
    pub fn with_address_and_status(address: AccountId, orig_status: AccountStatus) -> Self {
        Transaction {
            account_addr: address,
            lt: 0,
            prev_trans_hash: UInt256::from([0;32]),
            prev_trans_lt: 0,
            now: 0,
            outmsg_cnt: 0,
            orig_status: orig_status,
            end_status: AccountStatus::AccStateActive,
            in_msg: None,
            out_msgs: OutMessages::default(),
            total_fees: CurrencyCollection::default(),
            state_update: ChildCell::default(),
            description: ChildCell::default(),
        }
    }

    pub fn with_account_and_message(account: &Account, msg: &Message, lt: u64) -> Result<Self> {
        Ok(
            Transaction {
                account_addr: account.get_id().unwrap_or(msg.int_dst_account_id().unwrap()),
                lt: lt,
                prev_trans_hash: UInt256::from([0;32]),
                prev_trans_lt: 0,
                now: 0,
                outmsg_cnt: 0,
                orig_status: account.status(),
                end_status: account.status(),
                in_msg: Some(ChildCell::with_struct(msg)?),
                out_msgs: OutMessages::default(),
                total_fees: CurrencyCollection::default(),
                state_update: ChildCell::default(),
                description: ChildCell::default(),
            }
        )
    }

    /// Get account address of transaction
    pub fn account_id<'a>(&'a self) -> &'a AccountId {
        &self.account_addr
    }

    /// set transaction time
    pub fn set_logical_time(&mut self, lt: u64) {
        self.lt = lt;
    }

    /// get transaction logical time
    pub fn logical_time(&self) -> u64 {
        self.lt
    }

    /// get hash of previous transaction
    pub fn prev_trans_hash(&self) -> UInt256 {
        self.prev_trans_hash.clone()
    }

    /// get logical time of previous transaction
    pub fn prev_trans_lt(&self) -> u64 {
        self.prev_trans_lt
    }

    /// set end status accaunt
    pub fn set_end_status(&mut self, end_status: AccountStatus) {
        self.end_status = end_status;
    }

    /// set total fees
    pub fn set_total_fees(&mut self, fees: CurrencyCollection) {
        self.total_fees = fees;
    }

    /// get total fees
    pub fn total_fees(&self) -> &CurrencyCollection {
        &self.total_fees
    }

    /// get mutable total fees
    pub fn total_fees_mut(&mut self) -> &mut CurrencyCollection {
        &mut self.total_fees
    }

    ///
    /// Calculate total transaction fees
    /// transaction fees is the amount fee for all out-messages
    ///
    pub fn calc_total_fees(&mut self) -> &CurrencyCollection {
        self.total_fees = CurrencyCollection::default();
        // TODO uncomment after merge with feature-block-builder
        /*for msg in self.out_msgs.iter() {
            if let Some(fee) = msg.get_fee()
            {
                total += fee;
            }
        }*/
        &self.total_fees
    }

    pub fn read_in_msg(&self) -> Result<Option<Message>> {
        Ok(
            match self.in_msg {
                Some(ref in_msg) => Some(in_msg.read_struct()?),
                None => None
            }
        )
    }

    pub fn write_in_msg(&mut self, value: Option<&Message>) -> Result<()> {
        self.in_msg = value.map(|v| ChildCell::with_struct(v)).transpose()?;
        Ok(())
    }

    pub fn in_msg_cell(&self) -> Option<&Cell> {
        self.in_msg.as_ref().map(|c| c.cell())
    }

    /// get output message by index
    pub fn get_out_msg(&self, index: i16) -> Result<Option<Message>> {
        Ok(self.out_msgs.get(&U15(index))?.map(|msg| msg.0))
    }

    /// iterate output messages
    pub fn iterate_out_msgs<F>(&self, f: &mut F) -> Result<()>
    where F: FnMut(Message) -> Result<bool> {
        self.out_msgs.iterate(&mut |msg| f(msg.0)).map(|_|())
    }

    /// add output message to Hashmap
    pub fn add_out_message(&mut self, mgs: &Message) -> Result<()> {
        let msg_cell = mgs.write_to_new_cell()?.into();

        let mut descr = self.read_description()?;
        descr.append_to_storage_used(&msg_cell);
        self.write_description(&descr)?;

        self.out_msgs.setref(
            &U15(self.outmsg_cnt),
            &msg_cell
            )?;
        self.outmsg_cnt += 1;
        Ok(())
    }


    pub fn read_state_update(&self) -> Result<HashUpdate> {
        self.state_update.read_struct()
    }

    pub fn write_state_update(&mut self, value: &HashUpdate) -> Result<()> {
        self.state_update.write_struct(value)
    }

    pub fn state_update_cell(&self) -> &Cell {
        self.state_update.cell()
    }

    pub fn read_description(&self) -> Result<TransactionDescr> {
        self.description.read_struct()
    }

    pub fn write_description(&mut self, value: &TransactionDescr) -> Result<()> {
        self.description.write_struct(value)
    }

    pub fn description_cell(&self) -> &Cell {
        self.description.cell()
    }

    pub fn msg_count(&self) -> i16 {
        self.outmsg_cnt
    }

    /// return now time
    pub fn now(&self) -> u32 {
        self.now
    }

    /// set now time
    pub fn set_now(&mut self, now: u32) {
        self.now = now;
    }

    pub fn prepare_proof(&self, block_root: &Cell) -> Result<Cell> {
        // proof for transaction and block info in block

        let usage_tree = UsageTree::with_root(block_root.clone());
        let block: Block = Block::construct_from(&mut usage_tree.root_slice()).unwrap();

        block.read_info()?;

        block
            .read_extra()?
            .read_account_blocks()?
            .get(self.account_id())?
            .ok_or(
                BlockError::InvalidArg(
                    "Transaction doesn't belong to given block \
                     (can't find account block)".to_string() 
                )
            )?
            .transactions()
            .get(&self.logical_time())?
            .ok_or(
                BlockError::InvalidArg(
                    "Transaction doesn't belong to given block".to_string()
                )
            )?;

        MerkleProof::create_by_usage_tree(block_root, &usage_tree)
            .and_then(|proof| proof.write_to_new_cell())
            .map(|cell| cell.into())
    }
}

impl PartialEq for Transaction {
    fn eq(&self, other: &Transaction) -> bool {
        self.account_addr == other.account_addr &&
        self.lt == other.lt &&
        self.prev_trans_hash == other.prev_trans_hash &&
        self.prev_trans_lt == other.prev_trans_lt &&
        self.now == other.now &&
        self.outmsg_cnt == other.outmsg_cnt &&
        self.orig_status == other.orig_status &&
        self.end_status == other.end_status &&
        self.in_msg == other.in_msg &&
        self.out_msgs == other.out_msgs &&
        self.total_fees == other.total_fees &&
        self.state_update == other.state_update &&
        self.description == other.description
    }
}

impl Eq for Transaction {}

impl Default for Transaction {
    fn default() -> Self {
        Transaction {
            account_addr: AccountId::from_raw(vec![0;32], 256),
            lt: 0,
            prev_trans_hash: UInt256::from([0;32]),
            prev_trans_lt: 0,
            now: 0,            
            outmsg_cnt: 0,
            orig_status: AccountStatus::AccStateUninit,
            end_status: AccountStatus::AccStateUninit,
            in_msg: None,
            out_msgs: OutMessages::default(),
            total_fees: CurrencyCollection::default(),
            state_update: ChildCell::default(),
            description: ChildCell::default()
        }
    }
}
const TRANSACTION_TAG : usize = 0x7;

impl Serializable for Transaction {
    fn write_to(&self, builder: &mut BuilderData) -> Result<()> {

        builder.append_bits(TRANSACTION_TAG, 4)?;
        self.account_addr.write_to(builder)?; // account_addr: AccountId,
        builder.append_u64(self.lt)?; // lt: u64,
        self.prev_trans_hash.write_to(builder)?;
        self.prev_trans_lt.write_to(builder)?;
        self.now.write_to(builder)?;
        builder.append_bits(self.outmsg_cnt as usize, 15)?; // outmsg_cnt: u15
        self.orig_status.write_to(builder)?; // orig_status: AccountStatus,
        self.end_status.write_to(builder)?; // end_status: AccountStatus
        // self.in_msg.write_maybe_to(builder)?;
        let mut builder1 = BuilderData::new();
        match &self.in_msg {
            Some(in_msg) => {
                builder1.append_bit_one()?;
                builder1.append_reference(in_msg.write_to_new_cell()?);
            },
            None => {
                builder1.append_bit_zero()?;
            }
        };
        self.out_msgs.write_to(&mut builder1)?;
        builder.append_reference(builder1);
        self.total_fees.write_to(builder)?; // total_fees
        builder.append_reference(self.state_update.write_to_new_cell()?); // ^(HASH_UPDATE Account)
        builder.append_reference(self.description.write_to_new_cell()?); // ^TransactionDescr

        Ok(())
    }
}

impl Deserializable for Transaction {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag = cell.get_next_int(4)? as usize;
        if tag != TRANSACTION_TAG {
            failure::bail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: "Transaction".to_string()
                }
            )
        }
        self.account_addr.read_from(cell)?; // account_addr
        self.lt = cell.get_next_u64()?; // lt
        self.prev_trans_hash.read_from(cell)?;
        self.prev_trans_lt.read_from(cell)?;
        self.now.read_from(cell)?;
        self.outmsg_cnt = cell.get_next_int(15)? as i16; // outmsg_cnt
        self.orig_status.read_from(cell)?; // orig_status
        self.end_status.read_from(cell)?; // end_status
        let ref mut cell1 = SliceData::from(cell.checked_drain_reference()?);
        if cell1.get_next_bit()? {
            let mut msg = ChildCell::default();
            msg.read_from(&mut cell1.checked_drain_reference()?.into())?;
            self.in_msg = Some(msg);
        }
        self.out_msgs.read_from(cell1)?;
        self.total_fees.read_from(cell)?; // total_fees
        self.state_update.read_from(&mut cell.checked_drain_reference()?.into())?; // ^(HASH_UPDATE Account)
        self.description.read_from(&mut cell.checked_drain_reference()?.into())?; // ^TransactionDescr

        Ok(())
    }
}

define_HashmapAugE!(Transactions, 64, InRefValue<Transaction>, CurrencyCollection);

/// 4.2.15. Collection of all transactions of an account.
/// From Lite Client v11:
/// acc_trans#5 account_addr:bits256
///      transactions:(HashmapAug 64 ^Transaction CurrencyCollection)
///      state_update:^(HASH_UPDATE Account)
/// = AccountBlock;
#[derive(Clone, Debug, Default, Eq)]
pub struct AccountBlock {
    account_addr: AccountId,
    transactions: Transactions,      // HashmapAug 64 ^Transaction CurrencyCollection
    state_update: ChildCell<HashUpdate>,        // ^(HASH_UPDATE Account)
    tr_count: isize,                 // for HashMap key - here need Logical Time of transaction
}

impl PartialEq for AccountBlock {
    fn eq(&self, other: &AccountBlock) -> bool {
        self.account_addr.eq(&other.account_addr)
            && self.transactions.eq(&other.transactions)
            && self.state_update.eq(&other.state_update)
    }
}

impl AccountBlock {
    pub fn with_address(address: AccountId) -> AccountBlock {
        AccountBlock {
            account_addr: address,
            transactions: Transactions::default(),
            state_update: ChildCell::default(),
            tr_count: 0
        }
    }

    /// add transaction to block
    pub fn add_transaction(&mut self, transaction: &Transaction) -> Result<()> {
        self.add_serialized_transaction(transaction, &transaction.write_to_new_cell()?.into())
    }

    /// append serialized transaction to block (use to increase speed)
    pub fn add_serialized_transaction(&mut self, transaction: &Transaction, transaction_cell: &Cell) -> Result<()> {
        if self.tr_count < 0 {
            self.tr_count = self.transactions.len()? as isize;
        }
        self.transactions.setref(
            &transaction.logical_time(),
            transaction_cell,
            transaction.total_fees()
        )?;
        self.tr_count += 1;
        Ok(())
    }

    /// get hash update for Account
    pub fn read_state_update(&self) -> Result<HashUpdate> {
        self.state_update.read_struct()
    }

    /// set hash update for Account
    pub fn write_state_update(&mut self, state_update: &HashUpdate) -> Result<()> {
        self.state_update.write_struct(state_update)
    }

    // get Block AccountId
    pub fn account_id<'a>(&'a self) -> &'a AccountId {
        &self.account_addr
    }

    // get Block AccountId as SliceData
    pub fn account_addr(&self) -> SliceData {
        self.account_addr.write_to_new_cell().unwrap().into()
    }

    /// get sum of all acoount's transactions
    pub fn total_fee(&self) -> &CurrencyCollection {
        self.transactions.root_extra()
    }
    /// count of transactions
    pub fn transaction_count(&self) -> Result<usize> {
        if self.tr_count < 0 {
            failure::bail!(BlockError::InvalidData("self.tr_count is negative".to_string()))
        }
        self.transactions.len()
    }
    /// update
    pub fn calculate_and_write_state(&mut self, old_state: &ShardStateUnsplit, new_state: &ShardStateUnsplit) -> Result<()> {
        if self.transactions.is_empty() {
            failure::bail!(BlockError::InvalidData("No transactions in account block".to_string()))
        } else if let Some(transaction) = self.transactions.single()? {
            // if block has only one transaction for account just copy state update from transaction
            self.write_state_update(&transaction.0.read_state_update()?)?;
        } else {
            // otherwice it is need to calculate Hash update
            let old_hash = old_state.read_accounts()?
                .get_as_slice(&self.account_addr)?
                .unwrap_or_default()
                .into_cell()
                .repr_hash();
            let new_hash = new_state.read_accounts()?
                .get_as_slice(&self.account_addr)?
                .ok_or(BlockError::Other("Account should be in new shard state".to_string()))?
                .into_cell()
                .repr_hash();
            self.write_state_update(&HashUpdate::with_hashes(old_hash, new_hash))?;
        }
        Ok(())
    }

    pub fn transaction_iterate<F> (&self, p: &mut F) -> Result<bool>
    where F: FnMut(Transaction) -> Result<bool> {
        self.transactions.iterate(&mut |transaction| p(transaction.0))
    }

    pub fn transaction_iterate_full<F> (&self, p: &mut F) -> Result<bool>
    where F: FnMut(u64, Cell, CurrencyCollection) -> Result<bool> {
        self.transactions.iterate_slices_with_keys_and_aug(&mut |ref mut key, transaction, aug|
            p(key.get_next_u64()?, transaction.reference(0)?, aug))
    }

    pub fn transactions(&self) -> &Transactions {
        &self.transactions
    }
}

const ACCOUNT_BLOCK_TAG : usize = 0x5;

impl Serializable for AccountBlock {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_bits(ACCOUNT_BLOCK_TAG, 4)?;
        self.account_addr.write_to(cell)?;                                  // account_addr: AccountId,
        self.transactions.write_hashmap_root(cell)?;
        cell.append_reference(self.state_update.write_to_new_cell()?);      // ^(HASH_UPDATE Account)
        Ok(())
    }
}

impl Deserializable for AccountBlock {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        let tag = slice.get_next_int(4)? as usize;
        if tag != ACCOUNT_BLOCK_TAG {
            failure::bail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: "AccountBlock".to_string()
                }
            )
        }
        self.account_addr.read_from(slice)?;                                 // account_addr

        let mut trs = Transactions::default();
        trs.read_hashmap_root(slice)?;
        // TODO: is it realy need to have it now? init with negative and move to using
        self.tr_count = trs.len().unwrap() as isize;
        self.transactions = trs;

        self.state_update.read_from(&mut slice.checked_drain_reference()?.into())?;   // ^(HASH_UPDATE Account)
        Ok(())
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
// 4.2.17. Collection of all transactions in a block.
// _ (HashmapAugE 256 AccountBlock CurrencyCollection) = ShardAccountBlocks;
define_HashmapAugE!(ShardAccountBlocks, 256, AccountBlock, CurrencyCollection);

/// external interface for ShardAccountBlock
impl ShardAccountBlocks {

    /// insert new AccountBlock or replace existing
    // TODO: will be removed when acc_id as slice and set as type
    pub fn insert(&mut self, account_block: &AccountBlock) -> Result<()> {
        self.set(
            &account_block.account_addr,
            &account_block,
            &account_block.total_fee()
        ).map(|_|())
    }

    /// remove AccountBlock
    // pub fn remove(&mut self, account_id: &AccountId) -> Option<AccountBlock> {
    //     let key = self.account_id.write_to_new_cell().unwrap().into()
    //     self.remove(account_id key.into())
    // }

    /// adds transaction to account by id from transaction
    pub fn add_transaction(&mut self, transaction: &Transaction) -> Result<()> {
        self.add_serialized_transaction(transaction, &transaction.write_to_new_cell()?.into())
    }

    pub fn add_serialized_transaction(&mut self, transaction: &Transaction, transaction_cell: &Cell) -> Result<()> {
        let account_id = transaction.account_id();
        // get AccountBlock for accountId, if not exist, create it
        let mut account_block = self.get(account_id)?.unwrap_or(
            AccountBlock::with_address(account_id.clone())
        );
        // append transaction to AccountBlock
        account_block.add_serialized_transaction(transaction, transaction_cell)?;
        self.set(account_id, &account_block, &transaction.total_fees())
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum TransactionProcessingStatus {
    Unknown = 0,
    Preliminary,
    Proposed,
    Finalized,
    Refused,
}

impl Default for TransactionProcessingStatus {
    fn default() -> Self {
        TransactionProcessingStatus::Unknown
    }
}

#[allow(dead_code)]
pub fn generate_tranzaction(address : AccountId) -> Transaction {
    let s_in_msg = generate_big_msg();
    let s_out_msg1 = generate_big_msg();
    let s_out_msg2 = Message::default();
    let s_out_msg3 = Message::default();

    let s_status_update = HashUpdate::default();
    let s_tr_desc = TransactionDescr::default();

    let mut tr = Transaction::with_address_and_status(address, AccountStatus::AccStateActive);
    tr.set_logical_time(123423);
    tr.set_end_status(AccountStatus::AccStateFrozen);
    tr.set_total_fees(CurrencyCollection::with_grams(653));
    tr.write_in_msg(Some(&s_in_msg)).unwrap();
    tr.add_out_message(&s_out_msg1).unwrap();
    tr.add_out_message(&s_out_msg2).unwrap();
    tr.add_out_message(&s_out_msg3).unwrap();
    tr.write_state_update(&s_status_update).unwrap();
    tr.write_description(&s_tr_desc).unwrap();
    tr
}

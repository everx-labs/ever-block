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
    master::McStateExtra,
    outbound_messages::EnqueuedMsg,
    shard::{AccountIdPrefixFull, ShardIdent},
    Serializable, Deserializable,
};
use ton_types::{
    error, Result, BuilderData, Cell, SliceData, HashmapE, HashmapType, UInt256,
};



/*
// key is [ shard:uint64 mc_seqno:uint32 ]  
_ (HashmapE 96 ProcessedUpto) = ProcessedInfo;
*/
define_HashmapE!(ProcessedInfo, 96, ProcessedUpto);

impl ProcessedInfo {
    pub fn min_seqno(&self) -> Result<u32> {
        match self.0.get_min(false, &mut 0)? {
            (Some(key), _value) => ProcessedInfoKey::construct_from(&mut key.into()).map(|key| key.mc_seqno),
            _ => Ok(0)
        }
    }
}

/// Struct ProcessedInfoKey describe key for ProcessedInfo
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ProcessedInfoKey {
    shard: u64,
    mc_seqno: u32,
}

impl ProcessedInfoKey {
    pub fn from_rec(rec: &ProcessedUpto) -> Self {
        Self {
            shard: rec.shard,
            mc_seqno: rec.mc_seqno,
        }
    }

    // New instance ProcessedInfoKey structure
    pub fn with_params(shard: u64, mc_seqno: u32) -> Self {
        ProcessedInfoKey {
            shard,
            mc_seqno,
        }
    }
    pub fn seq_no(&self) -> u32 {
        self.mc_seqno
    }
}

impl Serializable for ProcessedInfoKey {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.shard.write_to(cell)?;
        self.mc_seqno.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for ProcessedInfoKey {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.shard.read_from(cell)?;
        self.mc_seqno.read_from(cell)?;
        Ok(())
    }
}


///
/// Struct ProcessedUpto
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ProcessedUpto {
    pub shard: u64,
    pub mc_seqno: u32,
    pub last_msg_lt: u64,
    pub last_msg_hash: UInt256,
    pub ref_extra: Option<McStateExtra>,
}

impl ProcessedUpto {

    // New instance ProcessedUpto structure
    pub fn with_params(last_msg_lt: u64, last_msg_hash: UInt256) -> Self {
        ProcessedUpto {
            shard: 0,
            mc_seqno: 0,
            last_msg_lt,
            last_msg_hash,
            ref_extra: None,
        }   
    }
    pub fn already_processed(&self, enq: &EnqueuedMsg) -> Result<bool> {
        if enq.enqueued_lt() > self.last_msg_lt {
            return Ok(false)
        }

        let env = enq.read_out_msg()?;
        if !ShardIdent::contains(self.shard, env.next_addr().prefix()?) {
            return Ok(false)
        }
        if enq.enqueued_lt == self.last_msg_lt && self.last_msg_hash < env.message_cell().repr_hash() {
            return Ok(false)
        }
        if env.cur_addr().workchain_id()? == env.next_addr().workchain_id()?
            && ShardIdent::contains(self.shard, env.cur_addr().prefix()?)
        {
            // this branch is needed only for messages generated in the same shard
            // (such messages could have been processed without a reference from the masterchain)
            // enable this branch only if an extra boolean parameter is set
            return Ok(true)
        }
        let mut acc = AccountIdPrefixFull::default();
        acc.prefix = env.cur_addr().prefix()?;
        let shard_end_lt = self.compute_shard_end_lt(&acc)?;

        Ok(enq.enqueued_lt() < shard_end_lt)
    }
    pub fn contains(&self, other: &Self) -> bool {
        ShardIdent::with_tagged_prefix(0, self.shard).unwrap().is_ancestor_for(&ShardIdent::with_tagged_prefix(0, other.shard).unwrap())
            && self.mc_seqno >= other.mc_seqno
            && (self.last_msg_lt > other.last_msg_lt
            || (self.last_msg_lt == other.last_msg_lt && self.last_msg_hash >= other.last_msg_hash)
        )
    }
    pub fn compute_shard_end_lt(&self, acc: &AccountIdPrefixFull) -> Result<u64> {
        match self.ref_extra {
            Some(ref mc) if acc.is_valid() => mc.hashes.get_shard(
                &ShardIdent::with_tagged_prefix(acc.workchain_id, acc.prefix)?
            )?.map(|shard| shard.descr().end_lt).ok_or_else(|| error!("Shard not found")),
            _ => Ok(0)
        }
    }
}

impl Serializable for ProcessedUpto {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.last_msg_lt.write_to(cell)?;
        self.last_msg_hash.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for ProcessedUpto {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.last_msg_lt.read_from(cell)?;
        self.last_msg_hash.read_from(cell)?;
        Ok(())
    }
}

// IhrPendingInfo structure
define_HashmapE!(IhrPendingInfo, 320, IhrPendingSince);

///
/// IhrPendingSince structure
/// 
/// ihr_pending$_
///     import_lt:uint64
/// = IhrPendingSince;
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct IhrPendingSince {
	import_lt: u64,
}

impl IhrPendingSince {
    /// New default instance IhrPendingSince structure
    pub fn new() -> Self {
        Self::default()
    }

    // New instance IhrPendingSince structure
    pub fn with_import_lt(import_lt: u64) -> Self {
        IhrPendingSince {
            import_lt,
        }   
    }

    pub fn import_lt(&self) -> u64 {
        self.import_lt
    }
}

impl Serializable for IhrPendingSince {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.import_lt.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for IhrPendingSince {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.import_lt.read_from(cell)?;
        Ok(())
    }
}
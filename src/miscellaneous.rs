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
    Serializable, Deserializable,
};
use ton_types::{
    Result,
    UInt256,
    BuilderData, Cell, SliceData, HashmapE, HashmapType,
};



/*
// key is [ shard:uint64 mc_seqno:uint32 ]  
_ (HashmapE 96 ProcessedUpto) = ProcessedInfo;
*/
define_HashmapE!(ProcessedInfo, 96, ProcessedUpto);

/// Struct ProcessedInfoKey describe key for ProcessedInfo
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ProcessedInfoKey {
    shard: u64,
    mc_seeqno: u32,
}


impl ProcessedInfoKey {

    // New instance ProcessedInfoKey structure
    pub fn with_params(shard: u64, mc_seeqno: u32) -> Self {
        ProcessedInfoKey {
            shard,
            mc_seeqno,
        }
    }
}

impl Serializable for ProcessedInfoKey {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.shard.write_to(cell)?;
        self.mc_seeqno.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for ProcessedInfoKey {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.shard.read_from(cell)?;
        self.mc_seeqno.read_from(cell)?;
        Ok(())
    }
}


///
/// Struct ProcessedUpto
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ProcessedUpto {
    last_msg_lt: u64,
	last_msg_hash: UInt256, 
}

impl ProcessedUpto {

    // New instance ProcessedUpto structure
    pub fn with_params(last_msg_lt: u64, last_msg_hash: UInt256) -> Self {
        ProcessedUpto {
            last_msg_lt,
            last_msg_hash,
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
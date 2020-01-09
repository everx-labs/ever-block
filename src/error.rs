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

use failure::{Context, Fail, Backtrace};
use std::fmt::{Formatter, Result, Display};

#[derive(Debug)]
pub struct BlockError {
    inner: Context<BlockErrorKind>,
}

pub type BlockResult<T> = std::result::Result<T, failure::Error>;

#[derive(Debug, Fail)]
pub enum BlockErrorKind {
    /// Item is not found.
    #[fail(display = "{} is not found", item_name)]
    NotFound {
        item_name: String,
    },

    /// Invalid TL-B constructor tag.
    #[fail(display = "Invalid TL-B constructor tag `#{:x}` while parsing `{}` struct", t, s)]
    InvalidConstructorTag {
        t: u32,
        s: String,
    },

    /// Invalid operation.
    #[fail(display = "Invalid operation: {}", msg)]
    InvalidOperation {
        msg: String,
    },

    /// Invalid data.
    #[fail(display = "Invalid data: {}", msg)]
    InvalidData {
        msg: String,
    },

    /// Invalid argument.
    #[fail(display = "Invalid argument: {}", msg)]
    InvalidArg {
        msg: String,
    },

    /// Fatal error.
    #[fail(display = "Fatal error: {}", msg)]
    FatalError {
        msg: String,
    },

    /// Wrong merkle update.
    #[fail(display = "Wrong merkle update: {}", msg)]
    WrongMerkleUpdate {
        msg: String,
    },

    /// Wrong merkle proof.
    #[fail(display = "Wrong merkle proof: {}", msg)]
    WrongMerkleProof {
        msg: String,

    },

    /// Attempting to read data from pruned branch cell.
    #[fail(display = "Attempting to read {} from pruned branch cell", data)]
    PrunedCellAccess {
        data: String,
    },

    /// Signature error.
    #[fail(display = "Signature error: {}", inner)]
    Signature {
        inner: ed25519_dalek::SignatureError,
    },

    /// Invalid index.
    #[fail(display = "Invalid index: {}", index)]
    InvalidIndex {
        index: usize,
    },

    /// Wrong hash.
    #[fail(display = "Wrong hash")]
    WrongHash,

    /// TVM Exception
    #[fail(display = "VM Exception, code: {}", code)]
    TvmExceptionCode {
        code: ton_types::types::ExceptionCode,
    },

    /// IO Exception
    #[fail(display = "IO Exception: {}", error)]
    Io {
        error: std::io::Error,
    },

    /// Other error.
    #[fail(display = "{}", msg)]
    Other {
        msg: String,
    },
}

impl BlockError {
    pub fn kind(&self) -> &BlockErrorKind {
        self.inner.get_context()
    }
}

impl Fail for BlockError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for BlockError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        Display::fmt(&self.inner, f)
    }
}

impl From<BlockErrorKind> for BlockError {
    fn from(kind: BlockErrorKind) -> BlockError {
        BlockError { inner: Context::new(kind) }
    }
}

impl From<Context<BlockErrorKind>> for BlockError {
    fn from(inner: Context<BlockErrorKind>) -> BlockError {
        BlockError { inner }
    }
}

impl From<ton_types::types::ExceptionCode> for BlockError {
    fn from(code: ton_types::types::ExceptionCode) -> BlockError {
        BlockError::from(BlockErrorKind::TvmExceptionCode { code })
    }
}

impl From<std::io::Error> for BlockError {
    fn from(error: std::io::Error) -> BlockError {
        BlockError::from(BlockErrorKind::Io { error })
    }
}

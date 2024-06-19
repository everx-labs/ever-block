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

#[derive(Debug, thiserror::Error)]
pub enum BlockError {
    /// Fatal error.
    #[error("Fatal error: {0}")]
    FatalError(String),
    /// Invalid argument.
    #[error("Invalid argument: {0}")]
    InvalidArg(String),
    /// Invalid TL-B constructor tag.
    #[error("Invalid TL-B constructor tag `#{:x}` while parsing `{}` struct", .t, .s)]
    InvalidConstructorTag {
        t: u32,
        s: String,
    },
    /// Invalid data.
    #[error("Invalid data: {0}")]
    InvalidData(String),
    /// Invalid index.
    #[error("Invalid index: {0}")]
    InvalidIndex(usize),
    /// Invalid operation.
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    /// Item is not found.
    #[error("{0} is not found")]
    NotFound(String),
    /// Other error.
    #[error("{0}")]
    Other(String),
    /// Attempting to read data from pruned branch cell.
    #[error("Attempting to read {0} from pruned branch cell")]
    PrunedCellAccess(String),
    /// Wrong hash.
    #[error("Wrong hash")]
    WrongHash,
    /// Wrong merkle proof.
    #[error("Wrong merkle proof: {0}")]
    WrongMerkleProof(String),
    /// Wrong merkle update.
    #[error("Wrong merkle update: {0}")]
    WrongMerkleUpdate(String),
    #[error("Bad signature")]
    BadSignature,
    #[error("Unexpected struct variant: exp={0} real={1}")]
    UnexpectedStructVariant(String, String),
    #[error("Unsupported serde opts: {0} {:x}", .1)]
    UnsupportedSerdeOptions(String, usize),
    #[error("Mismatched serde options: {0} exp={1} real={2}")]
    MismatchedSerdeOptions(String, usize, usize),
}

/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
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

use ton_types::types::ExceptionCode;

#[derive(Debug, failure::Fail)]
pub enum BlockError {
    /// Fatal error.
    #[fail(display = "Fatal error: {}", 0)]
    FatalError(String),
    /// Invalid argument.
    #[fail(display = "Invalid argument: {}", 0)]
    InvalidArg(String),
    /// Invalid TL-B constructor tag.
    #[fail(display = "Invalid TL-B constructor tag `#{:x}` while parsing `{}` struct", t, s)]
    InvalidConstructorTag {
        t: u32,
        s: String,
    },
    /// Invalid data.
    #[fail(display = "Invalid data: {}", 0)]
    InvalidData(String),
    /// Invalid index.
    #[fail(display = "Invalid index: {}", 0)]
    InvalidIndex(usize),
    /// Invalid operation.
    #[fail(display = "Invalid operation: {}", 0)]
    InvalidOperation(String),
    /// Item is not found.
    #[fail(display = "{} is not found", 0)]
    NotFound(String),
    /// Other error.
    #[fail(display = "{}", 0)]
    Other(String),
    /// Attempting to read data from pruned branch cell.
    #[fail(display = "Attempting to read {} from pruned branch cell", 0)]
    PrunedCellAccess(String),
    /// TVM Exception
    #[fail(display = "VM Exception, code: {}", 0)]
    TvmException(ExceptionCode),
    /// Wrong hash.
    #[fail(display = "Wrong hash")]
    WrongHash,
    /// Wrong merkle proof.
    #[fail(display = "Wrong merkle proof: {}", 0)]
    WrongMerkleProof(String),
    /// Wrong merkle update.
    #[fail(display = "Wrong merkle update: {}", 0)]
    WrongMerkleUpdate(String),
    #[fail(display = "Bad signature")]
    BadSignature,
}

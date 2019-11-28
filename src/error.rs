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

use ExceptionCode;

error_chain! {

    types {
        BlockError, BlockErrorKind, BlockResultExt, BlockResult;
    }

    foreign_links {
        Io(std::io::Error);
        TvmExceptionCode(ExceptionCode);
        // Signature(ed25519_dalek::SignatureError);
    }

    errors {
        NotFound(item_name: String) {
            description("Item is not found"),
            display("{} is not found", item_name)
        }
        InvalidConstructorTag(t: u32, s: String) {
            description("Invalid TL-B constructor tag"),
            display("Invalid TL-B constructor tag `#{:x}` while parsing `{}` struct", t, s)
        }
        InvalidOperation(msg: String) {
            description("Invalid operation"),
            display("Invalid operation: {}", msg)
        }
        InvalidData(msg: String) {
            description("Invalid data"),
            display("Invalid data: {}", msg)
        }
        InvalidArg(msg: String) {
            description("Invalid argument"),
            display("Invalid argument: {}", msg)
        }
        FatalError(msg: String) {
            description("Fatal error"),
            display("Fatal error: {}", msg)
        }
        WrongMerkleUpdate(msg: String) {
            description("Wrong merkle update")
            display("Wrong merkle update: {}", msg)
        }
        WrongMerkleProof(msg: String) {
            description("Wrong merkle proof")
            display("Wrong merkle proof: {}", msg)
        }
        PrunedCellAccess(data: String) {
            description("Attempting to read data from pruned branch cell")
            display("Attempting to read {} from pruned branch cell", data)
        }
        Signature(inner: ed25519_dalek::SignatureError) {
            description("Signature error"),
            display("Signature error: {}", inner)
        }
        InvalidIndex(index: usize) {
            description("Invalid index")
            display("Invalid index: {}", index)
        }
        WrongHash {
            description("Wrong hash")
        }
        Other(msg: String) {
            description("Other error"),
            display("{}", msg)
        }
    }

}

#[macro_export]
macro_rules! block_err {
    ($code:expr) => {
        Err(BlockError::from_kind($code))
    };
}

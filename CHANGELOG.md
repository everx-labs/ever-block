# Release Notes

All notable changes to this project will be documented in this file.

## Version: 1.9.12

### New

- Fixed appending references to cells

## Version: 1.9.7

### New

- Add common as submodule

## Version: 1.9.0

### New

- New version of `Block` struct with out messages queue updates for foreign workchains
- Added `GlobalCapabilities::CapWc2WcQueueUpdates`
- Added new functions related with out messages queue updates

## Version: 1.8.19

### Fix

- Merkle proof pruned cell access fix

## Version: 1.8.0

### New

- Deprecated functions removed
- Refactor in message types naming
- Switched to Rust 2021 edition

## Version: 1.7.52

### New

- Performance issue in ValidatorDescr - removed ed25519_dalek::PublicKey using for holding public_key data
- Removed unused SignedBlock structure
- Bumped crc version to 3.0

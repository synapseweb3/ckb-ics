# ckb-ics

`ckb-ics` is a validation utility set providing IBC-compatible [objects](https://github.com/synapseweb3/ckb-ics/blob/main/axon/src/object.rs) and [messages](https://github.com/synapseweb3/ckb-ics/blob/main/axon/src/message.rs) for [CKB](https://github.com/nervosnetwork/ckb). `objects` are payloads of cross-chain commands, and their hash digests are located in the cells’ `data` field and the raw bytes are set in the witnesses to optimize on-chain capacity requirement. `messages` can be recognized as cross-chain commands, instructing the relayer on how to process them to complete the relay operations, such as establishing connection and handling packet.

In the case of [ibc-ckb-contracts](https://github.com/synapseweb3/ibc-ckb-contracts), the responsibility of `ckb-ics` is to verify whether an object in `objects` is valid under a specified IBC message in `messages`, by calling the corresponding handler in [handlers](https://github.com/nervosnetwork/ckb). In the case of Forcerelay, `ckb-ics` just provides structures in objects to help complete the assembly of CKB transactions, which can modify the status of on-chain IBC cells.

In general, project `ckb-ics` is the core validation library for `ibc-ckb-contracts` and Forcerelay, which validates parameters associated with Cosmos-IBC protocol for CKB.

## Integration

To integrate `ckb-ics` into your project, add the following line to your `Cargo.toml` file:

```toml
ckb-ics-axon = { git = "https://github.com/synapseweb3/ckb-ics", branch = "main" }
```

## Object Encoding/Decoding

IBC commitments are encoded with protobuf etc., compatible with [ibc-solidity](https://github.com/synapseweb3/ibc-solidity-contract/) or ibc-go.

IBC objects (whose hash are stored in ibc cells) are encoded with RLP due to historical implementation choices.

## Commitment Verification

The primary responsibility of an IBC light client is to validate counterparty commitments. In the case of `Axon → CKB`, Axon’s light client on CKB network follows the following steps to complete a verification process:

1. Verify the validity of an Axon `block` by referencing the metadata cells directly maintained by Axon itself.
2. Verify the account MPT proof of the IBC handler to get its storage trie root.
3. Verify the storage MPT proof for the commitment path and value.

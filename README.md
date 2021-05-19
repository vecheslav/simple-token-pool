# simple-token-pool
Simple token pool for solana

## Program
### Build
```
cd program
cargo build && cargo build-bpf
```

### Deploy
```
solana program deploy target/deploy/simple_token_pool.so
```

## CLI
### Build
```
cd cli && cargo build
```

### Create pool
```
cargo run create-pool <BANK_MINT_PUBKEY>
```
### Swap
```
cargo run swap <SENDER_PUBKEY> <RECIPIENT_PUBKEY> <AMOUNT> <POOL_PUBKEY> --owner <SENDER_KEYPAIR_PATH>
```

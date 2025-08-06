# Valence co-processor app template

This is a template for a Valence app.

It is configured for an application that leverages ZK-proofs in order to post
Ethereum ERC20 contract balances to a CW20 contract deployed on Neutron.

## Structure

### `./circuits`

The Valence Zero-Knowledge circuits directory.

Inside it you will find `erc20_balance` circuit, controller, and core crates that
will perform erc20 storage proofs.

#### Circuit

It serves as a recipient for witness data (state proofs or data) from the associated controller. It carries out assertions based on business logic and outputs a `Vec<u8>`, which is subsequently forwarded to on-chain applications.

#### Controller

Compiled WASM binary that the coprocessor service runs in order to compute the circuit witnesses from given JSON arguments. It features an entrypoint that accommodates user requests; it also receives the result of a proof computation by the service.

#### Core

Core crate will contain any types, methods, or other helpers that may be relevant to both the circuit and controller.

### `./deploy`

Valence Program and Circuit deployment script.

### `./strategist`

Valence Coordinator that submits proof requests to the co-processor, and posts the proofs
to the Valence Authorizations contract on Neutron.

## Requirements

- [Docker](https://docs.docker.com/get-started/)
- [Rust](https://www.rust-lang.org/tools/install)
- (only for manual debugging): [Cargo Valence subcommand](https://github.com/timewave-computer/valence-coprocessor/tree/v0.3.12?tab=readme-ov-file#cli-helper)
- (Optional): [Valence co-processor instance](https://github.com/timewave-computer/valence-coprocessor/tree/v0.3.12?tab=readme-ov-file#local-execution)

## Instructions

There are two ways to interact with your co-processor application.

First is the manual approach where you can leverage the `cargo-valence` package
to deploy your circuit to the co-processor.

Alternatively, you can take the automated approach where `deploy` crate binary
will do the deployment for you. After that, running the `strategist` crate
binary will perform the proof requests.

### Manual instructions

This section contains the instructions for manual interaction and debugging of a
co-processor app.

#### Install Cargo Valence

A CLI helper is provided to facilitate the use of standard operations like deploying a circuit, proving statements, and retrieving state information.

To install:

```bash
cargo install \
  --git https://github.com/timewave-computer/valence-coprocessor.git \
  --tag v0.3.12 \
  --locked cargo-valence
```

`cargo-valence` supports local development workflows, as well as connecting to the public coprocessor service at http://prover.timewave.computer:37281/

We will be using the public co-processor service. If you prefer to operate your own instance, omit the `--socket` parameter.

#### Deploy

The circuit must be deployed with its controller. The controller is the responsible to compute the circuit witnesses, while the circuit is the responsible to assert the logical statements of the partial program.

```sh
cargo-valence --socket https://service.coprocessor.valence.zone \
  deploy circuit \
  --controller ./circuits/erc20_balance/controller \
  --circuit erc20-balance-circuit
```

This will output the application id associated with the controller. Let's bind this id to an environment variable, for convenience.

```sh
export CONTROLLER=$(cargo-valence --socket https://service.coprocessor.valence.zone \
  deploy circuit \
  --controller ./circuits/erc20_balance/controller \
  --circuit erc20-balance-circuit | jq -r '.controller')
```

#### Prove

This command will queue a proof request for this circuit into the co-processor, returning a promise of execution.

```sh
cargo-valence --socket https://service.coprocessor.valence.zone \
  prove -j '{"erc20":"0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48","eth_addr":"0x8d41bb082C6050893d1eC113A104cc4C087F2a2a","neutron_addr": "neutron1m6w8n0hluq7avn40hj0n6jnj8ejhykfrwfnnjh"}' \
  -p /var/share/proof.bin \
  $CONTROLLER
```

The argument `-j '{"value": 42}'` will be forwarded to `./crates/controller/src/lib.rs:get_witnesses`. The output of this function will be then forwarded to the circuit for proving.

The command sends a proof request to the coprocessor's worker nodes. Once the proof is ready, it will be delivered to the program's entrypoint. The default implementation will then write the proof to the specified path within the program's virtual filesystem. Note that the virtual filesystem follows a FAT-16 structure, with file extensions limited to 3 characters and case-insensitive paths.

#### Storage

Once the proof is computed by the backend, it will be delivered to the virtual filesystem. We can visualize it via the `storage` command.

```sh
cargo-valence --socket https://service.coprocessor.valence.zone \
  storage \
  -p /var/share/proof.bin \
  $CONTROLLER | jq -r '.data' | base64 -d | jq
```

The output should be similar to the following structure:

```json
{
  "args": {
    "value": 42
  },
  "log": [
    "received a proof request with arguments {\"value\":42}"
  ],
  "payload": {
    "cmd": "store",
    "path": "/var/share/proof.bin"
  },
  "proof": "2gFcRWJhZ25SQTZYbTBKRWpnSUxyYzl6bEVxT3l4dEJPdHgyU2R0Z3ZqS2pTd2QvQU5MREJYcElZUytLOUo2VXlwK25tMzNCTU8vQkQwOStDZkVZNUhYZytRNDJwRU9SRkdqeVZVUFBoaGU3bXBBY1JYM0lVcnJDRm45VG92MjFzSFg5dFdidmdpeXA4cE43QU9HeHQ2VWFaRHpXVTdCdDZsRzBwSGd6Tm9lR085WkRzU2NER3Z1cnJxWXpJeGVQNGtVRFBsMFZKaWNhTDlhQWRJbXlxb2d5VFFtNWx3Vm00L25qVHBoUDhFNEZMQ3pOWDlnQzduK0Z0SVRiaHFlVndVdU11R0dUQ0xBQjEwV3B6MTluRzZ6L2o4M0VHTnJuNTk2Qkh0RnNEbkFBNnVFZklYREQ4Z3lXTDFuN0RIRVVDek1JKzhCYjJTMS9rOWgzejBmOGxjWEFCTUUzS1E92ThBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBckFBQUFBQUFBQUE9PQ==",
  "success": true
}
```

#### Public inputs

We can also open the public inputs of the proof via the Valence helper:

```sh
cargo-valence --socket https://service.coprocessor.valence.zone \
  proof-inputs \
  -p /var/share/proof.bin \
  $CONTROLLER | jq -r '.inputs' | base64 -d | hexdump -C
```

Note: The first 32 bytes of the public inputs are reserved for the co-processor root.

### Automated Instructions

Outlined below are the automated deployment and runtime instructions that
will enable the e2e flow of erc20 -> cw20 ZK-based queries.

#### Mnemonic setup

Full flow will involve transaction execution on Neutron. To enable that,
a mnemonic with available ntrn token balances is needed.

To configure your mnemonic, run the following:

```bash
cp .example.env .env
```

Then open the created `.env` file and replace `todo` with your mnemonic seed phrase.

#### Run the deployment script

`deploy` crate `main.rs` contains an automated script which will perform the
following actions:

1. Fetch the mnemonic from `env`
2. Read the input parameters from `deploy/src/inputs/neutron_inputs.toml`
3. Instantiate the neutron program on-chain
4. Compile and deploy the co-processor application
5. Set up the on-chain authorizations
6. Produce the setup artifacts which will be used as runtime inputs

You can execute the sequence above by running:

```bash
RUST_LOG=info cargo run -p deploy
```

#### Execute the runtime script

After the deployment script produces valid output artifact in `artifacts/neutron_strategy_config.toml`,
you are ready to start the coordinator that will submit the proof requests and post them on-chain.

You can start the coordinator by running:

```bash
RUST_LOG=info cargo run -p strategist
```

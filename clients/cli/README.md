# Network CLI

The command line interface (CLI) lets you run a prover node and contribute proofs to the Nexus network.
It is the highest-performance option for proving.

## Prerequisites

If you don't have these dependencies already, install them first.

### Linux

```
sudo apt update
sudo apt upgrade
sudo apt install build-essential pkg-config libssl-dev git-all
```

### macOS

If you have [installed Homebrew](https://brew.sh/) to manage packages on OS X,
run this command to install Git.

```
brew install git
```

### Windows

[Install WSL](https://learn.microsoft.com/en-us/windows/wsl/install),
then see Linux instructions above.

## Quick start

```
curl https://cli.nexus.xyz/ | sh
```

If you do not already have Rust, you will be prompted to install it.

## Terms of Use

Use of the CLI is subject to the [Terms of Use](https://nexus.xyz/terms-of-use).
The first time you run it, it prompts you to accept the terms. To accept the terms
noninteractively (for example, in a continuous integration environment),
add `NONINTERACTIVE=1` before `sh`.

## Known issues

* Only the latest version of the CLI is currently supported.
* Prebuilt binaries are not yet available.
* Linking email to prover id is currently available on the web version only.
* Counting cycles proved is not yet available in the CLI.
* Only proving is supported. Submitting programs to the network is in private beta.
  To request an API key, contact us at growth@nexus.xyz.

## Modifying source

The curl command in the quick start section downloads this repo to $HOME/.nexus/nexus-cli
and automatically runs it. If you want to modify the CLI, it's better to clone the GitHub
repo somewhere else.

To run an optimized build using Nexus servers, run the following command in clients/cli:

### Running the CLI

```sh
# Run the CLI on the beta network, with optional argument for parallelism.
cargo run -r -- start --env beta --max-threads 4
```

```sh
# Run the CLI on the beta network, with optional argument for parallelism.
# Support automatic node switching by node id list,If all node IDs fail, wait for an hour and try again. 
cargo run -r -- start --env beta --max-threads 4 --node-ids 1,2,3,4
```

### Clear credentials

```sh
cargo run -r -- logout
```

## Troubleshooting

### Protocol Buffer Compiler (protoc) Installation

If you encounter an error about `protoc` not being installed, you can install it:

#### Docker

```bash
docker pull nexusxyz/nexus-cli:latest
docker run -it --init nexusxyz/nexus-cli:latest start --env beta
```

#### macOS

```bash
# Install using Homebrew
brew install protobuf

# Verify installation
protoc --version
```

#### Windows

```bash
# Install using Chocolatey
choco install protobuf
```

#### Linux

```bash
# Install using apt
sudo apt install protobuf-compiler
```

## Resources

* [Network FAQ](https://nexus.xyz/network#network-faqs)
* [Discord server](https://discord.gg/nexus-xyz)

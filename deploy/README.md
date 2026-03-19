# RLMX Edge Deployment

## Overview

RLMX runs on edge devices (Raspberry Pi 5, ARM64 Linux servers) using local GGUF model inference via the ruvllm CandleBackend. No GPU required — CPU inference with NEON/ARM64 SIMD.

## Prerequisites

- ARM64 Linux device (RPi5 8GB recommended)
- Rust cross-compilation toolchain on build machine
- GGUF model file + matching tokenizer.json

## Build

On your development machine (macOS/Linux x86_64):

```bash
# Install cross-compilation target
rustup target add aarch64-unknown-linux-gnu

# Install linker (macOS)
brew install aarch64-linux-gnu-gcc

# Build release binary
cargo build --release --target aarch64-unknown-linux-gnu -p rlmx-cli --features ruvllm
```

The binary will be at `target/aarch64-unknown-linux-gnu/release/rlmx`.

## Deploy to Device

```bash
# Create directories on device
ssh rlmx@device "sudo mkdir -p /opt/rlmx/models"

# Copy binary
scp target/aarch64-unknown-linux-gnu/release/rlmx rlmx@device:/opt/rlmx/

# Copy model + tokenizer
scp ~/.rlmx/models/tinyllama-1.1b-q4_k_m.gguf rlmx@device:/opt/rlmx/models/
scp ~/.rlmx/models/tinyllama-1.1b-q4_k_m-tokenizer.json rlmx@device:/opt/rlmx/models/

# Copy systemd service
scp deploy/rlmx-edge.service rlmx@device:/tmp/
ssh rlmx@device "sudo cp /tmp/rlmx-edge.service /etc/systemd/system/"
```

## Start Service

```bash
ssh rlmx@device
sudo systemctl daemon-reload
sudo systemctl enable --now rlmx-edge
sudo systemctl status rlmx-edge
journalctl -u rlmx-edge -f     # Follow logs
```

## Memory Budget (8GB RPi5)

| Component | RAM |
|-----------|-----|
| Raspberry Pi OS | ~1.0 GB |
| RLMX binary + tokio runtime | ~50 MB |
| TinyLlama 1.1B Q4_K_M | ~600 MB |
| KV cache (2048 ctx) | ~100 MB |
| **Total** | **~1.75 GB** |
| **Headroom** | **~6.25 GB** |

For tighter memory, use a 0.5B model (~350MB).

## Systemd Service Configuration

The service file (`rlmx-edge.service`) is configured with:

- `MemoryMax=2G` — hard memory limit
- `Restart=on-failure` — auto-restart on crash
- `RLMX_MODEL_DIR=/opt/rlmx/models` — model location
- `RLMX_EDGE_MODEL=qwen2.5-0.5b-q4` — model to auto-load (edit to match your model name)

Edit the model name in the service file to match your downloaded model:

```bash
sudo systemctl edit rlmx-edge
# Add under [Service]:
# Environment=RLMX_EDGE_MODEL=tinyllama-1.1b-q4_k_m
```

## Test Connectivity

```bash
# From another machine on the network
curl -X POST http://device:3000/mcp \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'

# Check edge status
curl -X POST http://device:3000/mcp \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"rlmx_edge_status","arguments":{}}}'
```

## Supported Models

The ruvllm CandleBackend uses candle's `quantized_llama` module, which supports **llama-architecture GGUF files**. This includes:

| Model | Size | GGUF Q4_K_M | Notes |
|-------|------|-------------|-------|
| TinyLlama 1.1B Chat | 1.1B | ~669 MB | Good balance of size/quality |
| SmolLM2 135M | 135M | ~100 MB | Ultra-small, limited quality |
| Llama 3.2 1B | 1B | ~700 MB | Best quality at this size |
| Phi-3.5 mini | 3.8B | ~2.2 GB | Needs 4GB+ RAM |

**Important**: Qwen2 models use different GGUF metadata keys (`qwen2.*` instead of `llama.*`) and are NOT currently supported by the CandleBackend's GGUF loader. Stick to llama-architecture models.

Each GGUF model requires a companion `<model-name>-tokenizer.json` file in the same directory. Download it from the model's Hugging Face repo.

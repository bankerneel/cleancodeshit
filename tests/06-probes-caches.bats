#!/usr/bin/env bats

load test_helper

@test "Puppeteer removes only known downloaded browsers" {
  export PUPPETEER_CACHE_DIR="$HOME/downloads"
  for sub in chrome chrome-headless-shell chromium firefox; do fixture "$PUPPETEER_CACHE_DIR/$sub/bin"; done
  fixture "$PUPPETEER_CACHE_DIR/notes.txt"
  run ccs --select puppeteer --yes
  [ "$status" -eq 0 ]
  has "Puppeteer browser downloads"
  for sub in chrome chrome-headless-shell chromium firefox; do [ ! -d "$PUPPETEER_CACHE_DIR/$sub" ]; done
  [ -f "$PUPPETEER_CACHE_DIR/notes.txt" ]
}

@test "HF_HUB_CACHE takes precedence over HF_HOME and keeps datasets and tokens" {
  export HF_HOME="$HOME/hf" HF_HUB_CACHE="$HOME/other-hub"
  fixture "$HF_HOME/token"
  fixture "$HF_HOME/hub/models--sample/snapshots/v1/bin"
  fixture "$HF_HUB_CACHE/models--author--sample/snapshots/v1/bin"
  fixture "$HF_HUB_CACHE/datasets--sample/snapshots/v1/dataset"
  run ccs --select hf_hub --yes
  [ "$status" -eq 0 ]
  has "models--author--sample"
  [ ! -d "$HF_HUB_CACHE/models--author--sample" ]
  [ -f "$HF_HOME/token" ] && [ -d "$HF_HOME/hub/models--sample" ]
  [ -d "$HF_HUB_CACHE/datasets--sample" ]
  unset HF_HUB_CACHE
  run ccs --select hf_hub --yes
  [ "$status" -eq 0 ]
  [ ! -d "$HF_HOME/hub/models--sample" ]
}

@test "Ollama lists per-model size and cleans only manifests and blobs" {
  export OLLAMA_MODELS="$HOME/other-models"
  fixture "$OLLAMA_MODELS/blobs/sha256-sample"
  fixture "$OLLAMA_MODELS/manifests/registry/example"
  fixture "$OLLAMA_MODELS/manifests-v2/example"
  fixture "$OLLAMA_MODELS/keep"
  run ccs --select ollama --yes
  [ "$status" -eq 0 ]
  has "demo:latest  1.2 GB"
  [ ! -d "$OLLAMA_MODELS/blobs" ] && [ ! -d "$OLLAMA_MODELS/manifests" ]
  [ ! -d "$OLLAMA_MODELS/manifests-v2" ] && [ -f "$OLLAMA_MODELS/keep" ]
}

@test "PyTorch and LM Studio report individual files and remove just their caches" {
  export TORCH_HOME="$HOME/torch"
  fixture "$TORCH_HOME/hub/checkpoints/net.pth"
  fixture "$HOME/.lmstudio/models/owner/example/weights.gguf"
  fixture "$HOME/.lmstudio/models/owner/example/config.json"
  fixture "$HOME/.lmstudio/conversations/example.json"
  run ccs --select 'torch_hub,lm_studio' --yes
  [ "$status" -eq 0 ]
  has "net.pth"
  has "weights.gguf"
  [ ! -f "$TORCH_HOME/hub/checkpoints/net.pth" ]
  [ ! -f "$HOME/.lmstudio/models/owner/example/weights.gguf" ]
  [ -f "$HOME/.lmstudio/models/owner/example/config.json" ]
  [ -f "$HOME/.lmstudio/conversations/example.json" ]
  unset TORCH_HOME
  fixture "$XDG_CACHE_HOME/torch/hub/checkpoints/net.pth"
  run ccs --select torch_hub --yes
  [ "$status" -eq 0 ]
  [ ! -f "$XDG_CACHE_HOME/torch/hub/checkpoints/net.pth" ]
}

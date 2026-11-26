#!/usr/bin/env bats

load test_helper

@test "Claude debug handles both roots via a narrow session log allowlist" {
  export CLAUDE_CONFIG_DIR="$HOME/other-claude"
  log=abc12345-1234-2345-3456-123456789abc.txt
  fixture "$CLAUDE_CONFIG_DIR/debug/$log"
  fixture "$CLAUDE_CONFIG_DIR/settings.json"
  run ccs --select claude_debug --yes
  [ "$status" -eq 0 ]
  [ ! -e "$CLAUDE_CONFIG_DIR/debug/$log" ]
  [ -f "$CLAUDE_CONFIG_DIR/settings.json" ]
  export CLAUDE_CODE_DEBUG_LOGS_DIR="$CLAUDE_CONFIG_DIR/$log"
  fixture "$CLAUDE_CONFIG_DIR/$log"
  fixture "$CLAUDE_CONFIG_DIR/notes.txt"
  run ccs --list
  [ "$status" -eq 0 ]
  has "Claude Code debug logs"
  run ccs --select claude_debug --yes
  [ "$status" -eq 0 ]
  [ ! -e "$CLAUDE_CONFIG_DIR/$log" ]
  [ -f "$CLAUDE_CONFIG_DIR/notes.txt" ]
}

@test "Codex, Copilot and OpenCode log probes and clean respect their roots" {
  export CODEX_HOME="$HOME/other-codex" COPILOT_HOME="$HOME/other-copilot"
  export XDG_DATA_HOME="$HOME/other-data"
  fixture "$CODEX_HOME/log/codex-tui.log"
  fixture "$CODEX_HOME/auth.json"
  fixture "$COPILOT_HOME/logs/process-sample.log"
  fixture "$COPILOT_HOME/config.json"
  fixture "$XDG_DATA_HOME/opencode/log/sample.log"
  fixture "$XDG_DATA_HOME/opencode/opencode.db"
  run ccs --select 'codex_logs copilot_logs opencode_logs' --yes
  [ "$status" -eq 0 ]
  has "Codex CLI logs"
  has "GitHub Copilot CLI logs"
  has "OpenCode logs"
  [ ! -d "$CODEX_HOME/log" ] && [ -f "$CODEX_HOME/auth.json" ]
  [ ! -d "$COPILOT_HOME/logs" ] && [ -f "$COPILOT_HOME/config.json" ]
  [ ! -d "$XDG_DATA_HOME/opencode/log" ] && [ -f "$XDG_DATA_HOME/opencode/opencode.db" ]
}

@test "agent transcript cleanup preserves memory, project metadata and settings" {
  export CLAUDE_CONFIG_DIR="$HOME/claude" CODEX_HOME="$HOME/codex" GEMINI_CLI_HOME="$HOME/gemini"
  fixture "$CLAUDE_CONFIG_DIR/projects/demo/conversation.jsonl"
  fixture "$CLAUDE_CONFIG_DIR/projects/demo/memory/MEMORY.md"
  fixture "$CODEX_HOME/sessions/2026/11/04/rollout-1.jsonl"
  fixture "$CODEX_HOME/archived_sessions/rollout-2.jsonl"
  fixture "$CODEX_HOME/history.jsonl"
  fixture "$CODEX_HOME/config.toml"
  fixture "$GEMINI_CLI_HOME/.gemini/tmp/demo/chats/session-example.json"
  fixture "$GEMINI_CLI_HOME/.gemini/tmp/demo/chats/session-example.jsonl"
  fixture "$GEMINI_CLI_HOME/.gemini/tmp/demo/project.txt"
  run ccs_confirm yes --select 'claude_history codex_history gemini_history'
  [ "$status" -eq 0 ]
  has "Claude Code session transcripts"
  has "Codex CLI session transcripts"
  has "Gemini CLI session transcripts"
  [ ! -f "$CLAUDE_CONFIG_DIR/projects/demo/conversation.jsonl" ]
  [ -f "$CLAUDE_CONFIG_DIR/projects/demo/memory/MEMORY.md" ]
  [ ! -f "$CODEX_HOME/sessions/2026/11/04/rollout-1.jsonl" ]
  [ ! -f "$CODEX_HOME/archived_sessions/rollout-2.jsonl" ]
  [ ! -f "$CODEX_HOME/history.jsonl" ]
  [ -f "$CODEX_HOME/config.toml" ]
  [ ! -f "$GEMINI_CLI_HOME/.gemini/tmp/demo/chats/session-example.json" ]
  [ ! -f "$GEMINI_CLI_HOME/.gemini/tmp/demo/chats/session-example.jsonl" ]
  [ -f "$GEMINI_CLI_HOME/.gemini/tmp/demo/project.txt" ]
}

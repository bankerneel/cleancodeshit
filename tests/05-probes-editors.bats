#!/usr/bin/env bats

load test_helper

@test "closed VS Code cleans the four Linux XDG targets and preserves User" {
  export XDG_CONFIG_HOME="$HOME/xdg-team"
  code="$XDG_CONFIG_HOME/Code"
  for sub in logs Cache CachedData CachedExtensionVSIXs; do fixture "$code/$sub/entry"; done
  fixture "$code/User/settings.json"
  run ccs --select vscode --yes
  [ "$status" -eq 0 ]
  has "VS Code logs and caches"
  for sub in logs Cache CachedData CachedExtensionVSIXs; do [ ! -d "$code/$sub" ]; done
  [ -f "$code/User/settings.json" ]
}

@test "running VS Code is clearly skipped" {
  fixture "$XDG_CONFIG_HOME/Code/CachedData/entry"
  printf '%s\n' '/opt/VS Code/code' > "$CCS_PS_OUTPUT_FILE"
  run ccs --list
  [ "$status" -eq 0 ]
  has "VS Code skipped: close the app"
  [ -f "$XDG_CONFIG_HOME/Code/CachedData/entry" ]
}

@test "macOS Cursor cleans verified targets, skips running editor and preserves User" {
  export CCS_FAKE_UNAME=Darwin
  cursor="$HOME/Library/Application Support/Cursor"
  for sub in logs Cache CachedData 'Code Cache' GPUCache CachedExtensionVSIXs; do fixture "$cursor/$sub/entry"; done
  fixture "$cursor/User/settings.json"
  printf '%s\n' '/Applications/Cursor.app/Contents/MacOS/Cursor' > "$CCS_PS_OUTPUT_FILE"
  run ccs --list
  [ "$status" -eq 0 ]
  has "Cursor skipped: close the app"
  : > "$CCS_PS_OUTPUT_FILE"
  run ccs --select cursor --yes
  [ "$status" -eq 0 ]
  has "Cursor logs and caches"
  for sub in logs Cache CachedData 'Code Cache' GPUCache CachedExtensionVSIXs; do [ ! -d "$cursor/$sub" ]; done
  [ -f "$cursor/User/settings.json" ]
}

@test "macOS VS Code honours portable user data" {
  export CCS_FAKE_UNAME=Darwin VSCODE_PORTABLE="$HOME/portable"
  fixture "$VSCODE_PORTABLE/user-data/logs/log.txt"
  fixture "$HOME/Library/Application Support/Code/logs/untouched.txt"
  run ccs --select vscode --yes
  [ "$status" -eq 0 ]
  [ ! -f "$VSCODE_PORTABLE/user-data/logs/log.txt" ]
  [ -f "$HOME/Library/Application Support/Code/logs/untouched.txt" ]
}

@test "Windsurf and Devin Desktop skip open editors and touch only logs and crash dumps" {
  fixture "$XDG_CONFIG_HOME/Windsurf/logs/main.log"
  fixture "$XDG_CONFIG_HOME/Devin/crashpad/dump"
  fixture "$XDG_CONFIG_HOME/Windsurf/User/History/keep"
  printf '%s\n' Devin > "$CCS_PS_OUTPUT_FILE"
  run ccs --list
  [ "$status" -eq 0 ]
  has "Windsurf and Devin Desktop skipped"
  : > "$CCS_PS_OUTPUT_FILE"
  run ccs --select windsurf --yes
  [ "$status" -eq 0 ]
  [ ! -d "$XDG_CONFIG_HOME/Windsurf/logs" ]
  [ ! -d "$XDG_CONFIG_HOME/Devin/crashpad" ]
  [ -f "$XDG_CONFIG_HOME/Windsurf/User/History/keep" ]
}

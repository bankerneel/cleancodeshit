# Tests

Use Bats 1.13.0. Add its bin directory to your current command environment and run
`bash tests/run-tests.sh`. The runner keeps all fixtures and scratch files within
`tests/.bats-tmp`. The helper refuses to start the cleaner unless the resolved
`HOME` is strictly inside the current Bats test directory.

The tool and process shims only return fixture output or record calls. Test inputs
can select either platform layout. CI sets `BASH_UNDER_TEST=/bin/bash` so on macOS
the cleaner runs with the operating system's Bash 3.2.

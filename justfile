
nixgl := env_var('NIXGL')

default:
  just --list

[group('build')]
build:
  cargo build && \
    notify -a rust -i text-rust "Build Complete" "Powermenu build complete" || \
    notify -a rust -i text-rust "Build Failed!" "Powermenu build failed" 

[group('build')]
run:
  if [ -n "{{nixgl}}" ]; then \
    {{nixgl}} cargo run; \
  else \
    cargo run; \
  fi

[group('build')]
launcher:
  cargo run -- -v launcher;

[group('build')]
osd:
  cargo run -- -v osd;

[group('build')]
watch:
  if [ -n "{{nixgl}}" ]; then \
    {{nixgl}} cargo watch -w src -w resources -x "run -- -v daemon --quit-keybindings"; \
  else \
    cargo watch -w src -w resources -x "run -- -v daemon --quit-keybindings"; \
  fi

[group('build')]
nix:
  nix build

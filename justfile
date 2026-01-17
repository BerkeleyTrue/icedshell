
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
watch:
  if [ -n "{{nixgl}}" ]; then \
    {{nixgl}} cargo watch -w src -w resources -x "run -- --no-focus --dryrun -v"; \
  else \
    cargo watch -w src -w resources -x "run -- --no-focus --dryrun -v"; \
  fi

[group('build')]
nix:
  nix build

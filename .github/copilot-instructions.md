# Copilot instructions

## Project overview

`prompt` is a Rust 2021 command-line binary that renders a colored shell prompt. The implementation is intentionally concentrated in `src/main.rs`; `README.md` documents installation and the shell integration examples.

The prompt is assembled from:

- The current working directory, replacing the home-directory prefix with `~`.
- CLI-provided content (`--message`) and the previous command's exit code (`--exit-code`).
- Git repository information: branch/tag/short commit, repository operation state, working-tree changes, upstream divergence, whitespace errors, and unmerged-file count.
- Optional Kubernetes context/namespace from `kubectl`.
- Optional AWS profile/region from environment variables.

`main` starts the independent external lookups asynchronously and combines their results before printing. Git lookups only affect the output when the current directory is inside a Git work tree. Command failures generally degrade to an absent prompt component rather than aborting the prompt, which is important because this binary runs on every shell prompt.

The three chevrons are part of the user-facing output contract: exit status, uncommitted/untracked changes, and upstream state are represented by their colors. `--explain` must continue to describe that mapping. `--iterm2` adds the iTerm2 SetMark escape sequence before normal output.

## Build and test

The GitHub Actions workflows use the release profile:

```sh
cargo build --profile release
cargo test --profile release
```

For a fast local build or test run, omit the profile:

```sh
cargo build
cargo test
```

There are currently no automated tests. When adding tests, run one by name with:

```sh
cargo test <test_name>
cargo test <test_name> -- --exact
```

The release workflow builds `target/release/prompt`, and the publish workflow uploads that binary for tagged releases. Keep `Cargo.lock` committed when dependency versions change.

## Implementation conventions

- Keep the executable behavior in the single binary target unless a change clearly warrants extracting modules.
- Use small helpers for each prompt data source and preserve the existing `Option`-based absence semantics for unavailable external tools or values.
- Use `async_process::Command` for subprocesses and avoid blocking the shell prompt on slow Git operations; the working-tree diff check currently has a 500 ms timeout.
- When adding independent lookups, follow the existing `futures::join!` pattern so they run concurrently.
- Preserve the exact output ordering and color semantics: directory, custom message, Git name/state, Git warnings/counts, Kubernetes context/namespace, AWS profile/region, then the three chevrons.
- Update `README.md` when changing CLI flags, shell setup, prompt symbols, or output behavior.
- Keep changes compatible with Bash and fish command substitution, and avoid writing diagnostics to stdout because stdout is consumed as `PS1`/`fish_prompt`.
- Preserve forced color output: prompt output is rendered through pipes in normal shell usage, so relying on terminal color auto-detection breaks the display.

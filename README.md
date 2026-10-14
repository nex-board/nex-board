# NexBoard

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/nex-board/nex-board)

## What is this?

This system is developed for Kosen Festival in Osaka Metropolitan University College of Technology, by Student Association.

## Supported System

- NixOS 26.04 or Higher (With Nix Flakes)
- Linux (With Nix Flakes)
- macOS 26 or Higher (With Nix Flakes)

## Requirements

- CPU: Multi-Core, 1.0GHz or Higher x86_64 Processor (Linux) or ARM64 Processor (macOS).
- RAM: 4GB or Higher
- Storage: 10GB or more free space

## Usage

1. Create `~/ebb` and place your config.toml.
2. Create `~/ebb/presets` and place your preset csv files.
3. `nix run github:nex-board/nex-board` (If you're using mac: `nix run github:nex-board/nex-board#nex-board-unwrapped`) or run the binary.

\* Sample config files are in `example/` directory.

## Links
[Deepwiki](https://deepwiki.com/nex-board/nex-board)  
[Osaka Metropolitan University College of Technology Students' Association](https://omuct-gakuyukai.com)

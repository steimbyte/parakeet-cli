# parakeet-cli

A local **NVIDIA Parakeet** speech-to-text CLI for Linux, forked from
[`frankdierolf/whis`](https://github.com/frankdierolf/whis).

The original whis-cli supported OpenAI / Groq / Mistral / Deepgram / ElevenLabs /
OpenAI-Realtime cloud providers, plus local Whisper and Parakeet. **This fork
removes everything except local Parakeet** — no cloud APIs, no LLM
post-processing, no Whisper / whisper.cpp build.

## Why?

The upstream `transcribe-rs` dependency pins `whisper-rs 0.13.2`, which is
incompatible with the current `whisper-rs-sys` vendored whisper.cpp (the
`whisper_full_params` struct got restructured upstream). Building the original
`whis-cli` on a fresh toolchain hits a rustc `E0609` cascade. This fork sidesteps
the problem by not building whisper-rs at all.

If you want cloud transcription or local Whisper, use the upstream repo.
If you only want local Parakeet — that's all this fork does.

## Install

```bash
sudo pacman -S --needed vulkan-headers vulkan-icd-loader   # Arch / CachyOS
# or: sudo apt install libvulkan-dev vulkan-tools            # Debian / Ubuntu

cargo install --git https://github.com/steimbyte/parakeet-cli.git --branch main
# binary is named `whis` and lives in ~/.cargo/bin/
export PATH="$HOME/.cargo/bin:$PATH"
```

## First-run

```bash
whis setup          # pick a Parakeet model, downloads ~478 MB on first run
whis                # record from mic, transcribe, copy to clipboard
```

## Commands

```
whis                # one-shot: record → parakeet → clipboard
whis start          # background service (hotkey-driven)
whis stop
whis restart
whis status
whis toggle         # for compositor keybindings (e.g. Hyprland bindsym)
whis setup          # interactive wizard
whis model list     # list Parakeet models + install status
whis config ...     # git-style: `whis config vad true`, etc.
whis -d 10s         # timed recording (10 seconds)
whis -f audio.wav   # transcribe a WAV file
whis --print        # print to stdout instead of clipboard
whis -o out.txt     # save to file
whis --format srt   # subtitle output
```

## Configuration

Stored at `~/.config/parakeet-cli/settings.json` (0600).

```
whis config parakeet-model-path ~/.local/share/whis/models/parakeet/parakeet-tdt-0.6b-v3-int8
whis config vad true
whis config vad-threshold 0.4
whis config chunk-size 30
whis config language en
whis config cli-mode system          # or "direct" (needs input group on Linux)
whis config --list
```

## What's gone vs upstream

- All cloud providers (OpenAI, Mistral, Groq, Deepgram, ElevenLabs, Realtime)
- Local Whisper + whisper-rs / whisper.cpp / Vulkan GGML build
- LLM post-processing (Ollama / OpenAI / Mistral cleanup)
- Output presets
- Desktop (Tauri) and mobile frontends
- The `realtime` and `pulse-metadata` features

## License

MIT — same as upstream. See `LICENSE`.

## Attribution

Forked from [`frankdierolf/whis`](https://github.com/frankdierolf/whis) v0.7.2.
Original copyright: Frank Dierolf <frank_dierolf@web.de>.

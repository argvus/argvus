# Sounds of ARGVUS Focus and ARGVUS Clock

The three sounds in `packaging/archlinux/src/usr/share/argvus/sounds/` are original
works made for ARGVUS by additive synthesis (no samples, no third-party recordings).
They are distributed under the license of this repository, GPL-3.0-only.

| File | Use | Length | Content |
| --- | --- | --- | --- |
| `argvus-chime.oga` | Alarm and timer (default), played in a loop | 3.2 s | Rising E5, G#5, B5 bell notes, 0.22 s apart, twice (at 0 s and 1.6 s) |
| `argvus-beacon.oga` | Alarm and timer (alternative), played in a loop | 2.0 s | Three 880 Hz beeps of 0.13 s, 0.22 s apart, then silence |
| `argvus-bell.oga` | End of an ARGVUS Focus phase, played once | 2.6 s | Two soft strikes, A4 then C#5 at 0.35 s |

Synthesis parameters, to regenerate them:

- 48 kHz, mono, 16-bit PCM, peak normalized to 0.70 (about -3 dBFS), then encoded with
  `oggenc -q 4` (Ogg Vorbis).
- Bell notes: partials at 1.0, 2.0, 2.76 and 4.07 times the fundamental with amplitudes
  1.0, 0.35, 0.18 and 0.08, each with an exponential decay (time constants of the note
  decay multiplied by 1.0, 0.6, 0.4 and 0.25) and a 4 ms linear attack. Note decay:
  0.45 s for the chime, 0.9 s and 0.8 s for the bell; chime notes lose 10 % gain each.
- Beeps: fundamental plus 0.25 of the second and 0.08 of the third harmonic, with 8 ms
  linear fades.

Each loop ends in silence, so repeating it does not click.

## Playback

`argvus-time-ui::sound::SoundPlayer` plays files with `GtkMediaFile`, the GStreamer media
backend that the Arch `gtk4` package links in (`libgstplay`). Ogg Vorbis decoding comes from
`gst-plugins-base`; output goes through `pulsesink`/`pipewiresink`/`alsasink`
(`gst-plugins-good`, or `gst-plugin-pipewire`). Both GStreamer plugin packages are runtime
dependencies of `argvus`.

Alternatives considered: spawning `canberra-gtk-play`, `paplay` or `pw-play` (optional
executables, not declared by the package, and no error reporting back to the UI); `rodio`/`cpal`
(a second Rust audio stack, through ALSA only, plus `alsa-lib` and more crates to build);
`gstreamer-rs` (works, but adds bindings for what `GtkMediaFile` already offers). The chosen
backend adds no Rust dependency and reports errors through the `error` property of the
stream; a stream that is not prepared after 3 seconds also counts as failed.

`ARGVUS_SOUNDS_DIR` overrides `/usr/share/argvus/sounds`; `.cargo/config.toml` points it at
the payload for `cargo run` and `cargo test`.

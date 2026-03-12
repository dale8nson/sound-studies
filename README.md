# sound-studies

Experimental multi-threaded algorithmic composition engine in Rust.

> **Status: Work in progress / research.** This is an active area of exploration — code compiles and produces audio, but the API and architecture are subject to change.

## Overview

`sound-studies` is an experiment in using Rust's concurrency primitives to drive real-time algorithmic music generation. Multiple composer threads run independently, each emitting `NoteOn` / `NoteOff` messages over MPSC channels to a central synthesis engine that renders audio via the system's audio output device.

The current composition plays voices across a pentatonic pitch set with mathematically-scaled durations, producing overlapping melodic and sustained bass lines.

## Architecture

```
Composer threads (t1, t2, t3)
        │
        │ NoteOn / NoteOff  (MPSC channel)
        ▼
   Synth engine  ──►  cpal audio output
```

- **Composers** are standard Rust threads that loop through pitch/duration patterns, sleeping between note events to control rhythm.
- **Synth** maintains a per-note phase accumulator (`BitSet`-indexed) and renders samples using phase-based synthesis.
- **Audio output** is handled by [`cpal`](https://github.com/RustAudio/cpal), targeting whatever default output device the OS provides.

## Workspace layout

```
sound-studies/
└── interpreter/        # Main binary — composers + synth engine
    ├── src/
    │   ├── main.rs     # Composition logic and thread setup
    │   └── synth.rs    # Synth engine, phase generation, sample rendering
    └── macros/         # Proc macro utilities (keys! macro)
```

## Running

```bash
cd interpreter
cargo run
```

Audio will begin playing immediately on your default output device. Ctrl+C to stop.

## Built with

- [`cpal`](https://github.com/RustAudio/cpal) — cross-platform audio I/O
- [`pest`](https://github.com/pest-parser/pest) — PEG parser (for notation parsing, in progress)
- [`ndarray`](https://github.com/rust-ndarray/ndarray) — numerical arrays
- [`bit-set`](https://github.com/contain-rs/bit-set) — efficient note-mask representation
- [`ringbuf`](https://github.com/agerasev/ringbuf) — lock-free ring buffer for sample passing

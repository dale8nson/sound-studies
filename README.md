# sound-studies

Experimental algorithmic composition engine and music DSL compiler in Rust.

> **Status: Work in progress / research.** Active exploration — architecture and API subject to change.

## Overview

`sound-studies` is a workspace for experimenting with algorithmic music in Rust. The current focus is `compiler`: a pipeline that parses a custom music composition DSL (`.rsk` files) and compiles it to MIDI.

## Workspace layout

```
sound-studies/
├── compiler/           # .rsk DSL compiler — parser, AST, MIDI codegen
│   ├── src/
│   │   ├── main.rs             # Entry point: read .rsk file, parse, print AST
│   │   ├── pest_parser.rs      # Pest-based PEG parser → AST
│   │   └── compiler/
│   │       ├── ast.rs          # AST type definitions
│   │       ├── composer.rs     # Tree-walking compiler → MIDI events
│   │       └── mod.rs
│   ├── grammar-v2.pest         # Active PEG grammar
│   └── prototype.rsk           # Example composition
├── music-box/          # Experimental music utilities
├── macros/             # Proc-macro utilities (keys! macro)
└── src/main.rs         # Original multi-threaded synth engine
```

## The `.rsk` DSL

`.rsk` is an expression-oriented language for algorithmic music composition drawing on two programming paradigms:

- **Concatenative** — at the surface level, meaning arises from juxtaposition. Placing expressions next to each other implicitly threads a musical context from left to right, with no explicit binding operator. `d4 (120 144 60 120) bpm` is three tokens in sequence: a duration that sets context, a list that supplies values, and a suffix keyword that consumes them. This is the same model used by languages like Forth and Joy.
- **Functional** — in the compiler implementation, each node in the AST compiles to a closure (`F<Exp, Exp>`), and the compiler pipeline is built by composing those closures. Concatenative languages often have this property: Joy's formal semantics are defined entirely in terms of function composition.

Programs are nested expressions that specify duration, tempo, pitch, register, and rhythm.

**Grouping semantics:**

| Syntax | Meaning |
|--------|---------|
| `(...)` | Sequence — expressions play in order |
| `{...}` | Stack — expressions play simultaneously |
| `[...]` | Set — unordered collection |
| `a:b:c` | Ratio — proportional time subdivision |

**Primitives (prefix keywords):**

| Token | Meaning |
|-------|---------|
| `d<n>` | Fractional duration (e.g. `d4` = quarter note, `d8` = eighth note) |
| `5'` / `2"` | Fixed duration in minutes / seconds |
| `pc` | Pitch class |
| `reg` | Register (octave) |
| `r` | Rest |

**Primitives (suffix keywords):**

| Token | Meaning |
|-------|---------|
| `bpm` | Tempo in beats per minute |
| `A` | Amplitude (velocity) |
| `~` | Frequency (Hz) |

**Operators:**

| Token | Meaning |
|-------|---------|
| `><` | Intercalate — interleave two sequences |

### Example

```
5' (
  d4 (120 144 60 120) bpm
  2:5:7:3 (
    3:7:5:2 (
      5:3:2:7 (
        7:2:3:5 (
          {
            pc (
              (5 3 2 7)
              (7 2 3 5)
              (2 5 7 3)
              (3 7 5 2)
            )
            d (7.75 4.5 8 8.5)
            >< r (4.25 4 4.5)
            reg (4 5)
          }
        )
      )
    )
  )
)
```

This specifies a 5-minute composition, subdivided by nested ratios, with quarter-note durations, cycling BPM values, pitch-class sets interleaved with rests across two registers.

## Compiler pipeline

```
.rsk source
    │
    │  Pest PEG parser (grammar-v2.pest)
    ▼
  Program AST
    │
    │  Composer — tree of scoped contexts
    │  (duration · pitch class · tempo · register · velocity · instrument)
    ▼
  MIDI (midly)                   ← in progress
```

The parser is complete. The `Composer` walks the AST building a tree of `Ctx` nodes, each tracking its musical context inherited from its parent. Fixed durations (`5'`) create child contexts with an absolute length in microseconds; fractional durations (`d4`) are resolved relative to the current tempo. Scope type (Sequence / Stack / Set) determines how child events are serialised into MIDI tracks. Full MIDI event generation is in progress.

## Running

```bash
# Parse a .rsk file and print the AST
cd compiler
cargo run
```

## Built with

- [`pest`](https://github.com/pest-parser/pest) — PEG parser generator
- [`midly`](https://github.com/nickel-lang/midly) — MIDI file I/O
- [`cpal`](https://github.com/RustAudio/cpal) — cross-platform audio I/O
- [`ndarray`](https://github.com/rust-ndarray/ndarray) — numerical arrays
- [`ringbuf`](https://github.com/agerasev/ringbuf) — lock-free ring buffer
- [`bit-set`](https://github.com/contain-rs/bit-set) — efficient note-mask representation

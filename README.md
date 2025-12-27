# Knitlang V2 🧶

Knitlang is a tiny, playful programming language inspired by knitting terms and patterns. This repository (Knitlang v2) contains a minimal Rust implementation of a lexer, parser, and interpreter that demonstrates the language's core ideas and provides a REPL and file runner.

---

## ✨ Features

- Knitting-themed keywords: `cast_on`, `knit`, `purl`, `bind_off`, `repeat`.
- Integer arithmetic expressions (`+`, `-`, `*`, `/`).
- Simple variable environment.
- REPL for interactive experimentation and file-based execution.

## 🚀 Quick start

Requirements: Rust and Cargo (1.XX+)

1. Build and run the REPL:

```bash
cargo run
```

2. Run a Knitlang source file:

```bash
cargo run -- path/to/program.knit
```

3. Run the bundled example (from `examples/`):

```bash
cargo run -- --example hello
```

4. Start the REPL explicitly with a flag:

```bash
cargo run -- --repl
```

## 🧩 Example program

Save this as `examples/hello.knit` and run it with `cargo run -- examples/hello.knit`:

```knit
cast_on stitches = 0;
repeat 3 {
  knit stitches = stitches + 1;
  purl stitches;
}
bind_off;
```

Output:
```
1
2
3
```

## 🛠️ Language overview

- `cast_on <name> = <expr>;` — create/initialize a variable.
- `knit <name> = <expr>;` — assign/update a variable.
- `purl <expr>;` — evaluate an expression and print it (used here for demonstration).
- `repeat <expr> { ... }` — repeat a block a fixed number of times.
- `bind_off;` — stop execution early (used like `break`).

## 📚 Next steps / TODO

- Add more knitting primitives (`yo`, `ssk`, pattern macros).
- Support lists/rows for representing stitches and patterns.
- Add tests and CI, a standard library, and more example patterns.

## Contributing

Contributions are welcome! Open issues for feature requests or bugs, and send PRs for improvements. Keep changes small and add tests where possible.

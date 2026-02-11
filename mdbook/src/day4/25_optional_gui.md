# Chapter 25: (Optional) Desktop GUI with egui

This chapter is a stretch goal for students who finish the main Day 4 exercises early. It is lighter on exercises and structured as a guided walkthrough rather than a hands-on coding session.

The goal is to show how imgforge could gain a desktop GUI mode using egui -- without rewriting any of the existing transform logic. The same core library that powers the CLI (Chapter 20), the FFI backend (Chapter 21), and the HTTP server (Chapter 22) also drives the GUI. This is conditional compilation at scale: an entire module, with GPU rendering and font rasterization dependencies, hidden behind a single feature flag.

egui is a pure-Rust immediate-mode GUI library. It has no system-level dependencies beyond a graphics backend (which `eframe` provides). It compiles on Windows, macOS, and Linux without any platform-specific setup -- making it a practical choice for cross-platform tools.

## 1. Feature Flag Setup

### Cargo.toml Additions

Add `eframe` and `egui` as optional dependencies and declare a `gui` feature that enables both:

```toml
[features]
gui = ["dep:eframe", "dep:egui"]

[dependencies]
eframe = { version = "0.30", optional = true }
egui = { version = "0.30", optional = true }
```

With this setup, the default `cargo build` does not pull in any GUI dependencies. Only an explicit `cargo build --features gui` enables them.

### Conditional Module Declaration

In `lib.rs`, gate the entire GUI module behind the feature flag:

```rust,ignore
#[cfg(feature = "gui")]
pub mod gui;
```

This means the `gui.rs` file is not even compiled unless the `gui` feature is active. Any compilation errors inside `gui.rs` will not affect normal builds -- a useful property when platform-specific code is involved.

### Building and Running

```bash
# Normal build -- no GUI code compiled
cargo build

# GUI build -- pulls in eframe, egui, and their transitive dependencies
cargo build --features gui

# Run the GUI directly
cargo run --features gui -- gui
```

You would add a `gui` subcommand to your `clap` CLI definition, gated behind `#[cfg(feature = "gui")]`, so it only appears when the feature is enabled.

## 2. Minimal egui Window

Create `src/gui.rs` with the following content:

```rust,ignore
#![cfg(feature = "gui")]

use eframe::egui;

pub fn run_gui() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "imgforge",
        options,
        Box::new(|_cc| Ok(Box::new(ImgforgeApp::default()))),
    )
}

#[derive(Default)]
struct ImgforgeApp {
    input_path: String,
    status: String,
}

impl eframe::App for ImgforgeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("imgforge");
            ui.horizontal(|ui| {
                ui.label("Image path:");
                ui.text_edit_singleline(&mut self.input_path);
            });
            if ui.button("Grayscale").clicked() {
                // Call the same transform library used by CLI and server
                self.status = match process_file(&self.input_path) {
                    Ok(()) => "Done!".to_string(),
                    Err(e) => format!("Error: {e}"),
                };
            }
            ui.label(&self.status);
        });
    }
}

fn process_file(path: &str) -> crate::Result<()> {
    let backend = crate::transform::default_backend();
    let data = std::fs::read(path)?;
    let output = backend.apply(&data, &crate::Operation::Grayscale)?;
    let out_path = format!("{path}.gray.png");
    std::fs::write(&out_path, output)?;
    Ok(())
}
```

Key points about this code:

- `ImgforgeApp` is a plain struct -- no inheritance, no base class, no widget tree. The entire UI state lives in two `String` fields.
- `process_file` calls into the same `crate::transform` module used by the CLI and the HTTP server. There is no GUI-specific image processing code.
- The `update` method is called every frame. The UI is rebuilt from scratch each time. This is the defining characteristic of immediate-mode GUI.

## 3. Immediate Mode GUI -- Comparison with Retained Mode

If you have worked with WPF, WinForms, Qt, or any other traditional GUI framework, egui will feel different. Those frameworks use a **retained-mode** model: you create widget objects, bind them to data, and the framework manages updates. egui uses an **immediate-mode** model: you describe the entire UI every frame, and the library handles rendering.

| Retained mode (WPF, Qt) | Immediate mode (egui) |
|--------------------------|----------------------|
| Create widget objects, bind data | Rebuild UI every frame |
| Event handlers, callbacks | Poll-based: `if button.clicked()` |
| Complex state management | Simple -- state is just your struct |
| Layout computed once | Layout recomputed each frame |
| Rich styling and theming | Functional but minimal styling |
| Large framework surface area | Small API, fast to learn |

Why immediate mode works well for developer tools:

- **Fast iteration** -- change a label, recompile, see it immediately. No XAML or designer files.
- **No state synchronization bugs** -- the UI is always a direct function of your data. There is no stale binding or missed `PropertyChanged` event.
- **Simple mental model** -- if you can write a function, you can write a UI. No need to learn a widget hierarchy.

The tradeoff is that immediate mode is less efficient for complex, heavily styled UIs. For a tool like imgforge, that tradeoff is overwhelmingly favorable.

## 4. Binary Size Implications

Adding a GUI framework has a measurable impact on the compiled binary:

```bash
cargo build --release                    # ~5 MB
cargo build --release --features gui     # ~15 MB (adds GPU rendering, fonts, etc.)
```

The additional 10 MB comes from:

- **GPU rendering backend** -- eframe includes `wgpu` or `glow` for hardware-accelerated rendering
- **Font rasterization** -- egui bundles a default font and a text shaping engine
- **Windowing** -- platform-specific window creation and event loop code

This is precisely why the GUI is behind a feature flag. A server deployment of imgforge has no use for GPU rendering or font rasterization. A CI/CD pipeline running batch image transforms should not pay the compile-time or binary-size cost of a desktop GUI. Feature flags let each deployment target pull in only what it needs.

## 5. When to Use Feature Flags vs Separate Crates

imgforge uses feature flags to gate the GUI, the FFI backend, and the server. This is a reasonable choice for a learning project, but production codebases often take a different approach.

| Approach | Pros | Cons |
|----------|------|------|
| **Feature flags** (single crate) | Simple, one `Cargo.toml`, shared types | Feature interactions can be complex, all code in one repo |
| **Workspace** (separate crates) | Independent versioning, cleaner dependency graphs, better for teams | More boilerplate, cross-crate type sharing requires a `core` crate |

imgforge stays as a single crate because it is a learning project and the simplicity is valuable. In a real production setting, you would likely split it into separate crates:

```text
imgforge/
  imgforge-core/      # Transform traits, error types, Operation enum
  imgforge-cli/       # clap CLI binary
  imgforge-server/    # Axum HTTP server binary
  imgforge-gui/       # eframe desktop GUI binary
```

Each binary crate depends on `imgforge-core` and brings in only the dependencies it needs. The CLI does not link against `eframe`. The GUI does not link against `axum`. The core library has no binary-specific dependencies at all.

This is the same pattern used by major Rust projects. For example, `ripgrep` separates its core search logic (`grep-regex`, `grep-searcher`) from its CLI binary (`rg`).

## 6. Exercise (Light)

This exercise is intentionally lighter than previous chapters. There are no test requirements.

**Task 1: Add an operation dropdown.**
Modify the `update` method to include a `ComboBox` that lets the user select an operation (Grayscale, Blur, Resize) instead of hardcoding Grayscale:

```rust,ignore
egui::ComboBox::from_label("Operation")
    .selected_text(format!("{:?}", self.selected_op))
    .show_ui(ui, |ui| {
        ui.selectable_value(&mut self.selected_op, Operation::Grayscale, "Grayscale");
        ui.selectable_value(
            &mut self.selected_op,
            Operation::Blur { sigma: 3.0 },
            "Blur",
        );
        ui.selectable_value(
            &mut self.selected_op,
            Operation::Resize { width: 800, height: 600 },
            "Resize",
        );
    });
```

You will need to add a `selected_op: Operation` field to `ImgforgeApp` and derive or implement `Default` and `PartialEq` for your `Operation` enum (egui's `selectable_value` requires `PartialEq` to determine which item is selected).

**Task 2 (optional): Add a file browser.**
Use the `rfd` crate (Rusty File Dialogs) to add a "Browse" button that opens a native file picker:

```toml
[dependencies]
rfd = { version = "0.15", optional = true }

[features]
gui = ["dep:eframe", "dep:egui", "dep:rfd"]
```

```rust,ignore
if ui.button("Browse...").clicked() {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter("Images", &["jpg", "jpeg", "png", "bmp"])
        .pick_file()
    {
        self.input_path = path.display().to_string();
    }
}
```

## 7. Day 4 Wrap-Up

Over the course of Day 4, you built a complete Rust application from an empty `cargo new` to a multi-mode tool with CLI, server, and GUI interfaces. Here is what each chapter contributed:

| Chapter | What was added | Key concept |
|---------|---------------|-------------|
| Ch20 | CLI tool, project structure | Cargo, error handling, clap |
| Ch21 | FFI backend | Feature flags, unsafe abstraction |
| Ch22 | HTTP server | Axum, async, IntoResponse |
| Ch23 | Concurrent processing | spawn_blocking, Arc/Mutex, jobs |
| Ch24 | Tests, batch mode | Integration tests, coverage |
| Ch25 | Desktop GUI | Conditional compilation at scale |

Every stage reused the same core transform library. The `Transform` trait, the `Operation` enum, and the error types defined in Chapter 20 were never rewritten -- they were extended and composed. This is the payoff of trait-based design: new consumers (CLI, server, GUI) plug into existing abstractions without modifying them.

### Patterns Worth Remembering

The patterns you used today appear throughout real-world Rust codebases:

- **Trait-based backends** -- define behavior as a trait, swap implementations at compile time or runtime. Used by database drivers, HTTP clients, and serialization libraries.
- **Explicit error types** -- a crate-level `Error` enum with `From` implementations for automatic conversion. Used by virtually every well-maintained Rust library.
- **Thin `main.rs`** -- the binary is a thin wrapper that parses arguments and calls into library code. All logic lives in `lib.rs` and its modules. This makes the code testable without running the binary.
- **Feature-gated modules** -- optional functionality behind `#[cfg(feature = "...")]`. Used by `serde`, `tokio`, `reqwest`, and hundreds of other crates to keep default compile times and binary sizes small.

You have built a production-quality Rust application from scratch. The same architectural decisions -- trait-based backends, explicit error types, thin `main.rs`, feature-gated modules -- appear in the Rust projects you will encounter and contribute to in your day-to-day work.

# FAQ

## What is this project about?
`dcr` is a tool for managing C/C++ projects and their dependencies.
It is inspired by Cargo, but it is not a direct equivalent of Makefile, CMake, or other classic build systems.

The goal is to provide a simpler and easier-to-configure development workflow.

## How is the project developed?
The project is written in Rust and is primarily developed by one maintainer, Dexoron.

AI tools are used during development, but they do not write the project end-to-end. Their role is limited to supporting specific tasks (for example, suggestions, wording checks, and translations).

## Which tools are used for development?
Main development stack:
- `RustRover` 2025 - development IDE.
- `Codex` (`GPT-5.3-Codex (medium)`) - primary AI tool, integrated via JetBrains AI Chat.
- `Rust` - main project language.
- `Cargo` - tool for building, testing, and dependency management in Rust projects.

## Is the project open source?
Yes, the project is open source.
You can propose ideas, report issues, and contribute code changes.

The project is distributed under the MIT license (see `LICENSE`).

## How can I contribute?
Thank you for contributing.
You can:
- help with code;
- help with translations;
- help with documentation;
- suggest an idea or report a bug.

### If you want to submit changes (code, translation, documentation)
1. Fork the required repository: `dexoron/dcr` (main project) or `dexoron/dcr-site` (project website).
2. Make your changes following the rules in `CONTRIBUTING.md`.
3. Push changes to your fork.
4. Open a Pull Request.
5. Wait for review.

### If you want to report a problem or suggest an idea
1. Open the relevant repository: `dexoron/dcr` or `dexoron/dcr-site`.
2. Go to the `Issues` tab.
3. Create an Issue and specify its type (bug, suggestion, question/help).
4. Describe the problem or idea clearly and concisely.
5. Wait for feedback.

## How do I install `dcr`?
Available installation methods may vary by project version.
Recommended path:
- check instructions in `README.md`;
- if there is no prebuilt package for your platform, build from source using `cargo build --release`.

## How can I get started quickly?
Minimal workflow:
1. Install `dcr`.
2. Create a project (or open an existing C/C++ project).
3. Initialize `dcr` configuration.
4. Add project dependencies.
5. Build using `dcr`.

For detailed commands and examples, see `README.md` and the project documentation.

## What is the project status?
`dcr` is under active development.
This means some interfaces and behaviors may change between versions.

Before upgrading, check the changelog and release notes.

## Which systems does `dcr` support?
`dcr` targets modern C/C++ development environments.
Actual compatibility depends on the `dcr` version, operating system, and toolchain.

Before starting, check documentation for:
- supported operating systems;
- supported compilers and linkers;
- version-specific limitations.

## How is `dcr` different from Make/CMake?
`dcr` does not aim to be a direct replacement for Make/CMake.
Its main focus is simplifying project and dependency management with a Cargo-like experience.

If you need low-level, full control over build steps, classic tools may be a better fit.
If you want a simpler and more uniform configuration model, `dcr` may be more convenient.

## Where can I find documentation and examples?
Primary sources:
- `README.md` in the repository;
- project documentation (see the project website);
- `Issues` for known problems and discussions.

## How do I report a bug properly?
To make a bug report easy to reproduce, include:
1. `dcr` version.
2. OS and compiler/toolchain version.
3. Steps to reproduce.
4. Expected behavior.
5. Actual behavior and error output.
6. Minimal project example (if possible).

Submit bugs and issues through `Issues` in the relevant repository.

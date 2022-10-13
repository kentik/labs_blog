---
date: 2022-10-13
author: "Will Glozer"
title: "Adventures In Cross Compilation"
slug: "adventures-in-cross-compilation"
summary: "Cross compilation can solve many of the complexities involved in building & testing executables for many different target systems"
---

The days of the x86 monoculture are receding rapidly and today's software is expected to run on [AArch64][aarch64] and [x86-64][x86-64] CPUs at a minimum, with 32-bit [ARM][arm] remaining common and [RISC-V][risc-v] emerging as a popular open architecture. A diverse array of operating systems runs atop this hardware including [FreeBSD][freebsd], [Linux][linux], [macOS][macos], and [Windows][windows]. Maintaining build & test environments for every permutation of CPU architecture and OS quickly becomes unmanageable, what's a programmer to do?

The simplest solution is perhaps to write the program in a platform-independent language like [F#][fsharp], [Java][java], or [Python][python]. [Write once, run anywhere][wora] has existed in various forms for decades, and can be an effective solution when the entire program, including all dependencies, is written in a platform-independent language. However most programs of any significant size will want to leverage existing libraries written in other languages, particularly C and C++. That code is excluded from the portability offered by the language runtime and must still be built and tested for every supported target system.

This article will focus on systems programming languages like [Go][go], [Rust][rust], and [Zig][zig] that compile to native code and have built-in support for cross platform builds. These languages bundle their standard library code into the compiled executable, and other native dependencies can be built with a [cross compiler][x-compiler] and [statically linked][static-lib] to produce self-contained executables. In the ideal case this allows distribution of one executable per `(architecture, operating system)` pair. Testing requires environments capable of running those executables, and [musl libc][musl] can be used to produce fully static binaries for Linux which can be tested for all architectures using QEMU's [user space emulator][qemu-user] on a single host system.

# Cross Compilation

Modern systems programming languages like [Go][go], [Rust][rust], and [Zig][zig] have built-in support for [cross compilation][x-compiler], allowing a single host system to build executables for all supported target systems. [Rust][rust-x-c] and [Zig][zig-x-c] use command line flags to specify the target, `--target aarch64-unknown-linux-gnu` and `-target aarch64v8-linux-gnu` respectively, while Go uses environment variables, `GOARCH=arm64 GOOS=linux go build ...`.

[LLVM/Clang][llvm-x-c] offers similar support for cross-compiling C and C++, however the user must still supply compiled libraries and headers for all dependencies, including fundamentals like the C runtime, for the target platform. [Go][go-std], [Rust][rust-std], and Zig have large standard libraries that provide platform-independent abstractions, and their toolchains manage fetching and linking that library code automatically when cross compiling.

Nevertheless decades of effort has gone into developing high performance, battle-tested, libraries in C and C++ and reusing these libraries in new programs is valuable. The [musl-cross-make][musl-cm] project offers an easy way to build GCC cross compilers targeting Linux on every common architecture, and the [MinGW-w64][mingw-64] project offers cross compilers for Windows. Go and Rust can be configured to use these cross compilers for C and C++ dependencies, allowing a single host system to generate executables for itself, Linux, and Windows. Zig takes this a step further and can [import C headers][zig-call-c] and [compile C code][zig-build-c] directly.

# Static Linking

[Dynamic linking][dyn-link] has a number of advantages including being more [space-efficient][dyn-link-eff] on disk and in memory, and making library updates a simple matter of replacing the shared library. Security issues in critical libraries such as OpenSSL can be fixed by updating the library, without requiring new versions of all linked executables. However this also means that comprehensive testing requires environments with every expected permutation of all shared libraries. For most software this is impractical, and many configurations will not be tested until a user reports an issue at which point the developer must attempt to reproduce the user's environment.

[Static linking][static-lib] produces a self-contained executable that relies only on standard system libraries, or in the case of [musl libc][musl] a fully static binary with no dependencies aside from the kernel. A well-tested static executable can be expected to run correctly in a larger set of system configurations, both past and future, due to the reduced external surface area. Troubleshooting is greatly simplified, requiring only a machine architecture and operating system version matching the user's rather than exact versions of all dynamically linked libraries.

# Linux & musl

Minor exceptions aside, most operating systems come from a single source and every [FreeBSD 12.3][freebsd], [macOS 12.6][macos], or [Windows 11 22H2][windows] installation at the same patch level will generally have the same system libraries. Linux is a different story, there are many different [Linux distributions][linux-distro] including [Debian][linux-debian], [Fedora][linux-fedora], [Ubuntu][linux-ubuntu], and [Red Hat Enterprise Linux][linux-rhel]. Each distribution has its own custom set of system libraries. Most use the [GNU C library][glibc] but the version varies, and some such as [Alpine][linux-alpine] use [musl][musl] instead. So while it's easy to build an executable that runs on any AArch64 FreeBSD 12.3 or x86-64 Windows 11 system, Linux is another story and in the worst case could require building & testing binaries for every supported permutation of `(architecture, distribution type, distribution version)`.

However instead of dynamically linking C/C++/Go/Rust/Zig code to a specific Linux distribution's libc you can statically link [musl libc][musl] and create a fully static binary with no dependencies other than the Linux kernel. Linux has an extremely stable kernel-user space API and these binaries should run correctly on past and future versions, modulo bugs and use of features only available from specific kernel versions.

The [musl-cross-make][musl-cm] project offers an easy method of producing GCC cross compilation toolchains targeting Linux and linking to musl. GCC lacks native support for cross compilation, so one toolchain must be built for each target architecture. Then it's a simple matter of configuring the Go or Rust toolchains to use these when building and linking C or C++ code. [Rich Felker][rich-felker] is the main author and maintainer of [musl libc][musl] and [musl-cross-make][musl-cm], and Kentik is happy to be able to [sponsor][rich-sponsor] his work as one of our contributions to the open source ecosystem.

# Testing

Thus far this article has focused on building executables. Combining the Go/Rust/Zig/etc toolchains with GCC cross-compilation toolchains, or LLVM/Clang, allows a single host system, such as a developer's workstation or CI environment, to generate executables for many target platforms. This eliminates the complexity of needing a build environment for every supported target, but what of testing environments? Many targets will require an installation of the target operating system for testing, whether in a virtual machine or on physical hardware. If the executable has no platform-specific dependencies then a stock installation of the OS may suffice and wrangling a collection of these may be an unfortunate hassle, but not unmanageable.

For Linux there is a better solution. [QEMU][qemu-user]'s [user space emulator][qemu-user] can execute Linux binaries for many different architectures on a single host system. If the executable has dynamically linked dependencies then versions of these for the correct architecture must be supplied. However a fully statically linked executable can be run as-is, for example a x86-64 host system can run 32-bit ARMv7 binaries with `qemu-arm-static <binary>`, AArch64 binaries with `qemu-aarch64-static <binary>`, MIPS64 binaries with `qemu-mips64-static <binary>`, etc.

# Rust Example

A practical application of the topics discussed in this article is available as a [small Rust program][x-ex-repo]. In this example [main.rs][x-ex-main] references an exernal function, `now` which writes the current time to a caller-supplied buffer. `now` is a C function implemented in [now.c][x-ex-now-c]. With the Rust toolchain, Docker, and a Rust tool called [cross][rust-cross] installed, the following commands will build fully static Linux executables for AArch64, ARMv7, and MIPS64:

```
cross build --target aarch64-unknown-linux-musl
cross build --target mips64-unknown-linux-muslabi64
cross build --target armv7-unknown-linux-musleabihf
```

Running one of these executables is a simple matter of invoking the appropriate QEMU binary:

```
$ file target/mips64-unknown-linux-muslabi64/debug/cross-compile
target/mips64-unknown-linux-muslabi64/debug/cross-compile: ELF 64-bit MSB executable, MIPS, MIPS64 rel2 version 1 (SYSV), statically linked, with debug_info, not stripped

$ qemu-mips64-static target/mips64-unknown-linux-muslabi64/debug/cross-compile
current time: 2022-10-13 03:00:23
```

An even simpler solution is to install appropriate [binfmt_misc][binfmt_misc] package which allows the executables to be run directly:

```
$ file target/armv7-unknown-linux-musleabihf/debug/cross-compile
target/armv7-unknown-linux-musleabihf/debug/cross-compile: ELF 32-bit LSB executable, ARM, EABI5 version 1 (SYSV), statically linked, with debug_info, not stripped

$ target/armv7-unknown-linux-musleabihf/debug/cross-compile
current time: 2022-10-13 03:05:12
```

[cross][rust-cross] is a drop-in replacement for `cargo` that executes the build in a Docker container pre-configured with a cross-compilation toolchain for the specified target. However using cross is completely optional and, with the appropriate [musl-cross-make][musl-cm] cross-compilers present in `PATH`, the following commands will have the same effect without needing Docker:

```
cargo build --target aarch64-unknown-linux-musl
cargo build --target mips64-unknown-linux-muslabi64
cargo build --target armv7-unknown-linux-musleabihf
```

[aarch64]:      https://en.wikipedia.org/wiki/AArch64
[arm]:          https://en.wikipedia.org/wiki/ARM_architecture_family
[binfmt_misc]:  https://en.wikipedia.org/wiki/Binfmt_misc
[dyn-link]:     https://en.wikipedia.org/wiki/Dynamic_linker
[dyn-link-eff]: https://en.wikipedia.org/wiki/Dynamic_linker#Efficiency
[freebsd]:      https://en.wikipedia.org/wiki/FreeBSD
[fsharp]:       https://en.wikipedia.org/wiki/F_Sharp_(programming_language)
[glibc]:        https://en.wikipedia.org/wiki/Glibc
[go]:           https://en.wikipedia.org/wiki/Go_(programming_language)
[java]:         https://en.wikipedia.org/wiki/Java_(programming_language)
[kentik]:       https://www.kentik.com/
[kentik-labs]:  https://kentiklabs.com/
[linux]:        https://en.wikipedia.org/wiki/Linux
[linux-alpine]: https://en.wikipedia.org/wiki/Alpine_Linux
[linux-debian]: https://en.wikipedia.org/wiki/Debian
[linux-distro]: https://en.wikipedia.org/wiki/Linux_distribution
[linux-fedora]: https://en.wikipedia.org/wiki/Fedora_Linux
[linux-ubuntu]: https://en.wikipedia.org/wiki/Ubuntu
[linux-rhel]:   https://en.wikipedia.org/wiki/Red_Hat_Enterprise_Linux
[llvm-x-c]:     https://clang.llvm.org/docs/CrossCompilation.html
[macos]:        https://en.wikipedia.org/wiki/MacOS
[mingw-64]:     https://www.mingw-w64.org/
[musl]:         https://en.wikipedia.org/wiki/Musl
[musl-cm]:      https://github.com/richfelker/musl-cross-make
[python]:       https://en.wikipedia.org/wiki/Python_(programming_language)
[qemu-user]:    https://www.qemu.org/docs/master/user/main.html
[rich-felker]:  https://github.com/richfelker
[rich-sponsor]: https://github.com/sponsors/richfelker
[risc-v]:       https://en.wikipedia.org/wiki/RISC-V
[rust]:         https://en.wikipedia.org/wiki/Rust_(programming_language)
[rust-cross]:   https://github.com/cross-rs/cross
[rust-std]:     https://doc.rust-lang.org/std/
[rust-x-c]:     https://rust-lang.github.io/rustup/cross-compilation.html
[static-lib]:   https://en.wikipedia.org/wiki/Static_library
[static-exe]:   https://en.wikipedia.org/wiki/Static_build
[go-std]:       https://pkg.go.dev/std
[windows]:      https://en.wikipedia.org/wiki/Microsoft_Windows
[wora]:         https://en.wikipedia.org/wiki/Write_once,_run_anywhere
[x86-64]:       https://en.wikipedia.org/wiki/X86-64
[x-compiler]:   https://en.wikipedia.org/wiki/Cross_compiler
[x-ex-repo]:    https://github.com/kentik/labs_blog/tree/main/examples/cross-compile
[x-ex-cargo]:   https://github.com/kentik/labs_blog/blob/main/examples/cross-compile/.cargo/config.toml
[x-ex-main]:    https://github.com/kentik/labs_blog/blob/main/examples/cross-compile/src/main.rs
[x-ex-now-c]:   https://github.com/kentik/labs_blog/blob/main/examples/cross-compile/src/now.c
[x-ex-script]:  https://github.com/kentik/labs_blog/blob/main/examples/cross-compile/build.rs
[zig]:          https://en.wikipedia.org/wiki/Zig_(programming_language)
[zig-build-c]:  https://ziglang.org/learn/overview/#zig-is-also-a-c-compiler
[zig-call-c]:   https://ziglang.org/learn/overview/#integration-with-c-libraries-without-ffibindings
[zig-x-c]:      https://ziglang.org/learn/overview/#cross-compiling-is-a-first-class-use-case

# Chapter 17: Embedded Rust

Rust runs on microcontrollers. The same ownership model that secures a web service also prevents the memory bugs that make embedded development painful — and `no_std` lets you drop the standard library entirely, leaving only the language core. This chapter is a short tour: what `no_std` means, the `embedded-hal` abstraction layer, and a blinking LED on a typical microcontroller.

## Learning Objectives

- Understand `#![no_std]` and the core vs. alloc vs. std layers.
- Use `embedded-hal` traits to write portable peripheral code.
- Cross-compile for a microcontroller target.
- Blink an LED with a hardware-abstraction crate (PAC/HAL).
- Know where to go deeper into the embedded ecosystem.

---

## 17.1 The three layers: core, alloc, std

Rust code targets one of three layers, controlled by attributes:

| Layer | Provides | Attribute | Typical use |
|-------|----------|-----------|-------------|
| `std` | Heap, threads, files, networking | (default) | Desktop, server. |
| `alloc` | `Box`, `Vec`, `String`, `Arc` | `#![no_std]` + `extern crate alloc` | OS kernels, larger embedded. |
| `core` | Slices, iterators, `Option`/`Result` | `#![no_std]` | Microcontrollers, bootloaders. |

A `#![no_std]` binary drops `std` and links only `core` (and optionally `alloc`). Anything you write against `core` works everywhere — including `std` programs — which is why library authors prefer `no_std`-compatible code where feasible.

```rust
#![no_std]

// Only `core` is available: no Vec, no String, no println!, no threads.
pub fn sum(slice: &[i32]) -> i32 {
    slice.iter().copied().sum()
}
```

---

## 17.2 `embedded-hal`: portable traits

The genius of the embedded ecosystem is `embedded-hal`, a set of traits that describe peripherals generically: a GPIO pin, a serial port, an I²C bus, a timer. Code written against these traits runs unchanged on any chip whose HAL implements them.

```rust
use embedded_hal::digital::OutputPin;

// This function blinks any pin that implements OutputPin — any chip, any HAL.
pub fn blink<P: OutputPin>(pin: &mut P, count: u8) {
    for _ in 0..count {
        let _ = pin.set_high();
        // delay omitted for brevity
        let _ = pin.set_low();
    }
}
```

Because the trait is generic, the same `blink` works on an STM32, an ESP32, or an nRF52 — only the concrete pin type changes at the call site.

---

## 17.3 The PAC, HAL, and BSP stack

Embedded Rust is layered:

- **PAC** (Peripheral Access Crate) — generated from the chip's SVD file; raw register access at addresses.
- **HAL** (Hardware Abstraction Layer) — implements `embedded-hal` traits on top of the PAC, with a safe API.
- **BSP** (Board Support Package) — pins and peripherals wired for a specific board (e.g. "the user LED is on PB5").

You usually write code against the HAL/BSP, dropping to the PAC only for unusual registers.

---

## 17.4 Cross-compiling

Rust cross-compiles by installing a target and pointing cargo at it:

```bash
# Add a target (example: Cortex-M4F, common on STM32 / nRF52).
rustup target add thumbv7em-none-eabihf

# Build without standard library, without an entry point defined by std.
cargo build --release --target thumbv7em-none-eabihf
```

The target triple `thumbv7em-none-eabihf` encodes the architecture, ABI, and hard-float. A `#![no_std]` binary also needs a custom entry point and a linker script; the `cortex-m-rt` crate and `cortex-m-quickstart` template provide these.

---

## 17.5 A blinky in outline

The shape of a blinky program (details vary by HAL):

```rust
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_hal::digital::OutputPin;
use panic_halt as _;          // define a panic handler: halt

#[entry]
fn main() -> ! {
    let (mut led, mut delay) = board::take_peripherals();

    loop {
        led.set_high();
        delay.delay_ms(500);
        led.set_low();
        delay.delay_ms(500);
    }
}
```

Three things stand out:

1. `#![no_main]` — there is no standard `main`; `#[entry]` from `cortex-m-rt` defines the reset handler.
2. `panic_halt as _` — a `#![no_std]` binary must supply a panic handler; this one halts the CPU.
3. `main -> !` — embedded `main` never returns; it loops forever.

---

## 17.6 Async on microcontrollers

`embedded-hal` now has async variants, and executors like `embassy` run futures on a microcontroller without an OS. This lets you write non-blocking drivers — reading a sensor while an LED blinks — with the same `async`/`await` you use on a server, on a chip with tens of kilobytes of RAM.

---

## 17.7 Resources

- **The Embedded Rust Book** — `docs.rust-embedded.org/book` — the canonical tutorial.
- **`embedded-hal`** docs — the trait reference.
- **`probe-rs`** — flashing and debugging via a debug probe, replacing vendor toolchains.
- **`embassy`** — async embedded framework, growing fast.

---

## 17.8 Summary

Embedded Rust trades `std` for `core`, writes portable drivers against `embedded-hal`, and cross-compiles to bare-metal targets with the same `cargo` you already use. The result is microcontroller firmware with the same memory-safety guarantees as server code — a meaningful change for a domain long plagued by buffer overflows and dangling pointers.

### Exercises

1. Write a `#![no_std]` function `fn count_ones(bytes: &[u8]) -> u32` that counts set bits, and unit-test it with `cargo test` on your host.
2. Install the `thumbv7em-none-eabihf` target and confirm a `#![no_std]` crate builds for it.
3. Read the first chapter of the Embedded Rust Book and identify the PAC, HAL, and BSP for a board you own.

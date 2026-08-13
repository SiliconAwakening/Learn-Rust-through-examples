# 第17章：嵌入式 Rust

Rust 不仅能跑在服务器上，也能跑在单片机里。所有权模型在 Web 服务里防止的内存错误，正是嵌入式开发长期以来的痛点；而 `no_std` 让你可以完全抛开标准库，只保留语言内核。本章是一次短途导览：`no_std` 意味着什么、`embedded-hal` 抽象层如何让代码可移植，以及如何在典型单片机上点亮一颗 LED。

## 学习目标

- 理解 `#![no_std]` 以及 core / alloc / std 三个层次。
- 使用 `embedded-hal` trait 编写可移植的外设驱动。
- 为单片机目标进行交叉编译。
- 用 HAL/PAC crate 实现一个闪烁 LED 的程序。
- 知道嵌入式生态的深入方向。

---

## 17.1 三个层次：core、alloc、std

Rust 代码可以面向三个层次之一，由属性控制：

| 层次 | 提供内容 | 属性 | 典型场景 |
|------|----------|------|----------|
| `std` | 堆、线程、文件、网络 |（默认）| 桌面、服务端 |
| `alloc` | `Box`、`Vec`、`String`、`Arc` | `#![no_std]` + `extern crate alloc` | OS 内核、较大的嵌入式系统 |
| `core` | 切片、迭代器、`Option`/`Result` | `#![no_std]` | 单片机、引导加载器 |

`#![no_std]` 二进制会丢弃 `std`，只链接 `core`（可选地链接 `alloc`）。凡是只依赖 `core` 写成的代码，在任何地方都能用——包括 `std` 程序——所以库作者在可行时都倾向于写 `no_std` 兼容的代码。

```rust
#![no_std]

// 只有 core 可用：没有 Vec、没有 String、没有 println!、没有线程。
pub fn sum(slice: &[i32]) -> i32 {
    slice.iter().copied().sum()
}
```

---

## 17.2 `embedded-hal`：可移植的 trait

嵌入式生态的精妙之处在于 `embedded-hal`：一组用 trait 通用描述外设的接口——GPIO 引脚、串口、I²C 总线、定时器。只要 HAL 实现了这些 trait，针对 trait 写的代码在任何芯片上都能原样运行。

```rust
use embedded_hal::digital::OutputPin;

// 这个函数能让任何实现了 OutputPin 的引脚闪烁——任何芯片、任何 HAL。
pub fn blink<P: OutputPin>(pin: &mut P, count: u8) {
    for _ in 0..count {
        let _ = pin.set_high();
        // 延时此处省略
        let _ = pin.set_low();
    }
}
```

因为用了泛型，同一段 `blink` 既能在 STM32 上跑，也能在 ESP32、nRF52 上跑——改变的只是调用处的具体引脚类型。

---

## 17.3 PAC、HAL、BSP 三层结构

嵌入式 Rust 是分层的：

- **PAC**（外设访问 crate）——由芯片的 SVD 文件生成，提供按地址访问寄存器的原始接口。
- **HAL**（硬件抽象层）——在 PAC 之上实现 `embedded-hal` trait，提供安全 API。
- **BSP**（板级支持包）——针对某块板子预先接好引脚与外设（例如“用户 LED 在 PB5”）。

通常你面向 HAL/BSP 写代码，只有用到特殊寄存器时才下沉到 PAC。

---

## 17.4 交叉编译

Rust 的交叉编译只需安装目标并让 cargo 指向它：

```bash
# 添加一个目标（示例：Cortex-M4F，常见于 STM32 / nRF52）
rustup target add thumbv7em-none-eabihf

# 不带标准库、不带 std 定义的入口点进行构建
cargo build --release --target thumbv7em-none-eabihf
```

目标三元组 `thumbv7em-none-eabihf` 编码了架构、ABI 与硬浮点。`#![no_std]` 二进制还需要自定义入口点和链接脚本，`cortex-m-rt` crate 与 `cortex-m-quickstart` 模板提供了这些。

---

## 17.5 闪烁程序的结构

一个 blinky 程序的大致形状（细节随 HAL 而异）：

```rust
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_hal::digital::OutputPin;
use panic_halt as _;          // 提供 panic 处理器：停机

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

三点值得注意：

1. `#![no_main]`——没有标准 `main`；`cortex-m_rt::entry` 定义了复位处理函数。
2. `panic_halt as _`——`#![no_std]` 二进制必须提供 panic 处理器；这里让 CPU 停机。
3. `main -> !`——嵌入式 `main` 永不返回，它死循环。

---

## 17.6 单片机上的异步

`embedded-hal` 已有异步变体，`embassy` 这类执行器能在没有 OS 的单片机上跑 future。于是你可以用与服务端相同的 `async`/`await` 写非阻塞驱动——一边读传感器一边让 LED 闪烁——而芯片只有几十 KB 内存。

---

## 17.7 延伸资源

- **The Embedded Rust Book**——`docs.rust-embedded.org/book`——权威教程。
- **`embedded-hal`** 文档——trait 参考。
- **`probe-rs`**——通过调试探针烧录与调试，替代厂商工具链。
- **`embassy`**——异步嵌入式框架，发展迅速。

---

## 17.8 小结

嵌入式 Rust 用 `core` 换掉 `std`，面向 `embedded-hal` 写可移植驱动，并用你早已熟悉的 `cargo` 交叉编译到裸机目标。结果是具备与服务端同等内存安全保证的单片机固件——对一个长期被缓冲区溢出与悬垂指针困扰的领域而言，这是实实在在的改变。

### 练习

1. 写一个 `#![no_std]` 函数 `fn count_ones(bytes: &[u8]) -> u32` 统计置位比特数，并在主机上用 `cargo test` 做单元测试。
2. 安装 `thumbv7em-none-eabihf` 目标，确认一个 `#![no_std]` crate 能为之构建。
3. 阅读 Embedded Rust Book 第一章，为你手头的板子找出对应的 PAC、HAL、BSP。

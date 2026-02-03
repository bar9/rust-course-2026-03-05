# Chapter 17: Embedded HAL - Hardware Register Access & Volatile Memory

This chapter covers hardware abstraction in embedded Rust, focusing on memory-mapped I/O, volatile access patterns, and the embedded-hal ecosystem. These concepts are essential for writing safe, portable embedded code.

## Part 1: Why Volatile Access Matters

### The Compiler Optimization Problem

When accessing regular memory, the compiler assumes it has complete control and can optimize away "redundant" operations:

```rust
// Regular memory access - compiler can optimize
fn regular_memory() {
    let mut value = 0u32;

    value = 1;  // Compiler might optimize away
    value = 2;  // Only this write matters
    value = 3;  // And this one

    let x = value;  // Reads 3
    let y = value;  // Compiler might reuse x instead of reading again
}
```

Hardware registers are different - they're windows into hardware state that can change independently:

```rust
// Hardware register at address 0x4000_0000
const GPIO_OUT: *mut u32 = 0x4000_0000 as *mut u32;

unsafe fn bad_gpio_control() {
    // ❌ WRONG: Compiler might optimize these away!
    *GPIO_OUT = 0b0001;  // Turn on LED 1
    *GPIO_OUT = 0b0010;  // Turn on LED 2
    *GPIO_OUT = 0b0100;  // Turn on LED 3

    // Compiler might only emit the last write!
}

unsafe fn good_gpio_control() {
    use core::ptr;

    // ✅ CORRECT: Volatile writes are never optimized away
    ptr::write_volatile(GPIO_OUT, 0b0001);  // Turn on LED 1
    ptr::write_volatile(GPIO_OUT, 0b0010);  // Turn on LED 2
    ptr::write_volatile(GPIO_OUT, 0b0100);  // Turn on LED 3

    // All three writes will happen!
}
```

### Memory-Mapped I/O Fundamentals

In embedded systems, hardware peripherals appear as memory addresses:

```rust
// ESP32-C3 GPIO registers (simplified)
const GPIO_BASE: usize = 0x6000_4000;

// GPIO output registers
const GPIO_OUT_W1TS: *mut u32 = (GPIO_BASE + 0x0008) as *mut u32;  // Set bits
const GPIO_OUT_W1TC: *mut u32 = (GPIO_BASE + 0x000C) as *mut u32;  // Clear bits
const GPIO_OUT: *mut u32 = (GPIO_BASE + 0x0004) as *mut u32;       // Direct write
const GPIO_IN: *const u32 = (GPIO_BASE + 0x003C) as *const u32;    // Read input

unsafe fn control_gpio() {
    use core::ptr;

    // Set pin 5 high (write 1 to set)
    ptr::write_volatile(GPIO_OUT_W1TS, 1 << 5);

    // Clear pin 5 (write 1 to clear - yes, really!)
    ptr::write_volatile(GPIO_OUT_W1TC, 1 << 5);

    // Read current pin states
    let pins = ptr::read_volatile(GPIO_IN);
    let pin5_state = (pins >> 5) & 1;
}
```

### Why Each Access Must Be Volatile

Hardware registers can change at any time due to:
- External signals (button presses, sensor readings)
- Hardware state machines (timers, DMA completion)
- Interrupt handlers modifying registers
- Peripheral operations completing

```rust
// Timer register that counts up automatically
const TIMER_COUNTER: *const u32 = 0x6002_0000 as *const u32;

unsafe fn wait_for_timeout() {
    use core::ptr;

    // ❌ WRONG: Compiler might read once and cache
    while *TIMER_COUNTER < 1000 {
        // Infinite loop - compiler assumes value never changes!
    }

    // ✅ CORRECT: Each read goes to hardware
    while ptr::read_volatile(TIMER_COUNTER) < 1000 {
        // Works correctly - reads actual hardware value
    }
}
```

## Part 2: Safe Register Abstractions

### Building Type-Safe Register Access

```rust
use core::marker::PhantomData;

/// Type-safe register wrapper
pub struct Register<T> {
    address: *mut T,
}

impl<T> Register<T> {
    pub const fn new(address: usize) -> Self {
        Self {
            address: address as *mut T,
        }
    }

    pub fn read(&self) -> T
    where
        T: Copy,
    {
        unsafe { core::ptr::read_volatile(self.address) }
    }

    pub fn write(&self, value: T) {
        unsafe { core::ptr::write_volatile(self.address, value) }
    }

    pub fn modify<F>(&self, f: F)
    where
        T: Copy,
        F: FnOnce(T) -> T,
    {
        self.write(f(self.read()));
    }
}

// Usage
const GPIO_OUT: Register<u32> = Register::new(0x4000_0004);

fn toggle_led() {
    GPIO_OUT.modify(|val| val ^ (1 << 5));  // Toggle bit 5
}
```

### Field Access with Bitfields

```rust
use modular_bitfield::prelude::*;

#[bitfield]
#[derive(Clone, Copy)]
pub struct TimerControl {
    pub enable: bool,      // Bit 0
    pub interrupt: bool,   // Bit 1
    pub mode: B2,         // Bits 2-3
    #[skip] __: B4,       // Bits 4-7 reserved
    pub prescaler: B8,    // Bits 8-15
    pub reload: B16,      // Bits 16-31
}

pub struct TimerPeripheral {
    control: Register<TimerControl>,
    counter: Register<u32>,
}

impl TimerPeripheral {
    pub fn configure(&self, prescaler: u8, reload: u16) {
        let mut ctrl = self.control.read();
        ctrl.set_prescaler(prescaler);
        ctrl.set_reload(reload);
        ctrl.set_enable(true);
        self.control.write(ctrl);
    }
}
```

## Part 3: PAC Generation with svd2rust

### What is an SVD File?

System View Description (SVD) files describe microcontroller peripherals in XML format. The `svd2rust` tool generates Rust code from these descriptions.

### Generated PAC Structure

```rust
// Generated by svd2rust from manufacturer SVD
pub mod gpio {
    use core::ptr;

    pub struct RegisterBlock {
        pub moder: MODER,     // Mode register
        pub otyper: OTYPER,   // Output type register
        pub ospeedr: OSPEEDR, // Output speed register
        pub pupdr: PUPDR,     // Pull-up/pull-down register
        pub idr: IDR,         // Input data register
        pub odr: ODR,         // Output data register
        pub bsrr: BSRR,       // Bit set/reset register
    }

    pub struct MODER {
        register: vcell::VolatileCell<u32>,
    }

    impl MODER {
        pub fn read(&self) -> u32 {
            self.register.get()
        }

        pub fn write(&self, value: u32) {
            self.register.set(value)
        }

        pub fn modify<F>(&self, f: F)
        where
            F: FnOnce(u32) -> u32,
        {
            self.write(f(self.read()));
        }
    }
}

// Safe peripheral access
pub struct Peripherals {
    pub GPIO: gpio::RegisterBlock,
}

impl Peripherals {
    pub fn take() -> Option<Self> {
        // Ensure single instance (singleton pattern)
        static mut TAKEN: bool = false;

        cortex_m::interrupt::free(|_| unsafe {
            if TAKEN {
                None
            } else {
                TAKEN = true;
                Some(Peripherals {
                    GPIO: gpio::RegisterBlock {
                        // Initialize with hardware addresses
                    },
                })
            }
        })
    }
}
```

### Using a PAC

```rust
use esp32c3_pac::{Peripherals, GPIO};

fn configure_gpio() {
    let peripherals = Peripherals::take().unwrap();
    let gpio = peripherals.GPIO;

    // Configure pin as output
    gpio.enable_w1ts.write(|w| w.bits(1 << 5));
    gpio.func5_out_sel_cfg.write(|w| w.out_sel().bits(0x80));

    // Set pin high
    gpio.out_w1ts.write(|w| w.bits(1 << 5));
}
```

**Modern Alternatives (2024)**: While svd2rust remains popular, newer tools like `chiptool` and `metapac` offer alternative approaches. Metapac provides additional metadata (memory layout, interrupt tables) alongside register access, useful for HAL frameworks like Embassy.

## Part 4: The Embedded HAL Traits

### Core Traits

The embedded-hal provides standard traits for common peripherals:

```rust
use embedded_hal::digital::v2::{OutputPin, InputPin};
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi::{Write, Transfer};
use embedded_hal::blocking::i2c::{Read, Write as I2cWrite};

// GPIO traits
pub trait OutputPin {
    type Error;
    fn set_low(&mut self) -> Result<(), Self::Error>;
    fn set_high(&mut self) -> Result<(), Self::Error>;
}

pub trait InputPin {
    type Error;
    fn is_high(&self) -> Result<bool, Self::Error>;
    fn is_low(&self) -> Result<bool, Self::Error>;
}
```

### Implementing HAL Traits

```rust
use embedded_hal::digital::v2::OutputPin;
use core::convert::Infallible;

pub struct GpioPin {
    pin_number: u8,
    gpio_out: &'static Register<u32>,
}

impl OutputPin for GpioPin {
    type Error = Infallible;

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.gpio_out.modify(|val| val | (1 << self.pin_number));
        Ok(())
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.gpio_out.modify(|val| val & !(1 << self.pin_number));
        Ok(())
    }
}
```

### Driver Portability

Write drivers that work with any HAL implementation:

```rust
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::OutputPin;

pub struct Led<P: OutputPin> {
    pin: P,
}

impl<P: OutputPin> Led<P> {
    pub fn new(pin: P) -> Self {
        Led { pin }
    }

    pub fn on(&mut self) -> Result<(), P::Error> {
        self.pin.set_high()
    }

    pub fn off(&mut self) -> Result<(), P::Error> {
        self.pin.set_low()
    }

    pub fn blink<D: DelayMs<u32>>(
        &mut self,
        delay: &mut D,
        ms: u32,
    ) -> Result<(), P::Error> {
        self.on()?;
        delay.delay_ms(ms);
        self.off()?;
        delay.delay_ms(ms);
        Ok(())
    }
}
```

## Part 5: Real-World Example - SPI Display Driver

### Portable Display Driver

```rust
use embedded_hal::blocking::spi::Write;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::blocking::delay::DelayMs;

pub struct ST7789<SPI, DC, RST, DELAY> {
    spi: SPI,
    dc: DC,
    rst: RST,
    delay: DELAY,
}

impl<SPI, DC, RST, DELAY> ST7789<SPI, DC, RST, DELAY>
where
    SPI: Write<u8>,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayMs<u32>,
{
    pub fn new(spi: SPI, dc: DC, rst: RST, delay: DELAY) -> Self {
        ST7789 { spi, dc, rst, delay }
    }

    pub fn init(&mut self) -> Result<(), Error> {
        // Reset sequence
        self.rst.set_low().map_err(|_| Error::Gpio)?;
        self.delay.delay_ms(10);
        self.rst.set_high().map_err(|_| Error::Gpio)?;
        self.delay.delay_ms(120);

        // Initialization commands
        self.command(0x01)?;  // Software reset
        self.delay.delay_ms(150);

        self.command(0x11)?;  // Sleep out
        self.delay.delay_ms(10);

        self.command(0x3A)?;  // Pixel format
        self.data(&[0x55])?;  // 16-bit color

        self.command(0x29)?;  // Display on

        Ok(())
    }

    fn command(&mut self, cmd: u8) -> Result<(), Error> {
        self.dc.set_low().map_err(|_| Error::Gpio)?;
        self.spi.write(&[cmd]).map_err(|_| Error::Spi)?;
        Ok(())
    }

    fn data(&mut self, data: &[u8]) -> Result<(), Error> {
        self.dc.set_high().map_err(|_| Error::Gpio)?;
        self.spi.write(data).map_err(|_| Error::Spi)?;
        Ok(())
    }

    pub fn draw_pixel(&mut self, x: u16, y: u16, color: u16) -> Result<(), Error> {
        self.set_window(x, y, x, y)?;
        self.command(0x2C)?;  // Memory write
        self.data(&color.to_be_bytes())?;
        Ok(())
    }

    fn set_window(&mut self, x0: u16, y0: u16, x1: u16, y1: u16) -> Result<(), Error> {
        self.command(0x2A)?;  // Column address set
        self.data(&x0.to_be_bytes())?;
        self.data(&x1.to_be_bytes())?;

        self.command(0x2B)?;  // Row address set
        self.data(&y0.to_be_bytes())?;
        self.data(&y1.to_be_bytes())?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    Spi,
    Gpio,
}
```

## Part 6: Interrupt Handling

### Critical Sections and Atomics

```rust
use cortex_m::interrupt;
use core::cell::RefCell;
use cortex_m::interrupt::Mutex;

// Shared state between interrupt and main
static COUNTER: Mutex<RefCell<u32>> = Mutex::new(RefCell::new(0));

#[interrupt]
fn TIMER0() {
    interrupt::free(|cs| {
        let mut counter = COUNTER.borrow(cs).borrow_mut();
        *counter += 1;
    });
}

fn main() {
    // Access shared state safely
    let count = interrupt::free(|cs| {
        *COUNTER.borrow(cs).borrow()
    });
}
```

### DMA with Volatile Buffers

```rust
use core::sync::atomic::{AtomicBool, Ordering};

#[repr(C, align(4))]
struct DmaBuffer {
    data: [u8; 1024],
}

static mut DMA_BUFFER: DmaBuffer = DmaBuffer { data: [0; 1024] };
static DMA_COMPLETE: AtomicBool = AtomicBool::new(false);

fn start_dma_transfer() {
    unsafe {
        // Configure DMA to write to DMA_BUFFER
        let buffer_addr = &DMA_BUFFER as *const _ as u32;

        // Set up DMA registers (hardware-specific)
        const DMA_SRC: *mut u32 = 0x4002_0000 as *mut u32;
        const DMA_DST: *mut u32 = 0x4002_0004 as *mut u32;
        const DMA_LEN: *mut u32 = 0x4002_0008 as *mut u32;
        const DMA_CTRL: *mut u32 = 0x4002_000C as *mut u32;

        core::ptr::write_volatile(DMA_SRC, 0x2000_0000);  // Source address
        core::ptr::write_volatile(DMA_DST, buffer_addr);   // Destination
        core::ptr::write_volatile(DMA_LEN, 1024);          // Transfer length
        core::ptr::write_volatile(DMA_CTRL, 0x01);         // Start transfer
    }
}

#[interrupt]
fn DMA_DONE() {
    DMA_COMPLETE.store(true, Ordering::Release);
}

fn wait_for_dma() {
    while !DMA_COMPLETE.load(Ordering::Acquire) {
        cortex_m::asm::wfi();  // Wait for interrupt
    }

    // DMA complete - buffer contents are valid
    unsafe {
        // Must use volatile reads since DMA wrote the data
        let first_byte = core::ptr::read_volatile(&DMA_BUFFER.data[0]);
    }
}
```

## Part 7: Power Management

### Low-Power Modes

```rust
pub enum PowerMode {
    Active,
    Sleep,
    DeepSleep,
    Hibernate,
}

pub struct PowerController {
    pwr_ctrl: &'static Register<u32>,
}

impl PowerController {
    pub fn set_mode(&self, mode: PowerMode) {
        let ctrl_value = match mode {
            PowerMode::Active => 0x00,
            PowerMode::Sleep => 0x01,
            PowerMode::DeepSleep => 0x02,
            PowerMode::Hibernate => 0x03,
        };

        self.pwr_ctrl.write(ctrl_value);

        // Execute wait-for-interrupt to enter low-power mode
        cortex_m::asm::wfi();
    }

    pub fn configure_wakeup_sources(&self, sources: u32) {
        const WAKEUP_EN: Register<u32> = Register::new(0x4000_1000);
        WAKEUP_EN.write(sources);
    }
}
```

## Part 8: Real Hardware Example - ESP32-C3

### Complete Blinky Example

```rust
#![no_std]
#![no_main]

use esp32c3_hal::{clock::ClockControl, pac::Peripherals, prelude::*, timer::TimerGroup, Rtc};
use esp_backtrace as _;
use riscv_rt::entry;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    let mut wdt1 = timer_group1.wdt;

    // Disable watchdogs
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    // Configure GPIO
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut led = io.pins.gpio7.into_push_pull_output();

    // Main loop
    loop {
        led.toggle().unwrap();
        delay(500_000);
    }
}

fn delay(cycles: u32) {
    for _ in 0..cycles {
        unsafe { riscv::asm::nop() };
    }
}
```

## Best Practices

1. **Always Use Volatile**: Hardware registers require volatile access
2. **Type Safety**: Use strong types to prevent register misuse
3. **Singleton Pattern**: Ensure single ownership of peripherals
4. **Critical Sections**: Protect shared state in interrupts
5. **Zero-Cost Abstractions**: HAL traits compile to direct register access
6. **Test on Hardware**: Emulators may not match real hardware behavior

## Common Pitfalls

1. **Forgetting Volatile**: Regular access leads to optimization bugs
2. **Race Conditions**: Unprotected access from interrupts
3. **Alignment Issues**: DMA buffers need proper alignment
4. **Clock Configuration**: Wrong clock setup causes timing issues
5. **Power States**: Peripherals may need re-initialization after sleep

## Summary

Embedded HAL in Rust provides:

- **Volatile access patterns** for hardware registers
- **Type-safe abstractions** over raw memory access
- **PAC generation** from SVD files
- **Portable drivers** via HAL traits
- **Memory safety** in embedded contexts

The embedded-hal ecosystem enables writing portable, reusable drivers while maintaining the performance of direct hardware access.

## Additional Resources

- [The Embedded Rust Book](https://docs.rust-embedded.org/book/)
- [embedded-hal Documentation](https://docs.rs/embedded-hal/)
- [svd2rust](https://github.com/rust-embedded/svd2rust)
- [awesome-embedded-rust](https://github.com/rust-embedded/awesome-embedded-rust)
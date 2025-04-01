# xhypervisor

[![crates.io](https://img.shields.io/crates/v/xhypervisor.svg)](https://crates.io/crates/xhypervisor)
[![Docs.rs](https://docs.rs/xhypervisor/badge.svg)](https://docs.rs/xhypervisor)
[![License](https://img.shields.io/crates/l/xhypervisor.svg)](https://img.shields.io/crates/l/xhypervisor.svg)

`xhypervisor` is a Rust library that taps into functionality that enables hardware-accelerated execution of virtual machines on OS X.
It is a fork of [hypervisor-rs](https://github.com/saurvs/hypervisor-rs) and modified for the development of [uhyve](https://github.com/hermit-os/uhyve).
Derived from [ahv](https://github.com/Thog/ahv), we added the support of Apple's Hypervisor Framework on Apple Silicon.

It binds to the [Hypervisor](https://developer.apple.com/documentation/hypervisor) framework on OS X, and exposes a safe Rust interface through the `hypervisor` module, and an unsafe foreign function interface through the `xhypervisor::ffi` module.

## Prerequisites

To use this library, you need

* macOS 10, or newer
* An Intel processor with the VT-x feature or an Apple Silicon processor with virtualization support. To verify this, run and expect the
following in your Terminal:
  ```shell
  $ sysctl kern.hv_support
  kern.hv_support: 1
  ```

## Status
- [x] Accessing x86 registers
- [x] Accessing aarch64 registers
- [x] Mapping guest physical memory segments into guest physical address space
- [x] x86: Accessing fields of Virtual Machine Control Structures (VMCS)
- [x] Virtual x86 CPUs
  - [x] Executing and interrupting
  - [x] Force flushing cached state
  - [x] Invalidating translation lookaside buffer (TLB)
  - [x] Accessing floating point (FP) and SIMD state
  - [x] Obtaining cumulative execution time
  - [x] Synchronizing guest timestamp-counters (TSC)
  - [x] Accessing model-specific registers (MSRs)
- [x] Virtual aarch64 CPUs
  - [x] Executing and interrupting
  - [x] GICv3 support (requires macOS 15, or newer)

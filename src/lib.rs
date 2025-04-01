/*
Copyright (c) 2016 Saurav Sachidanand
			  2021 Stefan Lankes

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.
*/

/*!
This is a Rust library that taps into functionality that enables
hardware-accelerated execution of virtual machines on OS X.

It binds to the `Hypervisor` framework on OS X, and exposes a safe Rust
interface through the `xhypervisor` module, and an unsafe foreign function
interface through the `xhypervisor::ffi` module.

To use this library, you need

* OS X Yosemite (10.10), or newer

* an Intel processor with the VT-x feature set that includes Extended Page
Tables (EPT) and Unrestricted Mode. To verify this, run and expect the following
in your Terminal:

  ```shell
  $ sysctl kern.hv_support
  kern.hv_support: 1
  ```
!*/

extern crate core;
extern crate libc;
extern crate thiserror;

#[cfg(target_arch = "aarch64")]
#[allow(non_camel_case_types)]
pub mod aarch64;
#[cfg(target_arch = "x86_64")]
#[allow(non_camel_case_types)]
pub mod x86_64;

use thiserror::Error;

#[cfg(target_arch = "x86_64")]
use self::x86_64::ffi::*;
#[cfg(target_arch = "aarch64")]
use aarch64::ffi::*;
#[cfg(target_arch = "aarch64")]
pub use aarch64::*;
#[cfg(target_arch = "x86_64")]
pub use x86_64::*;

/// Error returned after every call
#[derive(Error, Debug)]
pub enum Error {
	#[error("success")]
	Success,
	#[error("error")]
	Error,
	#[error("busy")]
	Busy,
	#[error("bad argument")]
	BadArg,
	#[error("no resource")]
	NoRes,
	#[error("no device")]
	NoDev,
	#[error("unsupported")]
	Unsupp,
}

// Returns an Error for a hv_return_t
fn match_error_code(code: hv_return_t) -> Result<(), Error> {
	match code {
		HV_SUCCESS => Ok(()),
		HV_BUSY => Err(Error::Busy),
		HV_BAD_ARGUMENT => Err(Error::BadArg),
		HV_NO_RESOURCES => Err(Error::NoRes),
		HV_NO_DEVICE => Err(Error::NoDev),
		HV_UNSUPPORTED => Err(Error::Unsupp),
		_ => Err(Error::Error),
	}
}

/// Destroys the VM instance associated with the current Mach task
pub fn destroy_vm() -> Result<(), Error> {
	match_error_code(unsafe { hv_vm_destroy() })
}

/// Guest physical memory region permissions
#[derive(Debug)]
pub enum MemPerm {
	/// No access
	None,
	/// Read
	Read,
	/// Write
	Write,
	/// Read and write access
	ReadWrite,
	/// Execute
	Exec,
	/// Execute and write
	ExecWrite,
	/// Execute and read
	ExecRead,
	/// Execute, read and write
	ExecReadWrite,
}

#[allow(non_snake_case)]
#[inline(always)]
fn match_MemPerm(mem_perm: MemPerm) -> u64 {
	match mem_perm {
		MemPerm::None => 0,
		MemPerm::Read => HV_MEMORY_READ,
		MemPerm::Write => HV_MEMORY_WRITE,
		MemPerm::ReadWrite => HV_MEMORY_WRITE | HV_MEMORY_READ,
		MemPerm::Exec => HV_MEMORY_EXEC,
		MemPerm::ExecWrite => HV_MEMORY_EXEC | HV_MEMORY_WRITE,
		MemPerm::ExecRead => HV_MEMORY_EXEC | HV_MEMORY_READ,
		MemPerm::ExecReadWrite => HV_MEMORY_EXEC | HV_MEMORY_READ | HV_MEMORY_WRITE,
	}
}

impl VirtualCpu {
	/// Destroys the VirtualCpu instance associated with the current thread
	pub fn destroy(&self) -> Result<(), Error> {
		match_error_code(unsafe { hv_vcpu_destroy(self.get_handle()) })
	}

	/// Executes the VirtualCpu
	pub fn run(&self) -> Result<(), Error> {
		match_error_code(unsafe { hv_vcpu_run(self.get_handle()) })
	}
}

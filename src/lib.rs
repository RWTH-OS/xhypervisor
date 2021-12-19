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

pub mod consts;
#[cfg(target_arch = "aarch64")]
#[allow(non_camel_case_types)]
pub mod aarch64;
#[cfg(target_arch = "x86_64")]
#[allow(non_camel_case_types)]
pub mod x86_64;

use std::ptr::null_mut;

use self::core::fmt;
use libc::*;

#[cfg(target_arch = "aarch64")]
use self::aarch64::*;
#[cfg(target_arch = "x86_64")]
use self::x86_64::*;

/// Error returned after every call
#[derive(Clone)]
pub enum Error {
	/// Success
	Success,
	/// Error
	Error,
	/// Busy
	Busy,
	/// Bad argument
	BadArg,
	/// No resources
	NoRes,
	/// No device
	NoDev,
	/// Unsupported
	Unsupp,
}

impl fmt::Debug for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Error::Success => write!(f, "Success"),
			Error::Error => write!(f, "Error"),
			Error::Busy => write!(f, "Busy"),
			Error::BadArg => write!(f, "Bad argument"),
			Error::NoRes => write!(f, "No resources"),
			Error::NoDev => write!(f, "No device"),
			Error::Unsupp => write!(f, "Unsupported"),
		}
	}
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

/// Creates a VM instance for the current Mach task
#[cfg(target_arch = "aarch64")]
pub fn create_vm() -> Result<(), Error> {
	match_error_code(unsafe { hv_vm_create(null_mut()) })
}

/// Creates a VM instance for the current Mach task
#[cfg(target_arch = "x86_64")]
pub fn create_vm() -> Result<(), Error> {
	match_error_code(unsafe { hv_vm_create(HV_VM_DEFAULT) })
}

/// Destroys the VM instance associated with the current Mach task
pub fn destroy_vm() -> Result<(), Error> {
	match_error_code(unsafe { hv_vm_destroy() })
}

/// Guest physical memory region permissions
pub enum MemPerm {
	/// Read
	Read,
	/// Write (implies read)
	Write,
	/// Execute
	Exec,
	/// Execute and write (implies read)
	ExecAndWrite,
	/// Execute and read
	ExecAndRead,
}

#[allow(non_snake_case)]
#[inline(always)]
fn match_MemPerm(mem_perm: &MemPerm) -> u64 {
	match mem_perm {
		&MemPerm::Read => HV_MEMORY_READ,
		&MemPerm::Write => HV_MEMORY_WRITE | HV_MEMORY_READ,
		&MemPerm::Exec => HV_MEMORY_EXEC,
		&MemPerm::ExecAndWrite => HV_MEMORY_EXEC | HV_MEMORY_WRITE | HV_MEMORY_READ,
		&MemPerm::ExecAndRead => HV_MEMORY_EXEC | HV_MEMORY_READ,
	}
}

/// Maps a region in the virtual address space of the current task into the guest physical
/// address space of the virutal machine
#[cfg(target_arch = "x86_64")]
pub fn map_mem(mem: &[u8], gpa: u64, mem_perm: &MemPerm) -> Result<(), Error> {
	match_error_code(unsafe {
		hv_vm_map(
			mem.as_ptr() as *const c_void,
			gpa as hv_gpaddr_t,
			mem.len() as size_t,
			match_MemPerm(mem_perm),
		)
	})
}

/// Maps a region in the virtual address space of the current task into the guest physical
/// address space of the virutal machine
#[cfg(target_arch = "aarch64")]
pub fn map_mem(mem: &[u8], ipa: u64, mem_perm: &MemPerm) -> Result<(), Error> {
	match_error_code(unsafe {
		hv_vm_map(
			mem.as_ptr() as *mut c_void,
			ipa as hv_ipa_t,
			mem.len() as size_t,
			match_MemPerm(mem_perm),
		)
	})
}

/// Unmaps a region in the guest physical address space of the virutal machine
#[cfg(target_arch = "x86_64")]
pub fn unmap_mem(gpa: u64, size: usize) -> Result<(), Error> {
	match_error_code(unsafe { hv_vm_unmap(gpa as hv_gpaddr_t, size as size_t) })
}

/// Unmaps a region in the guest physical address space of the virutal machine
#[cfg(target_arch = "aarch64")]
pub fn unmap_mem(ipa: u64, size: usize) -> Result<(), Error> {
	match_error_code(unsafe { hv_vm_unmap(ipa as hv_ipa_t, size as size_t) })
}

/// Modifies the permissions of a region in the guest physical address space of the virtual
/// machine
#[cfg(target_arch = "x86_64")]
pub fn protect_mem(gpa: u64, size: usize, mem_perm: &MemPerm) -> Result<(), Error> {
	match_error_code(unsafe {
		hv_vm_protect(gpa as hv_gpaddr_t, size as size_t, match_MemPerm(mem_perm))
	})
}

/// Modifies the permissions of a region in the guest physical address space of the virtual
/// machine
#[cfg(target_arch = "aarch64")]
pub fn protect_mem(ipa: u64, size: usize, mem_perm: &MemPerm) -> Result<(), Error> {
	match_error_code(unsafe {
		hv_vm_protect(ipa as hv_ipa_t, size as size_t, match_MemPerm(mem_perm))
	})
}

/// Synchronizes the guest Timestamp-Counters (TSC) across all VirtualCpus
///
/// * `tsc` Guest TSC value
#[cfg(target_arch = "x86_64")]
pub fn sync_tsc(tsc: u64) -> Result<(), Error> {
	match_error_code(unsafe { hv_vm_sync_tsc(tsc) })
}

/// Forces an immediate VMEXIT of a set of VirtualCpus
///
/// * `VirtualCpu_ids` Array of VirtualCpu IDs
#[cfg(target_arch = "x86_64")]
pub fn interrupt_vcpus(vcpu_ids: &[u32]) -> Result<(), Error> {
	match_error_code(unsafe { hv_vcpu_interrupt(vcpu_ids.as_ptr(), vcpu_ids.len() as c_uint) })
}

#[cfg(target_arch = "aarch64")]
#[derive(Copy, Clone, Debug)]
/// Exit reason of a virtual CPU
/// Enum is derived from
/// https://github.com/Thog/ahv
pub enum VirtualCpuExitReason {
	/// Asynchronous exit.
	Cancelled,

	/// Guest exception.
	Exception {
		/// The informations about the guest exception.
		exception: hv_vcpu_exit_exception_t,
	},

	/// Virtual Timer enters the pending state.
	VTimerActivated,

	/// Unexpected exit.
	Unknown,
}

#[cfg(target_arch = "aarch64")]
impl From<hv_vcpu_exit_t> for VirtualCpuExitReason {
	fn from(value: hv_vcpu_exit_t) -> VirtualCpuExitReason {
		match value.reason {
			HV_EXIT_REASON_CANCELED => VirtualCpuExitReason::Cancelled,
			HV_EXIT_REASON_EXCEPTION => VirtualCpuExitReason::Exception {
				exception: value.exception,
			},
			HV_EXIT_REASON_VTIMER_ACTIVATED => VirtualCpuExitReason::VTimerActivated,
			HV_EXIT_REASON_UNKNOWN => VirtualCpuExitReason::Unknown,

			// Unexpected unknown
			_ => VirtualCpuExitReason::Unknown,
		}
	}
}

/// Virtual CPU
pub struct VirtualCpu {
	#[cfg(target_arch = "x86_64")]
	/// Virtual CPU handle
	id: hv_vcpuid_t,

	#[cfg(target_arch = "aarch64")]
	/// Virtual CPU handle
	id: hv_vcpu_t,

	#[cfg(target_arch = "aarch64")]
	/// VirtualCPU exit informations.
	vcpu_exit: *const hv_vcpu_exit_t,
}

/// aarch64 architectural register
#[cfg(target_arch = "aarch64")]
pub enum Register {
	/// X0 register.
	X0,

	/// X1 register.
	X1,

	/// X2 register.
	X2,

	/// X3 register.
	X3,

	/// X4 register.
	X4,

	/// X5 register.
	X5,

	/// X6 register.
	X6,

	/// X7 register.
	X7,

	/// X8 register.
	X8,

	/// X9 register.
	X9,

	/// X10 register.
	X10,

	/// X11 register.
	X11,

	/// X12 register.
	X12,

	/// X13 register.
	X13,

	/// X14 register.
	X14,

	/// X15 register.
	X15,

	/// X16 register.
	X16,

	/// X17 register.
	X17,

	/// X18 register.
	X18,

	/// X19 register.
	X19,

	/// X20 register.
	X20,

	/// X21 register.
	X21,

	/// X22 register.
	X22,

	/// X23 register.
	X23,

	/// X24 register.
	X24,

	/// X25 register.
	X25,

	/// X26 register.
	X26,

	/// X27 register.
	X27,

	/// X28 register.
	X28,

	/// X29 register.
	X29,

	/// FP register.
	FP,

	/// X30 register.
	X30,

	/// LR register.
	LR,

	/// PC register.
	PC,

	/// FPCR register.
	FPCR,

	/// FPSR register.
	FPSR,

	/// CPSR register.
	CPSR,
}

#[cfg(target_arch = "aarch64")]
impl From<Register> for hv_reg_t {
	fn from(value: Register) -> hv_reg_t {
		match value {
			Register::X0 => HV_REG_X0,
			Register::X1 => HV_REG_X1,
			Register::X2 => HV_REG_X2,
			Register::X3 => HV_REG_X3,
			Register::X4 => HV_REG_X4,
			Register::X5 => HV_REG_X5,
			Register::X6 => HV_REG_X6,
			Register::X7 => HV_REG_X7,
			Register::X8 => HV_REG_X8,
			Register::X9 => HV_REG_X9,
			Register::X10 => HV_REG_X10,
			Register::X11 => HV_REG_X11,
			Register::X12 => HV_REG_X12,
			Register::X13 => HV_REG_X13,
			Register::X14 => HV_REG_X14,
			Register::X15 => HV_REG_X15,
			Register::X16 => HV_REG_X16,
			Register::X17 => HV_REG_X17,
			Register::X18 => HV_REG_X18,
			Register::X19 => HV_REG_X19,
			Register::X20 => HV_REG_X20,
			Register::X21 => HV_REG_X21,
			Register::X22 => HV_REG_X22,
			Register::X23 => HV_REG_X23,
			Register::X24 => HV_REG_X24,
			Register::X25 => HV_REG_X25,
			Register::X26 => HV_REG_X26,
			Register::X27 => HV_REG_X27,
			Register::X28 => HV_REG_X28,
			Register::X29 => HV_REG_X29,
			Register::X30 => HV_REG_X30,
			Register::FP => HV_REG_FP,
			Register::LR => HV_REG_LR,
			Register::PC => HV_REG_PC,
			Register::FPCR => HV_REG_FPCR,
			Register::FPSR => HV_REG_FPSR,
			Register::CPSR => HV_REG_CPSR,
		}
	}
}

/// x86 architectural register
#[cfg(target_arch = "x86_64")]
#[derive(Clone)]
#[repr(C)]
pub enum Register {
	RIP,
	RFLAGS,
	RAX,
	RCX,
	RDX,
	RBX,
	RSI,
	RDI,
	RSP,
	RBP,
	R8,
	R9,
	R10,
	R11,
	R12,
	R13,
	R14,
	R15,
	CS,
	SS,
	DS,
	ES,
	FS,
	GS,
	IDT_BASE,
	IDT_LIMIT,
	GDT_BASE,
	GDT_LIMIT,
	LDTR,
	LDT_BASE,
	LDT_LIMIT,
	LDT_AR,
	TR,
	TSS_BASE,
	TSS_LIMIT,
	TSS_AR,
	CR0,
	CR1,
	CR2,
	CR3,
	CR4,
	DR0,
	DR1,
	DR2,
	DR3,
	DR4,
	DR5,
	DR6,
	DR7,
	TPR,
	XCR0,
	REGISTERS_MAX,
}

impl VirtualCpu {
	/// Creates a VirtualCpu instance for the current thread
	#[cfg(target_arch = "x86_64")]
	pub fn new() -> Result<VirtualCpu, Error> {
		let mut vcpuid: hv_vcpuid_t = 0;

		match_error_code(unsafe { hv_vcpu_create(&mut vcpuid, HV_VCPU_DEFAULT) })?;

		Ok(VirtualCpu { id: vcpuid })
	}

	#[cfg(target_arch = "aarch64")]
	pub fn new() -> Result<VirtualCpu, Error> {
		let handle: hv_vcpu_config_t = core::ptr::null_mut();
		let mut vcpu_handle: hv_vcpu_t = 0;
		let mut vcpu_exit: *const hv_vcpu_exit_t = core::ptr::null_mut();

		match_error_code(unsafe { hv_vcpu_create(&mut vcpu_handle, &mut vcpu_exit, &handle) })?;

		Ok(VirtualCpu {
			id: vcpu_handle,
			vcpu_exit: vcpu_exit,
		})
	}

	/// Destroys the VirtualCpu instance associated with the current thread
	pub fn destroy(&self) -> Result<(), Error> {
		match_error_code(unsafe { hv_vcpu_destroy(self.id) })
	}

	/// Executes the VirtualCpu
	pub fn run(&self) -> Result<(), Error> {
		match_error_code(unsafe { hv_vcpu_run(self.id) })
	}

	#[cfg(target_arch = "aarch64")]
	pub fn exit_reason(&self) -> VirtualCpuExitReason {
		VirtualCpuExitReason::from(unsafe { *self.vcpu_exit })
	}

	/// Forces an immediate VMEXIT of the VirtualCpu
	#[cfg(target_arch = "x86_64")]
	pub fn interrupt(&self) -> Result<(), Error> {
		match_error_code(unsafe { hv_vcpu_interrupt(&(self.id), 1 as c_uint) })
	}

	/// Returns the cumulative execution time of the VirtualCpu in nanoseconds
	#[cfg(target_arch = "x86_64")]
	pub fn exec_time(&self) -> Result<u64, Error> {
		let mut exec_time: u64 = 0;

		let _error = match_error_code(unsafe { hv_vcpu_get_exec_time(self.id, &mut exec_time) })?;

		Ok(exec_time)
	}

	/// Forces flushing of cached VirtualCpu state
	#[cfg(target_arch = "x86_64")]
	pub fn flush(&self) -> Result<(), Error> {
		match_error_code(unsafe { hv_vcpu_flush(self.id) })
	}

	/// Invalidates the translation lookaside buffer (TLB) of the VirtualCpu
	#[cfg(target_arch = "x86_64")]
	pub fn invalidate_tlb(&self) -> Result<(), Error> {
		match_error_code(unsafe { hv_vcpu_invalidate_tlb(self.id) })
	}

	/// Enables an MSR to be used natively by the VM
	#[cfg(target_arch = "x86_64")]
	pub fn enable_native_msr(&self, msr: u32, enable: bool) -> Result<(), Error> {
		match_error_code(unsafe { hv_vcpu_enable_native_msr(self.id, msr, enable) })
	}

	/// Returns the current value of an MSR of the VirtualCpu
	#[cfg(target_arch = "x86_64")]
	pub fn read_msr(&self, msr: u32) -> Result<u64, Error> {
		let mut value: u64 = 0;

		let _error = match_error_code(unsafe { hv_vcpu_read_msr(self.id, msr, &mut value) })?;

		Ok(value)
	}

	/// Set the value of an MSR of the VirtualCpu
	#[cfg(target_arch = "x86_64")]
	pub fn write_msr(&self, msr: u32, value: u64) -> Result<(), Error> {
		match_error_code(unsafe { hv_vcpu_write_msr(self.id, msr, &(value)) })
	}

	/// Returns the current value of an architectural x86 register
	/// of the VirtualCpu
	#[cfg(target_arch = "x86_64")]
	pub fn read_register(&self, reg: &Register) -> Result<u64, Error> {
		let mut value: u64 = 0;

		match_error_code(unsafe {
			hv_vcpu_read_register(self.id, (*reg).clone(), &mut value)
		})?;

		Ok(value)
	}

	/// Returns the current value of an architectural aarch64 register
	/// of the VirtualCpu
	#[cfg(target_arch = "aarch64")]
	pub fn read_register(&self, reg: Register) -> Result<u64, Error> {
		let mut value: u64 = 0;

		match_error_code(unsafe {
			hv_vcpu_get_reg(self.id, hv_reg_t::from(reg), &mut value as *mut u64)
		})?;

		Ok(value)
	}

	/// Sets the value of an architectural x86 register of the VirtualCpu
	#[cfg(target_arch = "x86_64")]
	pub fn write_register(&self, reg: &Register, value: u64) -> Result<(), Error> {
		match_error_code(unsafe { hv_vcpu_write_register(self.id, (*reg).clone(), value) })
	}

	/// Sets the value of an architectural x86 register of the VirtualCpu
	#[cfg(target_arch = "aarch64")]
	pub fn write_register(&self, reg: Register, value: u64) -> Result<(), Error> {
		match_error_code(unsafe { hv_vcpu_set_reg(self.id, hv_reg_t::from(reg), value) })
	}

	/// Returns the current value of a VMCS field of the VirtualCpu
	#[cfg(target_arch = "x86_64")]
	pub fn read_vmcs(&self, field: u32) -> Result<u64, Error> {
		let mut value: u64 = 0;

		match_error_code(unsafe { hv_vmx_vcpu_read_vmcs(self.id, field, &mut value) })?;

		Ok(value)
	}

	/// Sets the value of a VMCS field of the VirtualCpu
	#[cfg(target_arch = "x86_64")]
	pub fn write_vmcs(&self, field: u32, value: u64) -> Result<(), Error> {
		match_error_code(unsafe { hv_vmx_vcpu_write_vmcs(self.id, field, value) })
	}

	/// Sets the address of the guest APIC for the VirtualCpu in the
	/// guest physical address space of the VM
	#[cfg(target_arch = "x86_64")]
	pub fn set_apic_addr(&self, gpa: u64) -> Result<(), Error> {
		match_error_code(unsafe { hv_vmx_vcpu_set_apic_address(self.id, gpa) })
	}

	/// Reads the current architectural x86 floating point and SIMD state of the VirtualCpu
	#[cfg(target_arch = "x86_64")]
	pub fn read_fpstate(&self, buffer: &mut [u8]) -> Result<(), Error> {
		match_error_code(unsafe {
			hv_vcpu_read_fpstate(
				self.id,
				buffer.as_mut_ptr() as *mut c_void,
				buffer.len() as size_t,
			)
		})
	}

	/// Sets the architectural x86 floating point and SIMD state of the VirtualCpu
	#[cfg(target_arch = "x86_64")]
	pub fn write_fpstate(&self, buffer: &[u8]) -> Result<(), Error> {
		match_error_code(unsafe {
			hv_vcpu_write_fpstate(
				self.id,
				buffer.as_ptr() as *const c_void,
				buffer.len() as size_t,
			)
		})
	}
}

impl fmt::Debug for VirtualCpu {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "VirtualCpu ID: {}", (*self).id)
	}
}

/// VMX cabability
#[cfg(target_arch = "x86_64")]
#[allow(non_camel_case_types)]
#[derive(Clone, Debug)]
#[repr(C)]
pub enum VMXCap {
	/// Pin-based VMX capabilities
	PINBASED = 0,
	/// Primary proc-based VMX capabilities
	PROCBASED = 1,
	/// Secondary proc-based VMX capabilities
	PROCBASED2 = 2,
	/// VM-entry VMX capabilities
	ENTRY = 3,
	/// VM-exit VMX capabilities
	EXIT = 4,
	/// VMX preemption timer frequency
	PREEMPTION_TIMER = 32,
}

/// Reads a VMX capability of the host processor
#[cfg(target_arch = "x86_64")]
pub fn read_vmx_cap(vmx_cap: &VMXCap) -> Result<u64, Error> {
	let mut value: u64 = 0;

	match_error_code(unsafe { hv_vmx_read_capability((*vmx_cap).clone(), &mut value) })?;

	Ok(value)
}

#[cfg(target_arch = "x86_64")]
impl fmt::Display for VMXCap {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			VMXCap::PINBASED => write!(f, "Pin-based VMX capabilities"),
			VMXCap::PROCBASED => write!(f, "Primary proc-based VMX capabilities"),
			VMXCap::PROCBASED2 => write!(f, "Secondary proc-based VMX capabilities"),
			VMXCap::ENTRY => write!(f, "VM-entry VMX capabilities"),
			VMXCap::EXIT => write!(f, "VM-exit VMX capabilities"),
			VMXCap::PREEMPTION_TIMER => write!(f, "VMX preemption timer frequency"),
		}
	}
}

pub mod consts;
pub mod ffi;

use self::ffi::*;
use crate::{match_MemPerm, match_error_code, Error, MemPerm};
use core::fmt;
use libc::*;

/// Creates a VM instance for the current Mach task
pub fn create_vm() -> Result<(), Error> {
	match_error_code(unsafe { hv_vm_create(HV_VM_DEFAULT) })
}

/// Maps a region in the virtual address space of the current task into the guest physical
/// address space of the virutal machine
pub fn map_mem(mem: &[u8], gpa: u64, mem_perm: MemPerm) -> Result<(), Error> {
	match_error_code(unsafe {
		hv_vm_map(
			mem.as_ptr() as *const c_void,
			gpa as hv_gpaddr_t,
			mem.len() as size_t,
			match_MemPerm(mem_perm),
		)
	})
}

/// Modifies the permissions of a region in the guest physical address space of the virtual
/// machine
pub fn protect_mem(gpa: u64, size: usize, mem_perm: MemPerm) -> Result<(), Error> {
	match_error_code(unsafe {
		hv_vm_protect(gpa as hv_gpaddr_t, size as size_t, match_MemPerm(mem_perm))
	})
}

/// Unmaps a region in the guest physical address space of the virutal machine
pub fn unmap_mem(gpa: u64, size: usize) -> Result<(), Error> {
	match_error_code(unsafe { hv_vm_unmap(gpa as hv_gpaddr_t, size as size_t) })
}

/// Synchronizes the guest Timestamp-Counters (TSC) across all VirtualCpus
///
/// * `tsc` Guest TSC value
pub fn sync_tsc(tsc: u64) -> Result<(), Error> {
	match_error_code(unsafe { hv_vm_sync_tsc(tsc) })
}

/// Forces an immediate VMEXIT of a set of VirtualCpus
///
/// * `VirtualCpu_ids` Array of VirtualCpu IDs
pub fn interrupt_vcpus(vcpu_ids: &[u32]) -> Result<(), Error> {
	match_error_code(unsafe { hv_vcpu_interrupt(vcpu_ids.as_ptr(), vcpu_ids.len() as c_uint) })
}

/// Virtual CPU
pub struct VirtualCpu {
	/// Virtual CPU handle
	id: hv_vcpuid_t,
}

/// x86 architectural register
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
	pub fn new() -> Result<VirtualCpu, Error> {
		let mut vcpuid: hv_vcpuid_t = 0;

		match_error_code(unsafe { hv_vcpu_create(&mut vcpuid, HV_VCPU_DEFAULT) })?;

		Ok(VirtualCpu { id: vcpuid })
	}

	pub fn get_id(&self) -> hv_vcpuid_t {
		self.id
	}

	/// Forces an immediate VMEXIT of the VirtualCpu
	pub fn interrupt(&self) -> Result<(), Error> {
		match_error_code(unsafe { hv_vcpu_interrupt(&(self.id), 1 as c_uint) })
	}

	/// Returns the cumulative execution time of the VirtualCpu in nanoseconds
	pub fn exec_time(&self) -> Result<u64, Error> {
		let mut exec_time: u64 = 0;

		let _error = match_error_code(unsafe { hv_vcpu_get_exec_time(self.id, &mut exec_time) })?;

		Ok(exec_time)
	}

	/// Forces flushing of cached VirtualCpu state
	pub fn flush(&self) -> Result<(), Error> {
		match_error_code(unsafe { hv_vcpu_flush(self.id) })
	}

	/// Invalidates the translation lookaside buffer (TLB) of the VirtualCpu
	pub fn invalidate_tlb(&self) -> Result<(), Error> {
		match_error_code(unsafe { hv_vcpu_invalidate_tlb(self.id) })
	}

	/// Enables an MSR to be used natively by the VM
	pub fn enable_native_msr(&self, msr: u32, enable: bool) -> Result<(), Error> {
		match_error_code(unsafe { hv_vcpu_enable_native_msr(self.id, msr, enable) })
	}

	/// Returns the current value of an MSR of the VirtualCpu
	pub fn read_msr(&self, msr: u32) -> Result<u64, Error> {
		let mut value: u64 = 0;

		let _error = match_error_code(unsafe { hv_vcpu_read_msr(self.id, msr, &mut value) })?;

		Ok(value)
	}

	/// Set the value of an MSR of the VirtualCpu
	pub fn write_msr(&self, msr: u32, value: u64) -> Result<(), Error> {
		match_error_code(unsafe { hv_vcpu_write_msr(self.id, msr, &(value)) })
	}

	/// Returns the current value of an architectural x86 register
	/// of the VirtualCpu
	pub fn read_register(&self, reg: &Register) -> Result<u64, Error> {
		let mut value: u64 = 0;

		match_error_code(unsafe { hv_vcpu_read_register(self.id, (*reg).clone(), &mut value) })?;

		Ok(value)
	}

	/// Sets the value of an architectural x86 register of the VirtualCpu
	pub fn write_register(&self, reg: &Register, value: u64) -> Result<(), Error> {
		match_error_code(unsafe { hv_vcpu_write_register(self.id, (*reg).clone(), value) })
	}

    /// Returns the current value of a VMCS field of the VirtualCpu
	pub fn read_vmcs(&self, field: u32) -> Result<u64, Error> {
		let mut value: u64 = 0;

		match_error_code(unsafe { hv_vmx_vcpu_read_vmcs(self.get_id(), field, &mut value) })?;

		Ok(value)
	}

	/// Sets the value of a VMCS field of the VirtualCpu
	pub fn write_vmcs(&self, field: u32, value: u64) -> Result<(), Error> {
		match_error_code(unsafe { hv_vmx_vcpu_write_vmcs(self.id, field, value) })
	}

	/// Sets the address of the guest APIC for the VirtualCpu in the
	/// guest physical address space of the VM
	pub fn set_apic_addr(&self, gpa: u64) -> Result<(), Error> {
		match_error_code(unsafe { hv_vmx_vcpu_set_apic_address(self.id, gpa) })
	}

	/// Reads the current architectural x86 floating point and SIMD state of the VirtualCpu
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

/// VMX cabability
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
pub fn read_vmx_cap(vmx_cap: &VMXCap) -> Result<u64, Error> {
	let mut value: u64 = 0;

	match_error_code(unsafe { hv_vmx_read_capability((*vmx_cap).clone(), &mut value) })?;

	Ok(value)
}

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

pub mod consts;
pub mod ffi;

use self::consts::*;
use self::ffi::*;
use crate::x86_64::vmcs::*;
use crate::{match_MemPerm, match_error_code, Error, MemPerm};
use core::fmt;
use libc::*;

/// Creates a VM instance for the current Mach task
pub fn create_vm() -> Result<(), Error> {
	match_error_code(unsafe { hv_vm_create(HV_VM_DEFAULT) })
}

/// Maps a region in the virtual address space of the current task into the guest physical
/// address space of the viurtal machine
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
	/// Virtual CPU Id
	id: u32,
	/// Virtual CPU handle
	vcpu_handle: hv_vcpuid_t,
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
	///
	/// `id` represents the internal numbering of the processor.
	pub fn new(id: u32) -> Result<VirtualCpu, Error> {
		let mut vcpu_handle: hv_vcpuid_t = 0;

		match_error_code(unsafe { hv_vcpu_create(&mut vcpu_handle, HV_VCPU_DEFAULT) })?;

		Ok(VirtualCpu { id, vcpu_handle })
	}

	pub fn get_id(&self) -> u32 {
		self.id
	}

	pub fn get_handle(&self) -> hv_vcpuid_t {
		self.vcpu_handle
	}

	/// Forces an immediate VMEXIT of the VirtualCpu
	pub fn interrupt(&self) -> Result<(), Error> {
		match_error_code(unsafe { hv_vcpu_interrupt(&(self.vcpu_handle), 1 as c_uint) })
	}

	/// Returns the cumulative execution time of the VirtualCpu in nanoseconds
	pub fn exec_time(&self) -> Result<u64, Error> {
		let mut exec_time: u64 = 0;

		let _error =
			match_error_code(unsafe { hv_vcpu_get_exec_time(self.vcpu_handle, &mut exec_time) })?;

		Ok(exec_time)
	}

	/// Forces flushing of cached VirtualCpu state
	pub fn flush(&self) -> Result<(), Error> {
		match_error_code(unsafe { hv_vcpu_flush(self.vcpu_handle) })
	}

	/// Invalidates the translation lookaside buffer (TLB) of the VirtualCpu
	pub fn invalidate_tlb(&self) -> Result<(), Error> {
		match_error_code(unsafe { hv_vcpu_invalidate_tlb(self.vcpu_handle) })
	}

	/// Enables an MSR to be used natively by the VM
	pub fn enable_native_msr(&self, msr: u32, enable: bool) -> Result<(), Error> {
		match_error_code(unsafe { hv_vcpu_enable_native_msr(self.vcpu_handle, msr, enable) })
	}

	/// Returns the current value of an MSR of the VirtualCpu
	pub fn read_msr(&self, msr: u32) -> Result<u64, Error> {
		let mut value: u64 = 0;

		let _error =
			match_error_code(unsafe { hv_vcpu_read_msr(self.vcpu_handle, msr, &mut value) })?;

		Ok(value)
	}

	/// Set the value of an MSR of the VirtualCpu
	pub fn write_msr(&self, msr: u32, value: u64) -> Result<(), Error> {
		match_error_code(unsafe { hv_vcpu_write_msr(self.vcpu_handle, msr, &(value)) })
	}

	/// Returns the current value of an architectural x86 register
	/// of the VirtualCpu
	pub fn read_register(&self, reg: &Register) -> Result<u64, Error> {
		let mut value: u64 = 0;

		match_error_code(unsafe {
			hv_vcpu_read_register(self.vcpu_handle, (*reg).clone(), &mut value)
		})?;

		Ok(value)
	}

	/// Sets the value of an architectural x86 register of the VirtualCpu
	pub fn write_register(&self, reg: &Register, value: u64) -> Result<(), Error> {
		match_error_code(unsafe { hv_vcpu_write_register(self.vcpu_handle, (*reg).clone(), value) })
	}

	/// Returns the current value of a VMCS field of the VirtualCpu
	pub fn read_vmcs(&self, field: u32) -> Result<u64, Error> {
		let mut value: u64 = 0;

		match_error_code(unsafe { hv_vmx_vcpu_read_vmcs(self.vcpu_handle, field, &mut value) })?;

		Ok(value)
	}

	/// Sets the value of a VMCS field of the VirtualCpu
	pub fn write_vmcs(&self, field: u32, value: u64) -> Result<(), Error> {
		match_error_code(unsafe { hv_vmx_vcpu_write_vmcs(self.vcpu_handle, field, value) })
	}

	/// Sets the address of the guest APIC for the VirtualCpu in the
	/// guest physical address space of the VM
	pub fn set_apic_addr(&self, gpa: u64) -> Result<(), Error> {
		match_error_code(unsafe { hv_vmx_vcpu_set_apic_address(self.vcpu_handle, gpa) })
	}

	/// Reads the current architectural x86 floating point and SIMD state of the VirtualCpu
	pub fn read_fpstate(&self, buffer: &mut [u8]) -> Result<(), Error> {
		match_error_code(unsafe {
			hv_vcpu_read_fpstate(
				self.vcpu_handle,
				buffer.as_mut_ptr() as *mut c_void,
				buffer.len() as size_t,
			)
		})
	}

	/// Sets the architectural x86 floating point and SIMD state of the VirtualCpu
	pub fn write_fpstate(&self, buffer: &[u8]) -> Result<(), Error> {
		match_error_code(unsafe {
			hv_vcpu_write_fpstate(
				self.vcpu_handle,
				buffer.as_ptr() as *const c_void,
				buffer.len() as size_t,
			)
		})
	}
}

impl fmt::Debug for VirtualCpu {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "\nDump state of CPU {}", self.id)?;
		write!(f, "VMCS:")?;
		write!(f, "-----")?;
		write!(
			f,
			"CR0: mask {:016x}  shadow {:016x}",
			self.read_vmcs(VMCS_CTRL_CR0_MASK).unwrap(),
			self.read_vmcs(VMCS_CTRL_CR0_SHADOW).unwrap()
		)?;
		write!(
			f,
			"CR4: mask {:016x}  shadow {:016x}",
			self.read_vmcs(VMCS_CTRL_CR4_MASK).unwrap(),
			self.read_vmcs(VMCS_CTRL_CR4_SHADOW).unwrap()
		)?;
		write!(
			f,
			"Pinbased: {:016x}\n1st:      {:016x}\n2nd:      {:016x}",
			self.read_vmcs(VMCS_CTRL_PIN_BASED).unwrap(),
			self.read_vmcs(VMCS_CTRL_CPU_BASED).unwrap(),
			self.read_vmcs(VMCS_CTRL_CPU_BASED2).unwrap()
		)?;
		write!(
			f,
			"Entry:    {:016x}\nExit:     {:016x}",
			self.read_vmcs(VMCS_CTRL_VMENTRY_CONTROLS).unwrap(),
			self.read_vmcs(VMCS_CTRL_VMEXIT_CONTROLS).unwrap()
		)?;

		write!(f, "\nRegisters:")?;
		write!(f, "----------")?;

		let rip = self.read_register(&Register::RIP).unwrap();
		let rflags = self.read_register(&Register::RFLAGS).unwrap();
		let rsp = self.read_register(&Register::RSP).unwrap();
		let rbp = self.read_register(&Register::RBP).unwrap();
		let rax = self.read_register(&Register::RAX).unwrap();
		let rbx = self.read_register(&Register::RBX).unwrap();
		let rcx = self.read_register(&Register::RCX).unwrap();
		let rdx = self.read_register(&Register::RDX).unwrap();
		let rsi = self.read_register(&Register::RSI).unwrap();
		let rdi = self.read_register(&Register::RDI).unwrap();
		let r8 = self.read_register(&Register::R8).unwrap();
		let r9 = self.read_register(&Register::R9).unwrap();
		let r10 = self.read_register(&Register::R10).unwrap();
		let r11 = self.read_register(&Register::R11).unwrap();
		let r12 = self.read_register(&Register::R12).unwrap();
		let r13 = self.read_register(&Register::R13).unwrap();
		let r14 = self.read_register(&Register::R14).unwrap();
		let r15 = self.read_register(&Register::R15).unwrap();

		write!(
			f,
			"rip: {rip:016x}   rsp: {rsp:016x} flags: {rflags:016x}\n\
			rax: {rax:016x}   rbx: {rbx:016x}   rcx: {rcx:016x}\n\
			rdx: {rdx:016x}   rsi: {rsi:016x}   rdi: {rdi:016x}\n\
			rbp: {rbp:016x}    r8: {r8:016x}    r9: {r9:016x}\n\
			r10: {r10:016x}   r11: {r11:016x}   r12: {r12:016x}\n\
			r13: {r13:016x}   r14: {r14:016x}   r15: {r15:016x}\n"
		)?;

		let cr0 = self.read_register(&Register::CR0).unwrap();
		let cr2 = self.read_register(&Register::CR2).unwrap();
		let cr3 = self.read_register(&Register::CR3).unwrap();
		let cr4 = self.read_register(&Register::CR4).unwrap();
		let efer = self.read_vmcs(VMCS_GUEST_IA32_EFER).unwrap();

		write!(f,
			"cr0: {cr0:016x}   cr2: {cr2:016x}   cr3: {cr3:016x}\ncr4: {cr4:016x}  efer: {efer:016x}"
		)?;

		write!(f, "\nSegment registers:")?;
		write!(f, "------------------")?;
		write!(
			f,
			"register  selector  base              limit     type  p dpl db s l g avl"
		)?;

		let cs = self.read_register(&Register::CS).unwrap();
		let ds = self.read_register(&Register::DS).unwrap();
		let es = self.read_register(&Register::ES).unwrap();
		let ss = self.read_register(&Register::SS).unwrap();
		let fs = self.read_register(&Register::FS).unwrap();
		let gs = self.read_register(&Register::GS).unwrap();
		let tr = self.read_register(&Register::TR).unwrap();
		let ldtr = self.read_register(&Register::LDTR).unwrap();

		let cs_limit = self.read_vmcs(VMCS_GUEST_CS_LIMIT).unwrap();
		let cs_base = self.read_vmcs(VMCS_GUEST_CS_BASE).unwrap();
		let cs_ar = self.read_vmcs(VMCS_GUEST_CS_AR).unwrap();
		let ss_limit = self.read_vmcs(VMCS_GUEST_SS_LIMIT).unwrap();
		let ss_base = self.read_vmcs(VMCS_GUEST_SS_BASE).unwrap();
		let ss_ar = self.read_vmcs(VMCS_GUEST_SS_AR).unwrap();
		let ds_limit = self.read_vmcs(VMCS_GUEST_DS_LIMIT).unwrap();
		let ds_base = self.read_vmcs(VMCS_GUEST_DS_BASE).unwrap();
		let ds_ar = self.read_vmcs(VMCS_GUEST_DS_AR).unwrap();
		let es_limit = self.read_vmcs(VMCS_GUEST_ES_LIMIT).unwrap();
		let es_base = self.read_vmcs(VMCS_GUEST_ES_BASE).unwrap();
		let es_ar = self.read_vmcs(VMCS_GUEST_ES_AR).unwrap();
		let fs_limit = self.read_vmcs(VMCS_GUEST_FS_LIMIT).unwrap();
		let fs_base = self.read_vmcs(VMCS_GUEST_FS_BASE).unwrap();
		let fs_ar = self.read_vmcs(VMCS_GUEST_FS_AR).unwrap();
		let gs_limit = self.read_vmcs(VMCS_GUEST_GS_LIMIT).unwrap();
		let gs_base = self.read_vmcs(VMCS_GUEST_GS_BASE).unwrap();
		let gs_ar = self.read_vmcs(VMCS_GUEST_GS_AR).unwrap();
		let tr_limit = self.read_vmcs(VMCS_GUEST_TR_LIMIT).unwrap();
		let tr_base = self.read_vmcs(VMCS_GUEST_TR_BASE).unwrap();
		let tr_ar = self.read_vmcs(VMCS_GUEST_TR_AR).unwrap();
		let ldtr_limit = self.read_vmcs(VMCS_GUEST_LDTR_LIMIT).unwrap();
		let ldtr_base = self.read_vmcs(VMCS_GUEST_LDTR_BASE).unwrap();
		let ldtr_ar = self.read_vmcs(VMCS_GUEST_LDTR_AR).unwrap();

		/*
		 * Format of Access Rights
		 * -----------------------
		 * 3-0 : Segment type
		 * 4   : S — Descriptor type (0 = system; 1 = code or data)
		 * 6-5 : DPL — Descriptor privilege level
		 * 7   : P — Segment present
		 * 11-8: Reserved
		 * 12  : AVL — Available for use by system software
		 * 13  : L — 64-bit mode active (for CS only)
		 * 14  : D/B — Default operation size (0 = 16-bit segment; 1 = 32-bit segment)
		 * 15  : G — Granularity
		 * 16  : Segment unusable (0 = usable; 1 = unusable)
		 *
		 * Output sequence: type p dpl db s l g avl
		 */
		write!(f, "cs        {:04x}      {:016x}  {:08x}  {:02x}    {:x} {:x}   {:x}  {:x} {:x} {:x} {:x}",
			cs, cs_base, cs_limit, (cs_ar) & 0xf, (cs_ar >> 7) & 0x1, (cs_ar >> 5) & 0x3, (cs_ar >> 14) & 0x1,
			(cs_ar >> 4) & 0x1, (cs_ar >> 13) & 0x1, (cs_ar >> 15) & 0x1, (cs_ar >> 12) & 1)?;
		write!(f, "ss        {:04x}      {:016x}  {:08x}  {:02x}    {:x} {:x}   {:x}  {:x} {:x} {:x} {:x}",
			ss, ss_base, ss_limit, (ss_ar) & 0xf, (ss_ar >> 7) & 0x1, (ss_ar >> 5) & 0x3, (ss_ar >> 14) & 0x1,
			(ss_ar >> 4) & 0x1, (ss_ar >> 13) & 0x1, (ss_ar >> 15) & 0x1, (ss_ar >> 12) & 1)?;
		write!(f, "ds        {:04x}      {:016x}  {:08x}  {:02x}    {:x} {:x}   {:x}  {:x} {:x} {:x} {:x}",
			ds, ds_base, ds_limit, (ds_ar) & 0xf, (ds_ar >> 7) & 0x1, (ds_ar >> 5) & 0x3, (ds_ar >> 14) & 0x1,
			(ds_ar >> 4) & 0x1, (ds_ar >> 13) & 0x1, (ds_ar >> 15) & 0x1, (ds_ar >> 12) & 1)?;
		write!(f, "es        {:04x}      {:016x}  {:08x}  {:02x}    {:x} {:x}   {:x}  {:x} {:x} {:x} {:x}",
			es, es_base, es_limit, (es_ar) & 0xf, (es_ar >> 7) & 0x1, (es_ar >> 5) & 0x3, (es_ar >> 14) & 0x1,
			(es_ar >> 4) & 0x1, (es_ar >> 13) & 0x1, (es_ar >> 15) & 0x1, (es_ar >> 12) & 1)?;
		write!(f, "fs        {:04x}      {:016x}  {:08x}  {:02x}    {:x} {:x}   {:x}  {:x} {:x} {:x} {:x}",
			fs, fs_base, fs_limit, (fs_ar) & 0xf, (fs_ar >> 7) & 0x1, (fs_ar >> 5) & 0x3, (fs_ar >> 14) & 0x1,
			(fs_ar >> 4) & 0x1, (fs_ar >> 13) & 0x1, (fs_ar >> 15) & 0x1, (fs_ar >> 12) & 1)?;
		write!(f, "gs        {:04x}      {:016x}  {:08x}  {:02x}    {:x} {:x}   {:x}  {:x} {:x} {:x} {:x}",
			gs, gs_base, gs_limit, (gs_ar) & 0xf, (gs_ar >> 7) & 0x1, (gs_ar >> 5) & 0x3, (gs_ar >> 14) & 0x1,
			(gs_ar >> 4) & 0x1, (gs_ar >> 13) & 0x1, (gs_ar >> 15) & 0x1, (gs_ar >> 12) & 1)?;
		write!(f, "tr        {:04x}      {:016x}  {:08x}  {:02x}    {:x} {:x}   {:x}  {:x} {:x} {:x} {:x}",
			tr, tr_base, tr_limit, (tr_ar) & 0xf, (tr_ar >> 7) & 0x1, (tr_ar >> 5) & 0x3, (tr_ar >> 14) & 0x1,
			(tr_ar >> 4) & 0x1, (tr_ar >> 13) & 0x1, (tr_ar >> 15) & 0x1, (tr_ar >> 12) & 1)?;
		write!(f, "ldt       {:04x}      {:016x}  {:08x}  {:02x}    {:x} {:x}   {:x}  {:x} {:x} {:x} {:x}",
			ldtr, ldtr_base, ldtr_limit, (ldtr_ar) & 0xf, (ldtr_ar >> 7) & 0x1, (ldtr_ar >> 5) & 0x3, (ldtr_ar >> 14) & 0x1,
			(ldtr_ar >> 4) & 0x1, (ldtr_ar >> 13) & 0x1, (ldtr_ar >> 15) & 0x1, (ldtr_ar >> 12) & 1)?;

		let gdt_base = self.read_vmcs(VMCS_GUEST_GDTR_BASE).unwrap();
		let gdt_limit = self.read_vmcs(VMCS_GUEST_GDTR_LIMIT).unwrap();
		write!(f, "gdt                 {gdt_base:016x}  {gdt_limit:08x}")?;
		let idt_base = self.read_vmcs(VMCS_GUEST_IDTR_BASE).unwrap();
		let idt_limit = self.read_vmcs(VMCS_GUEST_IDTR_LIMIT).unwrap();
		write!(f, "idt                 {idt_base:016x}  {idt_limit:08x}")?;
		write!(
			f,
			"VMCS link pointer   {:016x}",
			self.read_vmcs(VMCS_GUEST_LINK_POINTER).unwrap()
		)
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

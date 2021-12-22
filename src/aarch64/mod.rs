pub mod ffi;

use self::ffi::*;
use crate::{match_MemPerm, match_error_code, Error, MemPerm};
use libc::*;
use std::ptr::null_mut;

/// Creates a VM instance for the current Mach task
pub fn create_vm() -> Result<(), Error> {
	match_error_code(unsafe { hv_vm_create(null_mut()) })
}

/// Maps a region in the virtual address space of the current task into the guest physical
/// address space of the virutal machine
pub fn map_mem(mem: &[u8], ipa: u64, mem_perm: MemPerm) -> Result<(), Error> {
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
pub fn unmap_mem(ipa: u64, size: usize) -> Result<(), Error> {
	match_error_code(unsafe { hv_vm_unmap(ipa as hv_ipa_t, size as size_t) })
}

/// Modifies the permissions of a region in the guest physical address space of the virtual
/// machine
pub fn protect_mem(ipa: u64, size: usize, mem_perm: MemPerm) -> Result<(), Error> {
	match_error_code(unsafe {
		hv_vm_protect(ipa as hv_ipa_t, size as size_t, match_MemPerm(mem_perm))
	})
}

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
	/// Virtual CPU handle
	id: hv_vcpu_t,

	/// VirtualCPU exit informations.
	vcpu_exit: *const hv_vcpu_exit_t,
}

/// aarch64 architectural register
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

/// ARM system register.
#[derive(Copy, Clone, Debug)]
pub enum SystemRegister {
	/// DBGBVR0_EL1 register.
	DBGBVR0_EL1,

	/// DBGBCR0_EL1 register.
	DBGBCR0_EL1,

	/// DBGWVR0_EL1 register.
	DBGWVR0_EL1,

	/// DBGWCR0_EL1 register.
	DBGWCR0_EL1,

	/// DBGBVR1_EL1 register.
	DBGBVR1_EL1,

	/// DBGBCR1_EL1 register.
	DBGBCR1_EL1,

	/// DBGWVR1_EL1 register.
	DBGWVR1_EL1,

	/// DBGWCR1_EL1 register.
	DBGWCR1_EL1,

	/// MDCCINT_EL1 register.
	MDCCINT_EL1,

	/// MDSCR_EL1 register.
	MDSCR_EL1,

	/// DBGBVR2_EL1 register.
	DBGBVR2_EL1,

	/// DBGBCR2_EL1 register.
	DBGBCR2_EL1,

	/// DBGWVR2_EL1 register.
	DBGWVR2_EL1,

	/// DBGWCR2_EL1 register.
	DBGWCR2_EL1,

	/// DBGBVR3_EL1 register.
	DBGBVR3_EL1,

	/// DBGBCR3_EL1 register.
	DBGBCR3_EL1,

	/// DBGWVR3_EL1 register.
	DBGWVR3_EL1,

	/// DBGWCR3_EL1 register.
	DBGWCR3_EL1,

	/// DBGBVR4_EL1 register.
	DBGBVR4_EL1,

	/// DBGBCR4_EL1 register.
	DBGBCR4_EL1,

	/// DBGWVR4_EL1 register.
	DBGWVR4_EL1,

	/// DBGWCR4_EL1 register.
	DBGWCR4_EL1,

	/// DBGBVR5_EL1 register.
	DBGBVR5_EL1,

	/// DBGBCR5_EL1 register.
	DBGBCR5_EL1,

	/// DBGWVR5_EL1 register.
	DBGWVR5_EL1,

	/// DBGWCR5_EL1 register.
	DBGWCR5_EL1,

	/// DBGBVR6_EL1 register.
	DBGBVR6_EL1,

	/// DBGBCR6_EL1 register.
	DBGBCR6_EL1,

	/// DBGWVR6_EL1 register.
	DBGWVR6_EL1,

	/// DBGWCR6_EL1 register.
	DBGWCR6_EL1,

	/// DBGBVR7_EL1 register.
	DBGBVR7_EL1,

	/// DBGBCR7_EL1 register.
	DBGBCR7_EL1,

	/// DBGWVR7_EL1 register.
	DBGWVR7_EL1,

	/// DBGWCR7_EL1 register.
	DBGWCR7_EL1,

	/// DBGBVR8_EL1 register.
	DBGBVR8_EL1,

	/// DBGBCR8_EL1 register.
	DBGBCR8_EL1,

	/// DBGWVR8_EL1 register.
	DBGWVR8_EL1,

	/// DBGWCR8_EL1 register.
	DBGWCR8_EL1,

	/// DBGBVR9_EL1 register.
	DBGBVR9_EL1,

	/// DBGBCR9_EL1 register.
	DBGBCR9_EL1,

	/// DBGWVR9_EL1 register.
	DBGWVR9_EL1,

	/// DBGWCR9_EL1 register.
	DBGWCR9_EL1,

	/// DBGBVR10_EL1 register.
	DBGBVR10_EL1,

	/// DBGBCR10_EL1 register.
	DBGBCR10_EL1,

	/// DBGWVR10_EL1 register.
	DBGWVR10_EL1,

	/// DBGWCR10_EL1 register.
	DBGWCR10_EL1,

	/// DBGBVR11_EL1 register.
	DBGBVR11_EL1,

	/// DBGBCR11_EL1 register.
	DBGBCR11_EL1,

	/// DBGWVR11_EL1 register.
	DBGWVR11_EL1,

	/// DBGWCR11_EL1 register.
	DBGWCR11_EL1,

	/// DBGBVR12_EL1 register.
	DBGBVR12_EL1,

	/// DBGBCR12_EL1 register.
	DBGBCR12_EL1,

	/// DBGWVR12_EL1 register.
	DBGWVR12_EL1,

	/// DBGWCR12_EL1 register.
	DBGWCR12_EL1,

	/// DBGBVR13_EL1 register.
	DBGBVR13_EL1,

	/// DBGBCR13_EL1 register.
	DBGBCR13_EL1,

	/// DBGWVR13_EL1 register.
	DBGWVR13_EL1,

	/// DBGWCR13_EL1 register.
	DBGWCR13_EL1,

	/// DBGBVR14_EL1 register.
	DBGBVR14_EL1,

	/// DBGBCR14_EL1 register.
	DBGBCR14_EL1,

	/// DBGWVR14_EL1 register.
	DBGWVR14_EL1,

	/// DBGWCR14_EL1 register.
	DBGWCR14_EL1,

	/// DBGBVR15_EL1 register.
	DBGBVR15_EL1,

	/// DBGBCR15_EL1 register.
	DBGBCR15_EL1,

	/// DBGWVR15_EL1 register.
	DBGWVR15_EL1,

	/// DBGWCR15_EL1 register.
	DBGWCR15_EL1,

	/// MIDR_EL1 register.
	MIDR_EL1,

	/// MPIDR_EL1 register.
	MPIDR_EL1,

	/// ID_AA64PFR0_EL1 register.
	ID_AA64PFR0_EL1,

	/// ID_AA64PFR1_EL1 register.
	ID_AA64PFR1_EL1,

	/// ID_AA64DFR0_EL1 register.
	ID_AA64DFR0_EL1,

	/// ID_AA64DFR1_EL1 register.
	ID_AA64DFR1_EL1,

	/// ID_AA64ISAR0_EL1 register.
	ID_AA64ISAR0_EL1,

	/// ID_AA64ISAR1_EL1 register.
	ID_AA64ISAR1_EL1,

	/// AA64MMFR0_EL1 register.
	ID_AA64MMFR0_EL1,

	/// ID_AA64MMFR1_EL1 register.
	ID_AA64MMFR1_EL1,

	/// AA64MMFR2_EL1 register.
	ID_AA64MMFR2_EL1,

	/// SCTLR_EL1 register.
	SCTLR_EL1,

	/// CPACR_EL1 register.
	CPACR_EL1,

	/// TTBR0_EL1 register.
	TTBR0_EL1,

	/// TTBR1_EL1 register.
	TTBR1_EL1,

	/// TCR_EL1 register.
	TCR_EL1,

	/// APIAKEYLO_EL1 register.
	APIAKEYLO_EL1,

	/// APIAKEYHI_EL1 register.
	APIAKEYHI_EL1,

	/// APIBKEYLO_EL1 register.
	APIBKEYLO_EL1,

	/// APIBKEYHI_EL1 register.
	APIBKEYHI_EL1,

	/// APDAKEYLO_EL1 register.
	APDAKEYLO_EL1,

	/// APDAKEYHI_EL1 register.
	APDAKEYHI_EL1,

	/// APDBKEYLO_EL1 register.
	APDBKEYLO_EL1,

	/// APDBKEYHI_EL1 register.
	APDBKEYHI_EL1,

	/// APGAKEYLO_EL1 register.
	APGAKEYLO_EL1,

	/// APGAKEYHI_EL1 register.
	APGAKEYHI_EL1,

	/// SPSR_EL1 register.
	SPSR_EL1,

	/// ELR_EL1 register.
	ELR_EL1,

	/// SP_EL0 register.
	SP_EL0,

	/// AFSR0_EL1 register.
	AFSR0_EL1,

	/// AFSR1_EL1 register.
	AFSR1_EL1,

	/// ESR_EL1 register.
	ESR_EL1,

	/// FAR_EL1 register.
	FAR_EL1,

	/// PAR_EL1 register.
	PAR_EL1,

	/// MAIR_EL1 register.
	MAIR_EL1,

	/// AMAIR_EL1 register.
	AMAIR_EL1,

	/// VBAR_EL1 register.
	VBAR_EL1,

	/// CONTEXTIDR_EL1 register.
	CONTEXTIDR_EL1,

	/// TPIDR_EL1 register.
	TPIDR_EL1,

	/// CNTKCTL_EL1 register.
	CNTKCTL_EL1,

	/// CSSELR_EL1 register.
	CSSELR_EL1,

	/// TPIDR_EL0 register.
	TPIDR_EL0,

	/// TPIDRRO_EL0 register.
	TPIDRRO_EL0,

	/// CNTV_CTL_EL0 register.
	CNTV_CTL_EL0,

	/// CNTV_CVAL_EL0 register.
	CNTV_CVAL_EL0,

	/// SP_EL1 register.
	SP_EL1,
}

impl From<SystemRegister> for hv_sys_reg_t {
	fn from(value: SystemRegister) -> hv_sys_reg_t {
		match value {
			SystemRegister::DBGBVR0_EL1 => HV_SYS_REG_DBGBVR0_EL1,
			SystemRegister::DBGBCR0_EL1 => HV_SYS_REG_DBGBCR0_EL1,
			SystemRegister::DBGWVR0_EL1 => HV_SYS_REG_DBGWVR0_EL1,
			SystemRegister::DBGWCR0_EL1 => HV_SYS_REG_DBGWCR0_EL1,
			SystemRegister::DBGBVR1_EL1 => HV_SYS_REG_DBGBVR1_EL1,
			SystemRegister::DBGBCR1_EL1 => HV_SYS_REG_DBGBCR1_EL1,
			SystemRegister::DBGWVR1_EL1 => HV_SYS_REG_DBGWVR1_EL1,
			SystemRegister::DBGWCR1_EL1 => HV_SYS_REG_DBGWCR1_EL1,
			SystemRegister::MDCCINT_EL1 => HV_SYS_REG_MDCCINT_EL1,
			SystemRegister::MDSCR_EL1 => HV_SYS_REG_MDSCR_EL1,
			SystemRegister::DBGBVR2_EL1 => HV_SYS_REG_DBGBVR2_EL1,
			SystemRegister::DBGBCR2_EL1 => HV_SYS_REG_DBGBCR2_EL1,
			SystemRegister::DBGWVR2_EL1 => HV_SYS_REG_DBGWVR2_EL1,
			SystemRegister::DBGWCR2_EL1 => HV_SYS_REG_DBGWCR2_EL1,
			SystemRegister::DBGBVR3_EL1 => HV_SYS_REG_DBGBVR3_EL1,
			SystemRegister::DBGBCR3_EL1 => HV_SYS_REG_DBGBCR3_EL1,
			SystemRegister::DBGWVR3_EL1 => HV_SYS_REG_DBGWVR3_EL1,
			SystemRegister::DBGWCR3_EL1 => HV_SYS_REG_DBGWCR3_EL1,
			SystemRegister::DBGBVR4_EL1 => HV_SYS_REG_DBGBVR4_EL1,
			SystemRegister::DBGBCR4_EL1 => HV_SYS_REG_DBGBCR4_EL1,
			SystemRegister::DBGWVR4_EL1 => HV_SYS_REG_DBGWVR4_EL1,
			SystemRegister::DBGWCR4_EL1 => HV_SYS_REG_DBGWCR4_EL1,
			SystemRegister::DBGBVR5_EL1 => HV_SYS_REG_DBGBVR5_EL1,
			SystemRegister::DBGBCR5_EL1 => HV_SYS_REG_DBGBCR5_EL1,
			SystemRegister::DBGWVR5_EL1 => HV_SYS_REG_DBGWVR5_EL1,
			SystemRegister::DBGWCR5_EL1 => HV_SYS_REG_DBGWCR5_EL1,
			SystemRegister::DBGBVR6_EL1 => HV_SYS_REG_DBGBVR6_EL1,
			SystemRegister::DBGBCR6_EL1 => HV_SYS_REG_DBGBCR6_EL1,
			SystemRegister::DBGWVR6_EL1 => HV_SYS_REG_DBGWVR6_EL1,
			SystemRegister::DBGWCR6_EL1 => HV_SYS_REG_DBGWCR6_EL1,
			SystemRegister::DBGBVR7_EL1 => HV_SYS_REG_DBGBVR7_EL1,
			SystemRegister::DBGBCR7_EL1 => HV_SYS_REG_DBGBCR7_EL1,
			SystemRegister::DBGWVR7_EL1 => HV_SYS_REG_DBGWVR7_EL1,
			SystemRegister::DBGWCR7_EL1 => HV_SYS_REG_DBGWCR7_EL1,
			SystemRegister::DBGBVR8_EL1 => HV_SYS_REG_DBGBVR8_EL1,
			SystemRegister::DBGBCR8_EL1 => HV_SYS_REG_DBGBCR8_EL1,
			SystemRegister::DBGWVR8_EL1 => HV_SYS_REG_DBGWVR8_EL1,
			SystemRegister::DBGWCR8_EL1 => HV_SYS_REG_DBGWCR8_EL1,
			SystemRegister::DBGBVR9_EL1 => HV_SYS_REG_DBGBVR9_EL1,
			SystemRegister::DBGBCR9_EL1 => HV_SYS_REG_DBGBCR9_EL1,
			SystemRegister::DBGWVR9_EL1 => HV_SYS_REG_DBGWVR9_EL1,
			SystemRegister::DBGWCR9_EL1 => HV_SYS_REG_DBGWCR9_EL1,
			SystemRegister::DBGBVR10_EL1 => HV_SYS_REG_DBGBVR10_EL1,
			SystemRegister::DBGBCR10_EL1 => HV_SYS_REG_DBGBCR10_EL1,
			SystemRegister::DBGWVR10_EL1 => HV_SYS_REG_DBGWVR10_EL1,
			SystemRegister::DBGWCR10_EL1 => HV_SYS_REG_DBGWCR10_EL1,
			SystemRegister::DBGBVR11_EL1 => HV_SYS_REG_DBGBVR11_EL1,
			SystemRegister::DBGBCR11_EL1 => HV_SYS_REG_DBGBCR11_EL1,
			SystemRegister::DBGWVR11_EL1 => HV_SYS_REG_DBGWVR11_EL1,
			SystemRegister::DBGWCR11_EL1 => HV_SYS_REG_DBGWCR11_EL1,
			SystemRegister::DBGBVR12_EL1 => HV_SYS_REG_DBGBVR12_EL1,
			SystemRegister::DBGBCR12_EL1 => HV_SYS_REG_DBGBCR12_EL1,
			SystemRegister::DBGWVR12_EL1 => HV_SYS_REG_DBGWVR12_EL1,
			SystemRegister::DBGWCR12_EL1 => HV_SYS_REG_DBGWCR12_EL1,
			SystemRegister::DBGBVR13_EL1 => HV_SYS_REG_DBGBVR13_EL1,
			SystemRegister::DBGBCR13_EL1 => HV_SYS_REG_DBGBCR13_EL1,
			SystemRegister::DBGWVR13_EL1 => HV_SYS_REG_DBGWVR13_EL1,
			SystemRegister::DBGWCR13_EL1 => HV_SYS_REG_DBGWCR13_EL1,
			SystemRegister::DBGBVR14_EL1 => HV_SYS_REG_DBGBVR14_EL1,
			SystemRegister::DBGBCR14_EL1 => HV_SYS_REG_DBGBCR14_EL1,
			SystemRegister::DBGWVR14_EL1 => HV_SYS_REG_DBGWVR14_EL1,
			SystemRegister::DBGWCR14_EL1 => HV_SYS_REG_DBGWCR14_EL1,
			SystemRegister::DBGBVR15_EL1 => HV_SYS_REG_DBGBVR15_EL1,
			SystemRegister::DBGBCR15_EL1 => HV_SYS_REG_DBGBCR15_EL1,
			SystemRegister::DBGWVR15_EL1 => HV_SYS_REG_DBGWVR15_EL1,
			SystemRegister::DBGWCR15_EL1 => HV_SYS_REG_DBGWCR15_EL1,
			SystemRegister::MIDR_EL1 => HV_SYS_REG_MIDR_EL1,
			SystemRegister::MPIDR_EL1 => HV_SYS_REG_MPIDR_EL1,
			SystemRegister::ID_AA64PFR0_EL1 => HV_SYS_REG_ID_AA64PFR0_EL1,
			SystemRegister::ID_AA64PFR1_EL1 => HV_SYS_REG_ID_AA64PFR1_EL1,
			SystemRegister::ID_AA64DFR0_EL1 => HV_SYS_REG_ID_AA64DFR0_EL1,
			SystemRegister::ID_AA64DFR1_EL1 => HV_SYS_REG_ID_AA64DFR1_EL1,
			SystemRegister::ID_AA64ISAR0_EL1 => HV_SYS_REG_ID_AA64ISAR0_EL1,
			SystemRegister::ID_AA64ISAR1_EL1 => HV_SYS_REG_ID_AA64ISAR1_EL1,
			SystemRegister::ID_AA64MMFR0_EL1 => HV_SYS_REG_ID_AA64MMFR0_EL1,
			SystemRegister::ID_AA64MMFR1_EL1 => HV_SYS_REG_ID_AA64MMFR1_EL1,
			SystemRegister::ID_AA64MMFR2_EL1 => HV_SYS_REG_ID_AA64MMFR2_EL1,
			SystemRegister::SCTLR_EL1 => HV_SYS_REG_SCTLR_EL1,
			SystemRegister::CPACR_EL1 => HV_SYS_REG_CPACR_EL1,
			SystemRegister::TTBR0_EL1 => HV_SYS_REG_TTBR0_EL1,
			SystemRegister::TTBR1_EL1 => HV_SYS_REG_TTBR1_EL1,
			SystemRegister::TCR_EL1 => HV_SYS_REG_TCR_EL1,
			SystemRegister::APIAKEYLO_EL1 => HV_SYS_REG_APIAKEYLO_EL1,
			SystemRegister::APIAKEYHI_EL1 => HV_SYS_REG_APIAKEYHI_EL1,
			SystemRegister::APIBKEYLO_EL1 => HV_SYS_REG_APIBKEYLO_EL1,
			SystemRegister::APIBKEYHI_EL1 => HV_SYS_REG_APIBKEYHI_EL1,
			SystemRegister::APDAKEYLO_EL1 => HV_SYS_REG_APDAKEYLO_EL1,
			SystemRegister::APDAKEYHI_EL1 => HV_SYS_REG_APDAKEYHI_EL1,
			SystemRegister::APDBKEYLO_EL1 => HV_SYS_REG_APDBKEYLO_EL1,
			SystemRegister::APDBKEYHI_EL1 => HV_SYS_REG_APDBKEYHI_EL1,
			SystemRegister::APGAKEYLO_EL1 => HV_SYS_REG_APGAKEYLO_EL1,
			SystemRegister::APGAKEYHI_EL1 => HV_SYS_REG_APGAKEYHI_EL1,
			SystemRegister::SPSR_EL1 => HV_SYS_REG_SPSR_EL1,
			SystemRegister::ELR_EL1 => HV_SYS_REG_ELR_EL1,
			SystemRegister::SP_EL0 => HV_SYS_REG_SP_EL0,
			SystemRegister::AFSR0_EL1 => HV_SYS_REG_AFSR0_EL1,
			SystemRegister::AFSR1_EL1 => HV_SYS_REG_AFSR1_EL1,
			SystemRegister::ESR_EL1 => HV_SYS_REG_ESR_EL1,
			SystemRegister::FAR_EL1 => HV_SYS_REG_FAR_EL1,
			SystemRegister::PAR_EL1 => HV_SYS_REG_PAR_EL1,
			SystemRegister::MAIR_EL1 => HV_SYS_REG_MAIR_EL1,
			SystemRegister::AMAIR_EL1 => HV_SYS_REG_AMAIR_EL1,
			SystemRegister::VBAR_EL1 => HV_SYS_REG_VBAR_EL1,
			SystemRegister::CONTEXTIDR_EL1 => HV_SYS_REG_CONTEXTIDR_EL1,
			SystemRegister::TPIDR_EL1 => HV_SYS_REG_TPIDR_EL1,
			SystemRegister::CNTKCTL_EL1 => HV_SYS_REG_CNTKCTL_EL1,
			SystemRegister::CSSELR_EL1 => HV_SYS_REG_CSSELR_EL1,
			SystemRegister::TPIDR_EL0 => HV_SYS_REG_TPIDR_EL0,
			SystemRegister::TPIDRRO_EL0 => HV_SYS_REG_TPIDRRO_EL0,
			SystemRegister::CNTV_CTL_EL0 => HV_SYS_REG_CNTV_CTL_EL0,
			SystemRegister::CNTV_CVAL_EL0 => HV_SYS_REG_CNTV_CVAL_EL0,
			SystemRegister::SP_EL1 => HV_SYS_REG_SP_EL1,
		}
	}
}

impl VirtualCpu {
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

	pub fn get_id(&self) -> hv_vcpu_t {
		self.id
	}

	pub fn exit_reason(&self) -> VirtualCpuExitReason {
		VirtualCpuExitReason::from(unsafe { *self.vcpu_exit })
	}

	/// Returns the current value of an architectural aarch64 register
	/// of the VirtualCpu
	pub fn read_register(&self, reg: Register) -> Result<u64, Error> {
		let mut value: u64 = 0;

		match_error_code(unsafe {
			hv_vcpu_get_reg(self.id, hv_reg_t::from(reg), &mut value as *mut u64)
		})?;

		Ok(value)
	}

	/// Sets the value of an architectural x86 register of the VirtualCpu
	pub fn write_register(&self, reg: Register, value: u64) -> Result<(), Error> {
		match_error_code(unsafe { hv_vcpu_set_reg(self.id, hv_reg_t::from(reg), value) })
	}

	/// Gets a system register value.
	pub fn read_system_register(&self, reg: SystemRegister) -> Result<u64, Error> {
		let mut value: u64 = 0;

		match_error_code(unsafe {
			hv_vcpu_get_sys_reg(self.id, hv_sys_reg_t::from(reg), &mut value as *mut u64)
		})?;

		Ok(value)
	}

	/// Gets a system register value.
	pub fn write_system_register(&self, reg: SystemRegister, value: u64) -> Result<(), Error> {
		match_error_code(unsafe { hv_vcpu_set_sys_reg(self.id, hv_sys_reg_t::from(reg), value) })
	}
}

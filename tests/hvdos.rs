//! Example hypervisor and 16 bits VM from https://github.com/mist64/hvdos/blob/master/hvdos.c
//! original blog post at http://www.pagetable.com/?p=764
//! guest VM code taken from https://lwn.net/Articles/658511/
extern crate xhypervisor;

use std::alloc::{alloc, dealloc, Layout};
use std::io::Write;
use std::slice;
use xhypervisor::consts::vmcs::*;
use xhypervisor::consts::vmx_exit::*;
use xhypervisor::ffi::*;
use xhypervisor::*;

pub fn rreg(vcpu: hv_vcpuid_t, reg: x86Reg) -> u64 {
	let mut v: u64 = 0;

	unsafe {
		let res = hv_vcpu_read_register(vcpu, reg, &mut v);
		if res != 0 {
			panic!("rreg res: {}", res);
		}
	}
	return v;
}

/* write GPR */
pub fn wreg(vcpu: hv_vcpuid_t, reg: x86Reg, v: u64) {
	unsafe {
		let res = hv_vcpu_write_register(vcpu, reg, v);
		if res != 0 {
			panic!("wreg res: {}", res);
		}
	}
}

/* read VMCS field */
pub fn rvmcs(vcpu: hv_vcpuid_t, field: u32) -> u64 {
	let mut v: u64 = 0;

	unsafe {
		let res = hv_vmx_vcpu_read_vmcs(vcpu, field, &mut v);
		if res != 0 {
			panic!("rvcms res: {}", res);
		}
	}

	return v;
}

/* write VMCS field */
pub fn wvmcs(vcpu: hv_vcpuid_t, field: u32, v: u64) {
	unsafe {
		let res = hv_vmx_vcpu_write_vmcs(vcpu, field, v);
		if res != 0 {
			panic!("wvcms res: {}", res);
		}
	}
}

/* desired control word constrained by hardware/hypervisor capabilities */
pub fn cap2ctrl(cap: u64, ctrl: u64) -> u64 {
	(ctrl | (cap & 0xffffffff)) & (cap >> 32)
}

#[test]
fn vm_create() {
	unsafe {
		let mut res = hv_vm_create(HV_VM_DEFAULT as u64);
		if res != 0 {
			panic!("vm create res: {}", res);
		}

		let mut vmx_cap_pinbased: u64 = 0;
		let mut vmx_cap_procbased: u64 = 0;
		let mut vmx_cap_procbased2: u64 = 0;
		let mut vmx_cap_entry: u64 = 0;

		res = hv_vmx_read_capability(VMXCap::PINBASED, &mut vmx_cap_pinbased);
		if res != 0 {
			panic!("vmx read capability res: {}", res);
		}
		res = hv_vmx_read_capability(VMXCap::PROCBASED, &mut vmx_cap_procbased);
		if res != 0 {
			panic!("vmx read capability res: {}", res);
		}
		res = hv_vmx_read_capability(VMXCap::PROCBASED2, &mut vmx_cap_procbased2);
		if res != 0 {
			panic!("vmx read capability res: {}", res);
		}
		res = hv_vmx_read_capability(VMXCap::ENTRY, &mut vmx_cap_entry);
		if res != 0 {
			panic!("vmx read capability res: {}", res);
		}
		println!(
			"capabilities: pinbased: {} procbased: {} procbased2: {} entry: {}",
			vmx_cap_pinbased, vmx_cap_procbased, vmx_cap_procbased2, vmx_cap_entry
		);

		let capacity: usize = 4 * 1024;
		let layout: Layout = Layout::from_size_align(capacity, 4096).unwrap();
		let mem_raw = alloc(layout);

		//let mut mem = Vec::with_capacity(capacity);
		//mem.extend(repeat(0).take(capacity));

		println!("allocating memory at {:?}", mem_raw);
		//map the vec at address 0
		let mem = slice::from_raw_parts_mut(mem_raw, capacity);
		map_mem(mem, 0, &MemPerm::ExecAndWrite).unwrap();

		/*res = hv_vm_map(mem_raw as *mut c_void, 0, capacity,
		  Enum_Unnamed4::HV_MEMORY_READ as u64  |
		  Enum_Unnamed4::HV_MEMORY_WRITE as u64 |
		  Enum_Unnamed4::HV_MEMORY_EXEC as u64);
		if res != 0 {
		  panic!("vm map res: {}", res);
		}

		let mem = slice::from_raw_parts_mut(mem_raw, capacity);*/

		let mut vcpu: hv_vcpuid_t = 0;

		res = hv_vcpu_create(&mut vcpu, HV_VCPU_DEFAULT as u64);
		if res != 0 {
			panic!("vcpu create res: {}", res);
		}

		println!("vcpu id: {}", vcpu);

		const VMCS_PRI_PROC_BASED_CTLS_HLT: u64 = 1 << 7;
		const VMCS_PRI_PROC_BASED_CTLS_CR8_LOAD: u64 = 1 << 19;
		const VMCS_PRI_PROC_BASED_CTLS_CR8_STORE: u64 = 1 << 20;

		/* set VMCS control fields */
		wvmcs(
			vcpu,
			VMCS_CTRL_PIN_BASED as u32,
			cap2ctrl(vmx_cap_pinbased, 0),
		);
		wvmcs(
			vcpu,
			VMCS_CTRL_CPU_BASED as u32,
			cap2ctrl(
				vmx_cap_procbased,
				VMCS_PRI_PROC_BASED_CTLS_HLT
					| VMCS_PRI_PROC_BASED_CTLS_CR8_LOAD
					| VMCS_PRI_PROC_BASED_CTLS_CR8_STORE,
			),
		);
		wvmcs(
			vcpu,
			VMCS_CTRL_CPU_BASED2 as u32,
			cap2ctrl(vmx_cap_procbased2, 0),
		);
		wvmcs(
			vcpu,
			VMCS_CTRL_VMENTRY_CONTROLS as u32,
			cap2ctrl(vmx_cap_entry, 0),
		);
		wvmcs(vcpu, VMCS_CTRL_EXC_BITMAP as u32, 0xffffffff);
		wvmcs(vcpu, VMCS_CTRL_CR0_MASK as u32, 0x60000000);
		wvmcs(vcpu, VMCS_CTRL_CR0_SHADOW as u32, 0);
		wvmcs(vcpu, VMCS_CTRL_CR4_MASK as u32, 0);
		wvmcs(vcpu, VMCS_CTRL_CR4_SHADOW as u32, 0);
		/* set VMCS guest state fields */
		wvmcs(vcpu, VMCS_GUEST_CS as u32, 0);
		wvmcs(vcpu, VMCS_GUEST_CS_LIMIT as u32, 0xffff);
		wvmcs(vcpu, VMCS_GUEST_CS_AR as u32, 0x9b);
		wvmcs(vcpu, VMCS_GUEST_CS_BASE as u32, 0);

		wvmcs(vcpu, VMCS_GUEST_DS as u32, 0);
		wvmcs(vcpu, VMCS_GUEST_DS_LIMIT as u32, 0xffff);
		wvmcs(vcpu, VMCS_GUEST_DS_AR as u32, 0x93);
		wvmcs(vcpu, VMCS_GUEST_DS_BASE as u32, 0);

		wvmcs(vcpu, VMCS_GUEST_ES as u32, 0);
		wvmcs(vcpu, VMCS_GUEST_ES_LIMIT as u32, 0xffff);
		wvmcs(vcpu, VMCS_GUEST_ES_AR as u32, 0x93);
		wvmcs(vcpu, VMCS_GUEST_ES_BASE as u32, 0);

		wvmcs(vcpu, VMCS_GUEST_FS as u32, 0);
		wvmcs(vcpu, VMCS_GUEST_FS_LIMIT as u32, 0xffff);
		wvmcs(vcpu, VMCS_GUEST_FS_AR as u32, 0x93);
		wvmcs(vcpu, VMCS_GUEST_FS_BASE as u32, 0);

		wvmcs(vcpu, VMCS_GUEST_GS as u32, 0);
		wvmcs(vcpu, VMCS_GUEST_GS_LIMIT as u32, 0xffff);
		wvmcs(vcpu, VMCS_GUEST_GS_AR as u32, 0x93);
		wvmcs(vcpu, VMCS_GUEST_GS_BASE as u32, 0);

		wvmcs(vcpu, VMCS_GUEST_SS as u32, 0);
		wvmcs(vcpu, VMCS_GUEST_SS_LIMIT as u32, 0xffff);
		wvmcs(vcpu, VMCS_GUEST_SS_AR as u32, 0x93);
		wvmcs(vcpu, VMCS_GUEST_SS_BASE as u32, 0);

		wvmcs(vcpu, VMCS_GUEST_LDTR as u32, 0);
		wvmcs(vcpu, VMCS_GUEST_LDTR_LIMIT as u32, 0);
		wvmcs(vcpu, VMCS_GUEST_LDTR_AR as u32, 0x10000);
		wvmcs(vcpu, VMCS_GUEST_LDTR_BASE as u32, 0);

		wvmcs(vcpu, VMCS_GUEST_TR as u32, 0);
		wvmcs(vcpu, VMCS_GUEST_TR_LIMIT as u32, 0);
		wvmcs(vcpu, VMCS_GUEST_TR_AR as u32, 0x83);
		wvmcs(vcpu, VMCS_GUEST_TR_BASE as u32, 0);

		wvmcs(vcpu, VMCS_GUEST_GDTR_LIMIT as u32, 0);
		wvmcs(vcpu, VMCS_GUEST_GDTR_BASE as u32, 0);

		wvmcs(vcpu, VMCS_GUEST_IDTR_LIMIT as u32, 0);
		wvmcs(vcpu, VMCS_GUEST_IDTR_BASE as u32, 0);

		wvmcs(vcpu, VMCS_GUEST_CR0 as u32, 0x20);
		wvmcs(vcpu, VMCS_GUEST_CR3 as u32, 0x0);
		wvmcs(vcpu, VMCS_GUEST_CR4 as u32, 0x2000);

		let code: Vec<u8> = vec![
			0xba, 0xf8, 0x03, /* mov $0x3f8, %dx */
			0x00, 0xd8, /* add %bl, %al */
			0x04, '0' as u8, /* add $'0', %al */
			0xee,      /* out %al, (%dx) */
			0xb0, '\n' as u8, /* mov $'\n', %al */
			0xee,       /* out %al, (%dx) */
			0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90,
			0x90, 0xf4, /* hlt */
		];

		let _ = (&mut mem[256..]).write(&code);

		/* set up GPRs, start at adress 0x100 */
		wreg(vcpu, x86Reg::RIP, 0x100);

		wreg(vcpu, x86Reg::RFLAGS, 0x2);
		wreg(vcpu, x86Reg::RSP, 0x0);

		/* set up args for addition */
		wreg(vcpu, x86Reg::RAX, 0x5);
		wreg(vcpu, x86Reg::RBX, 0x3);

		let mut chars = 0u8;
		loop {
			let run_res = hv_vcpu_run(vcpu);
			if run_res != 0 {
				panic!("vcpu run res: {}", run_res);
			}

			let exit_reason = rvmcs(vcpu, VMCS_RO_EXIT_REASON as u32);
			println!("exit reason: {}", exit_reason);

			let rip = rreg(vcpu, x86Reg::RIP);
			println!("RIP at {}", rip);

			if exit_reason == VMX_REASON_IRQ as u64 {
				println!("IRQ");
			} else if exit_reason == VMX_REASON_HLT as u64 {
				println!("HALT");
				break;
			} else if exit_reason == VMX_REASON_EPT_VIOLATION as u64 {
				println!("EPT VIOLATION, ignore");
			//break;
			} else if exit_reason == VMX_REASON_IO as u64 {
				println!("IO");
				if chars > 2 {
					panic!("the guest code should not return more than 2 chars on the serial port");
				}
				let qual = rvmcs(vcpu, VMCS_RO_EXIT_QUALIFIC as u32);
				if (qual >> 16) & 0xFFFF == 0x3F8 {
					let rax = rreg(vcpu, x86Reg::RAX);
					println!("RAX == {}", rax);
					println!("got char: {}", (rax as u8) as char);

					if chars == 0 {
						assert_eq!(rax, '8' as u64);
					}
					if chars == 1 {
						assert_eq!(rax, '\n' as u64);
					}
					chars += 1;

					let inst_length = rvmcs(vcpu, VMCS_RO_VMEXIT_INSTR_LEN as u32);

					wreg(vcpu, x86Reg::RIP, rip + inst_length);
				} else {
					println!("unrecognized IO port, exit");
					break;
				}
				/*
				let rax = rreg(vcpu, hv_x86_reg_t::HV_X86_RAX);
				println!("RAX == {}", rax);
				let rdx = rreg(vcpu, hv_x86_reg_t::HV_X86_RDX);
				println!("RDX == {}", rdx);
				//println!("address 0x3f8: {:?}", &mem[0x3f8..0x408]);
				println!("qual: {}", qual);
				let size = qual >> 62;
				println!("size: {}", size);
				let direction = (qual << 2) >> 63;
				println!("direction (0=out): {}, {}", direction, qual & 0x8);
				let string = (qual << 4)    >> 63;
				println!("string (1=string): {}, {}", string, qual &0x10);
				println!("port: {}", (qual >> 16) & 0xFFFF);
				*/
			}
		}

		res = hv_vcpu_destroy(vcpu);
		if res != 0 {
			panic!("vcpu destroy res: {}", res);
		}
		res = hv_vm_unmap(0, mem.len());
		if res != 0 {
			panic!("vm unmap res: {}", res);
		}

		dealloc(mem_raw, layout);
	}

	//assert!(false);
}

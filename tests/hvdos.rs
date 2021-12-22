//! Example hypervisor and 16 bits VM from https://github.com/mist64/hvdos/blob/master/hvdos.c
//! original blog post at http://www.pagetable.com/?p=764
//! guest VM code taken from https://lwn.net/Articles/658511/
extern crate xhypervisor;

use std::alloc::{alloc, dealloc, Layout};
use std::io::Write;
use std::slice;
#[cfg(target_arch = "x86_64")]
use xhypervisor::consts::vmcs::*;
#[cfg(target_arch = "x86_64")]
use xhypervisor::consts::vmx_cap::*;
#[cfg(target_arch = "x86_64")]
use xhypervisor::consts::vmx_exit::*;
use xhypervisor::ffi::*;
use xhypervisor::*;

/* desired control word constrained by hardware/hypervisor capabilities */
#[cfg(target_arch = "x86_64")]
fn cap2ctrl(cap: u64, ctrl: u64) -> u64 {
	(ctrl | (cap & 0xffffffff)) & (cap >> 32)
}

#[cfg(target_arch = "x86_64")]
#[test]
fn vm_create() {
	unsafe {
		create_vm().unwrap();

		let mut vmx_cap_pinbased: u64 = 0;
		let mut vmx_cap_procbased: u64 = 0;
		let mut vmx_cap_procbased2: u64 = 0;
		let mut vmx_cap_entry: u64 = 0;

		let mut res = hv_vmx_read_capability(VMXCap::PINBASED, &mut vmx_cap_pinbased);
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

		println!("allocating memory at {:?}", mem_raw);
		//map the vec at address 0
		let mem = slice::from_raw_parts_mut(mem_raw, capacity);
		map_mem(mem, 0, MemPerm::ExecAndWrite).unwrap();

		let vcpu = VirtualCpu::new().unwrap();

		/* set VMCS control fields */
		vcpu.write_vmcs(VMCS_CTRL_PIN_BASED, cap2ctrl(vmx_cap_pinbased, 0))
			.unwrap();
		vcpu.write_vmcs(
			VMCS_CTRL_CPU_BASED,
			cap2ctrl(
				vmx_cap_procbased,
				CPU_BASED_HLT | CPU_BASED_CR8_LOAD | CPU_BASED_CR8_STORE,
			),
		)
		.unwrap();
		vcpu.write_vmcs(VMCS_CTRL_CPU_BASED2, cap2ctrl(vmx_cap_procbased2, 0))
			.unwrap();
		vcpu.write_vmcs(VMCS_CTRL_VMENTRY_CONTROLS, cap2ctrl(vmx_cap_entry, 0))
			.unwrap();
		vcpu.write_vmcs(VMCS_CTRL_EXC_BITMAP, 0xffffffff).unwrap();
		vcpu.write_vmcs(VMCS_CTRL_CR0_MASK, 0x60000000).unwrap();
		vcpu.write_vmcs(VMCS_CTRL_CR0_SHADOW, 0).unwrap();
		vcpu.write_vmcs(VMCS_CTRL_CR4_MASK, 0).unwrap();
		vcpu.write_vmcs(VMCS_CTRL_CR4_SHADOW, 0).unwrap();
		/* set VMCS guest state fields */
		vcpu.write_vmcs(VMCS_GUEST_CS, 0).unwrap();
		vcpu.write_vmcs(VMCS_GUEST_CS_LIMIT, 0xffff).unwrap();
		vcpu.write_vmcs(VMCS_GUEST_CS_AR, 0x9b).unwrap();
		vcpu.write_vmcs(VMCS_GUEST_CS_BASE, 0).unwrap();

		vcpu.write_vmcs(VMCS_GUEST_DS, 0).unwrap();
		vcpu.write_vmcs(VMCS_GUEST_DS_LIMIT, 0xffff).unwrap();
		vcpu.write_vmcs(VMCS_GUEST_DS_AR, 0x93).unwrap();
		vcpu.write_vmcs(VMCS_GUEST_DS_BASE, 0).unwrap();

		vcpu.write_vmcs(VMCS_GUEST_ES, 0).unwrap();
		vcpu.write_vmcs(VMCS_GUEST_ES_LIMIT, 0xffff).unwrap();
		vcpu.write_vmcs(VMCS_GUEST_ES_AR, 0x93).unwrap();
		vcpu.write_vmcs(VMCS_GUEST_ES_BASE, 0).unwrap();

		vcpu.write_vmcs(VMCS_GUEST_FS, 0).unwrap();
		vcpu.write_vmcs(VMCS_GUEST_FS_LIMIT, 0xffff).unwrap();
		vcpu.write_vmcs(VMCS_GUEST_FS_AR, 0x93).unwrap();
		vcpu.write_vmcs(VMCS_GUEST_FS_BASE, 0).unwrap();

		vcpu.write_vmcs(VMCS_GUEST_GS, 0).unwrap();
		vcpu.write_vmcs(VMCS_GUEST_GS_LIMIT, 0xffff).unwrap();
		vcpu.write_vmcs(VMCS_GUEST_GS_AR, 0x93).unwrap();
		vcpu.write_vmcs(VMCS_GUEST_GS_BASE, 0).unwrap();

		vcpu.write_vmcs(VMCS_GUEST_SS, 0).unwrap();
		vcpu.write_vmcs(VMCS_GUEST_SS_LIMIT, 0xffff).unwrap();
		vcpu.write_vmcs(VMCS_GUEST_SS_AR, 0x93).unwrap();
		vcpu.write_vmcs(VMCS_GUEST_SS_BASE, 0).unwrap();

		vcpu.write_vmcs(VMCS_GUEST_LDTR, 0).unwrap();
		vcpu.write_vmcs(VMCS_GUEST_LDTR_LIMIT, 0).unwrap();
		vcpu.write_vmcs(VMCS_GUEST_LDTR_AR, 0x10000).unwrap();
		vcpu.write_vmcs(VMCS_GUEST_LDTR_BASE, 0).unwrap();

		vcpu.write_vmcs(VMCS_GUEST_TR, 0).unwrap();
		vcpu.write_vmcs(VMCS_GUEST_TR_LIMIT, 0).unwrap();
		vcpu.write_vmcs(VMCS_GUEST_TR_AR, 0x83).unwrap();
		vcpu.write_vmcs(VMCS_GUEST_TR_BASE, 0).unwrap();

		vcpu.write_vmcs(VMCS_GUEST_GDTR_LIMIT, 0).unwrap();
		vcpu.write_vmcs(VMCS_GUEST_GDTR_BASE, 0).unwrap();

		vcpu.write_vmcs(VMCS_GUEST_IDTR_LIMIT, 0).unwrap();
		vcpu.write_vmcs(VMCS_GUEST_IDTR_BASE, 0).unwrap();

		vcpu.write_vmcs(VMCS_GUEST_CR0, 0x20).unwrap();
		vcpu.write_vmcs(VMCS_GUEST_CR3, 0x0).unwrap();
		vcpu.write_vmcs(VMCS_GUEST_CR4, 0x2000).unwrap();

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
		vcpu.write_register(&x86Reg::RIP, 0x100).unwrap();

		vcpu.write_register(&x86Reg::RFLAGS, 0x2).unwrap();
		vcpu.write_register(&x86Reg::RSP, 0x0).unwrap();

		/* set up args for addition */
		vcpu.write_register(&x86Reg::RAX, 0x5).unwrap();
		vcpu.write_register(&x86Reg::RBX, 0x3).unwrap();

		let mut chars = 0u8;
		loop {
			vcpu.run().unwrap();
			let exit_reason = vcpu.read_vmcs(VMCS_RO_EXIT_REASON).unwrap() & 0xffff;
			println!("exit reason: {}", exit_reason);

			let rip = vcpu.read_register(&x86Reg::RIP).unwrap();
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
				let qual = vcpu.read_vmcs(VMCS_RO_EXIT_QUALIFIC).unwrap();
				if (qual >> 16) & 0xFFFF == 0x3F8 {
					let rax = vcpu.read_register(&x86Reg::RAX).unwrap();
					println!("RAX == {}", rax);
					println!("got char: {}", (rax as u8) as char);

					if chars == 0 {
						assert_eq!(rax, '8' as u64);
					}
					if chars == 1 {
						assert_eq!(rax, '\n' as u64);
					}
					chars += 1;

					let inst_length = vcpu.read_vmcs(VMCS_RO_VMEXIT_INSTR_LEN).unwrap();

					vcpu.write_register(&x86Reg::RIP, rip + inst_length)
						.unwrap();
				} else {
					println!("unrecognized IO port, exit");
					break;
				}

				/*let rax = vcpu.read_register(&x86Reg::RAX).unwrap();
				println!("RAX == 0x{:x}", rax);
				let rdx = vcpu.read_register(&x86Reg::RDX).unwrap();
				println!("RDX == 0x{:x}", rdx);
				println!("address 0x3f8: {:?}", &mem[0x3f8..0x408]);
				println!("qual: {}", qual);
				let size = qual >> 62;
				println!("size: {}", size);
				let direction = (qual << 2) >> 63;
				println!("direction (0=out): {}, {}", direction, qual & 0x8);
				let string = (qual << 4)    >> 63;
				println!("string (1=string): {}, {}", string, qual &0x10);
				println!("port: {}", (qual >> 16) & 0xFFFF);*/
			}
		}

		drop(vcpu);
		unmap_mem(0, mem.len()).unwrap();

		dealloc(mem_raw, layout);
	}
}

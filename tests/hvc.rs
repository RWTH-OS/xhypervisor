//! Example hypervisor and 16 bits VM from https://github.com/mist64/hvdos/blob/master/hvdos.c
//! original blog post at http://www.pagetable.com/?p=764
//! guest VM code taken from https://lwn.net/Articles/658511/
extern crate xhypervisor;

use std::alloc::{alloc, dealloc, Layout};
use std::slice;
#[cfg(target_arch = "x86_64")]
use xhypervisor::consts::vmcs::*;
#[cfg(target_arch = "x86_64")]
use xhypervisor::consts::vmx_cap::*;
#[cfg(target_arch = "x86_64")]
use xhypervisor::consts::vmx_exit::*;
use xhypervisor::ffi::*;
use xhypervisor::*;

// Example is derived from https://github.com/Thog/ahv
#[cfg(target_arch = "aarch64")]
#[test]
fn vm_create() {
	unsafe {
		let el1_user_payload = [
			0x40, 0x00, 0x80, 0xD2, // mov x0, #2
			0x02, 0x00, 0x00, 0xD4, // hvc #0
		];
		const EL1_USER_PAYLOAD_ADDRESS: hv_ipa_t = 0x20000;

		create_vm().unwrap();

		let capacity: usize = 8 * 0x10000;
		let layout: Layout = Layout::from_size_align(capacity, 4096).unwrap();
		let mem_raw = alloc(layout);

		println!("allocating memory at {:?}", mem_raw);
		//map the vec at address 0
		let mem = slice::from_raw_parts_mut(mem_raw, capacity);
		mem[EL1_USER_PAYLOAD_ADDRESS as usize..].clone_from_slice(&el1_user_payload);
		map_mem(mem, 0, MemPerm::ExecAndWrite).unwrap();

		let vcpu = VirtualCpu::new().unwrap();

		vcpu.write_register(Register::CPSR, 0x3c4).unwrap();
		vcpu.write_register(Register::PC, EL1_USER_PAYLOAD_ADDRESS)
			.unwrap();

		loop {
			vcpu.run().unwrap();
			let reason = vcpu.exit_reason();

			match reason {
				VirtualCpuExitReason::Exception { exception } => {
					let ec = (exception.syndrome >> 26) & 0x3f;

					if ec == 0x16 {
						println!(
							"HVC executed! x0 is {}",
							vcpu.read_register(Register::X0).unwrap()
						);
						break;
					} else {
						println!("Unknown exception class 0x{:x}", ec);
						break;
					}
				}
				reason => {
					println!("Unexpected exit! Reason: {:?}", reason);
					break;
				}
			}
		}

		drop(vcpu);
		unmap_mem(0, mem.len()).unwrap();

		dealloc(mem_raw, layout);
	}
}

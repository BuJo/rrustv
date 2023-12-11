use std::fs;

use vm_fdt::{Error, FdtWriter};

use crate::dynbus::DynBus;

pub fn load(x: &str) -> Vec<u8> {
    fs::read(format!("data/{x}.dtb")).expect("no device tree data")
}

pub fn generate(bus: &DynBus) -> Result<Vec<u8>, Error> {
    let mut fdt = FdtWriter::new()?;

    let root_node = fdt.begin_node("root")?;
    fdt.property_string("model", "BuJo,rriscv")?;
    fdt.property_string("compatible", "riscv-virtio")?;
    fdt.property_u32("#address-cells", 0x1)?;
    fdt.property_u32("#size-cells", 0x1)?;

    let chosen_node = fdt.begin_node("chosen")?;
    fdt.property_string(
        "bootargs",
        "root=/dev/vda ro earlycon=uart8250,mmio,0x10000000,115200n8 console=ttyS0",
    )?;
    fdt.end_node(chosen_node)?;

    bus.devices(|dm| {
        let range = dm.0.clone();
        let ino = &dm.1;
        let name = "memory";
        let t = "memory";

        let node = fdt.begin_node(name).unwrap();
        fdt.property_string("device_type", "memory").unwrap();
        fdt.property_array_u64("reg", &vec![range.start as u64, range.end as u64]).unwrap();
        fdt.end_node(node).unwrap();
    });

    fdt.end_node(root_node)?;

    println!("DT: {:?}", fdt);

    fdt.finish()
}

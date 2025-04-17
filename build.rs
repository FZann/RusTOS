use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();
    let host_triple = env::var("HOST").unwrap();

    println!("cargo:rustc-check-cfg=cfg(has_fpu)");
    println!("cargo:rustc-check-cfg=cfg(native)");

    println!("cargo:rustc-check-cfg=cfg(armv6m)");
    println!("cargo:rustc-check-cfg=cfg(armv7em)");
    println!("cargo:rustc-check-cfg=cfg(armv7m)");
    println!("cargo:rustc-check-cfg=cfg(armv8m)");
    println!("cargo:rustc-check-cfg=cfg(armv8m_base)");
    println!("cargo:rustc-check-cfg=cfg(armv8m_main)");
    println!("cargo:rustc-check-cfg=cfg(cortex_m)");
    
    println!("cargo:rustc-check-cfg=cfg(riscv)");
    
    println!("cargo:rustc-check-cfg=cfg(mips)");

    if host_triple == target {
        println!("cargo:rustc-cfg=native");
    }

    if target.starts_with("thumbv6m-") {
        println!("cargo:rustc-cfg=cortex_m");
        println!("cargo:rustc-cfg=armv6m");
    } else if target.starts_with("thumbv7m-") {
        println!("cargo:rustc-cfg=cortex_m");
        println!("cargo:rustc-cfg=armv7m");
    } else if target.starts_with("thumbv7em-") {
        println!("cargo:rustc-cfg=cortex_m");
        println!("cargo:rustc-cfg=armv7m");
        println!("cargo:rustc-cfg=armv7em");
    } else if target.starts_with("thumbv8m.base") {
        println!("cargo:rustc-cfg=cortex_m");
        println!("cargo:rustc-cfg=armv8m");
        println!("cargo:rustc-cfg=armv8m_base");
    } else if target.starts_with("thumbv8m.main") {
        println!("cargo:rustc-cfg=cortex_m");
        println!("cargo:rustc-cfg=armv8m");
        println!("cargo:rustc-cfg=armv8m_main");
    } else if target.starts_with("riscv32i") {
        println!("cargo:rustc-cfg=riscv");
    }

    if target.ends_with("-eabihf") {
        println!("cargo:rustc-cfg=has_fpu");
    }
}
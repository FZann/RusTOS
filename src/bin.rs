#![no_std]
#![no_main]

use RusTOS::panic_handler;


#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn OSEntry() -> ! {

    

    loop {
    }
}


#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn OSFault() -> ! {
    loop {
        
    }
}

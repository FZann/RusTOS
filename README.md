# RusTOS

Real Time OS in Rust


RusTOS is a simple RTOS that have a fixed-priority scheduler, software timers, semaphores, rendezvous, mutexes, queues and stream buffers.
It uses extensively bit vectors to handle tasks state, timers and other things.
Bit Vectors allow for fast process scheduling by using a simple CLZ on ready tasks bit vector, keeps RAM usage low, but limit tasks number to the bit vector size (32 bits word-size on Cortex-M).

## Why RusTOS 

I started RusTOS to learn how an RTOS works and to acquire more knoledge while working on it.
This is far from being a production-ready OS, but it is simple enough to be a good learning spot to someone that is curious to know how an RTOS work.

## Project's objectives

Project's objectives are to develop a good-featured RTOS that can be used with many different MCUs with a common HAL and device drivers.
As for now, the assembly part is implemented only for ARM Cortex-M targets, but I want to get RusTOS run on RISC-V as well.
Right now I'm implementing HAL code, to be able to build other components i.e.: protocol stacks.
HAL uses synchronization primitives given by RusTOS to create non-blocking calls.
Interrupts should be hidden to the user, as they should be used by RusTOS drivers.
I think drivers as separated tasks that regulate accesses to peripherals, in pure μ-kernel style.

An allocator should be implemented.

Project objectives are:
- microkernel design
- be able to run kernel on ARM, RISC-V and MIPS (this is optional)
- no idle process: cpu is put to sleep by the scheduler if there's no more to do
- dynamic memory allocation available
- software timers, to handle lightweight tasks, be they repetitive, counted, burst, or one-shot
- create an HAL that takes advantage of the underlying OS syncronization for peripheral access
- ```embedded-hal``` compatibility across all peripherals
- be able to run on multiple MCUs with complete (or quasi complete) peripheral access
- drivers implemented as separated tasks that control peripherals
- have some of the most used protocol stacks, to simplify these communications start-up
- ability to create processes with a procedural macro, with something like: 
``` 
#[process(prio = 5, stack = 512)]
fn do_something() -> ! {
  /* init code */
  loop {
  /* task code */
  }
}
```

### HW used to develop RusTOS

I have used a NucleoG431 to make kernel switch context and to blink it's LED to see if everything worked.
I have tested UART communication with a USB-UART bridge device and I was able to send and receive from Nucleo board.


#### Credits

Credits for ideas:
 - Harsark project for boolean vectors
 - RTIC project for process declaration with macros


## Support me

To help me make RusTOS grow, please consider to make a donation.

![Donate](/qrcode.jpeg?raw=true "Paypal QR Code for donation").



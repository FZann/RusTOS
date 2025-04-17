# RusTOS

Real Time OS in Rust


This is a simple RTOS that have a fixed-priority scheduler, software timers, semaphores, rendezvous, mutexes and queues.
It uses extensively bit vectors to handle tasks state, timers and other things.
This allows for fast process scheduling by using a simple CLZ on ready tasks bit vector, keeps RAM usage low, 
but limit tasks number to the bit vector size (32 bits word-size on Cortex-M).

My goal is to develop a good-featured RTOS that can be used with many different MCUs, but with a common API.
As for now, the assembly part is implemented only for ARM Cortex-M targets, but I want to get RusTOS run on RISC-V as well.
Right now I'm implementing HAL code, to be able to build other components i.e.: protocol stacks.

HAL uses synchronization primitives given by RusTOS to create non-blocking calls.
Interrupts should be hidden to the user, as they're used by RusTOS; callbacks should be registered and fired when an ISR runs.

An allocator should be implemented.

A question remains open: is a good idea to create an async executor to handle I/O ops, insted of using a whole RTOS task?

Project objectives are:
- microkernel design, but splitted into User-Space and Kernel Space (device drivers tasks? // single async executor?)
- HW access through use of SysCalls, that pass messages to kernel device drivers tasks // async executor
- be able to run kernel on ARM, RISC-V and MIPS
- no idle process: cpu is put to sleep by the scheduler if there's no more to do
- dynamic memory allocation available
- software timers, to handle lightweight tasks, be they repetitive, counted, burst, or one-shot
- create an HAL that takes advantage of the underlying OS syncronization for peripheral access
- be able to run on multiple MCUs with complete (or quasi complete) peripheral access
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

Credits for ideas:
 - Harsark project for boolean vectors
 - RTIC project for process declaration with macros

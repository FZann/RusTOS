# RusTOS
Real Time OS in Rust

My goal is to build a simple RTOS, just because, and learn in the process.
I hope to be able to implement semaphores, queues and mutexes.

I want to implement RusTOS with these charateristics:
- boolean vectors for processes priority; this limit the OS to 32 tasks
- no idle process: cpu is put to sleep by the scheduler if there's no more to do
- software timers, to handle non-looping tasks
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
- interrupt handler are called with a Rust reference to interrupting peripheral. Pointer passed in assembly. I think this is feasible.
- create an HAL that takes advantage of the underlying OS syncronization for peripheral access.

Credits for ideas:
 - Harsark project for boolean vectors
 - RTIC project for process declaration with macros

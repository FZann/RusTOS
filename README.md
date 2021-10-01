# RusTOS
Real Time OS in Rust

My goal is to build a simple RTOS, just because, and learn in the process.
I hope to be able to implement semaphores, queues and mutexes.

I want to implement RusTOS with these charateristics:
- boolean vectors for processes states and priority
- no idle process: cpu is put to sleep by the scheduler if there's no more to do
- ability to create processes with a procedural macro, with something like: 
``` 
#[process(prio = 5, stack = 512)]
fn do_something() -> ! {
  /* init */
  loop {
  /* task */
  }
}
```
- interrupt handler are called with a Rust reference to interrupting peripheral. Address selected in assembly. I think this is feasible.
- create an HAL that takes advantage of the underlying OS syncronization for peripheral access.
- use of async whenever possible. Maybe implement an async executor within an OS process. If it is possible!... :)

Credits for ideas:
 - Harsark project for boolean vectors
 - RTIC project for process declaration with macros

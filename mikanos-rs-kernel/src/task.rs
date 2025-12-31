pub const TASK_TIMEOUT_INTERVAL: u64 = 10;
pub const TASK_TIMEOUT_MESSAGE: i64 = i64::MAX;

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug)]
pub struct TaskContext {
    // The same layout as the original MikanOS
    // offset 0x00
    pub cr3: u64,
    pub rip: u64,
    pub rflags: u64,
    _reserved: u64,
    // offset 0x20
    // cs is 16 bit, but push to/pop from stack as 64 bit
    pub cs: u64,
    // TODO: ds, es
    pub ss: u64,
    pub fs: u64,
    pub gs: u64,
    // offset 0x40
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    // offset 0x60
    pub rdi: u64,
    pub rsi: u64,
    pub rsp: u64,
    pub rbp: u64,
    // offset 0x80
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    // offset 0xa0
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    // offset 0xc0
    pub fxsave: [u8; 512],
}

impl TaskContext {
    pub const fn new() -> Self {
        Self {
            cr3: 0,
            rip: 0,
            rflags: 0,
            _reserved: 0,
            cs: 0,
            ss: 0,
            fs: 0,
            gs: 0,
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rdi: 0,
            rsi: 0,
            rsp: 0,
            rbp: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            fxsave: [0; 512],
        }
    }
}

pub type TaskFunc = fn();

#[derive(Clone, Copy)]
pub enum TaskDescriptor {
    Main,
    Func(TaskFunc),
}

#[derive(Debug)]
pub struct Task {
    _stack: alloc::vec::Vec<u64>,
    context: TaskContext,
}

impl Task {
    pub fn new(desc: TaskDescriptor) -> Self {
        let task_stack: alloc::vec::Vec<u64> = alloc::vec![0; 1024];
        let mut task_ctx = TaskContext::new();

        match desc {
            TaskDescriptor::Main => {}
            TaskDescriptor::Func(task) => {
                unsafe {
                    task_ctx.rip = task as u64;
                    let mut cr3: u64;
                    core::arch::asm!(
                        "mov rax, cr3",
                        out("rax") cr3,
                    );
                    task_ctx.cr3 = cr3;
                    task_ctx.rflags = 0x202; // IF=1
                    task_ctx.cs = 0x08;
                    task_ctx.ss = 0;
                    task_ctx.rsp = task_stack.as_ptr() as u64 + 8 * 1024;
                }
            }
        }
        Self {
            _stack: task_stack,
            context: task_ctx,
        }
    }
}

#[derive(Debug)]
pub struct TaskPool {
    tasks: alloc::vec::Vec<Task>,
    current_task_idx: usize,
}

impl TaskPool {
    pub fn new() -> Self {
        let mut tasks = alloc::vec::Vec::new();
        tasks.push(Task::new(TaskDescriptor::Main));
        Self {
            tasks,
            current_task_idx: 0,
        }
    }
    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }
    pub fn switch_task(&mut self) {
        let current_task_idx = self.current_task_idx;
        let next_task_idx = (self.current_task_idx + 1) % self.tasks.len();
        self.current_task_idx = next_task_idx;
        if next_task_idx == 0 {
            let (left, right) = self.tasks.split_at_mut(current_task_idx);
            switch_context(&mut left[0].context, &mut right[0].context);
        } else {
            let (left, right) = self.tasks.split_at_mut(next_task_idx);
            switch_context(&mut right[0].context, &mut left[current_task_idx].context);
        }
    }
}

static mut TASK_POOL: core::cell::OnceCell<TaskPool> = core::cell::OnceCell::new();

#[naked]
pub extern "C" fn switch_context(next_ctx: &mut TaskContext, current_ctx: &mut TaskContext) {
    unsafe {
        core::arch::naked_asm!(
            // Save context
            // General purpose registers
            "mov [rsi + 0x40], rax",
            "mov [rsi + 0x48], rbx",
            "mov [rsi + 0x50], rcx",
            "mov [rsi + 0x58], rdx",
            "mov [rsi + 0x60], rdi",
            "mov [rsi + 0x68], rsi",
            "mov [rsi + 0x78], rbp",
            "mov [rsi + 0x80], r8",
            "mov [rsi + 0x88], r9",
            "mov [rsi + 0x90], r10",
            "mov [rsi + 0x98], r11",
            "mov [rsi + 0xa0], r12",
            "mov [rsi + 0xa8], r13",
            "mov [rsi + 0xb0], r14",
            "mov [rsi + 0xb8], r15",
            // Save instruction pointer and stack pointer
            // x86_64 `call` instruction for this `switch_context()` pushes the next
            // 8-byte instruction pointer of the caller (which works in `current_ctx`).
            // Therefore we should save following values for the current_ctx:
            // - rip: [rsp]
            //   - which is pushed by the `call` instruction
            // - rsp: rsp + 8
            //   - rsp is the stack pointer of this `switch_context()`
            //   - "8" is the size of instruction pointer (pushed by the `call`)
            "mov rax, [rsp]",
            "mov [rsi + 0x08], rax", // rip
            "lea rax, [rsp + 8]",
            "mov [rsi + 0x70], rax", // rsp
            // Special registers
            "mov rax, cr3",
            "mov [rsi + 0x00], rax", // CR3
            "pushfq",
            "pop QWORD PTR [rsi + 0x10]", // RFLAGS
            // Segment registers
            "mov ax, cs",
            "mov [rsi + 0x20], rax",
            "mov ax, ss",
            "mov [rsi + 0x28], rax",
            "mov ax, fs",
            "mov [rsi + 0x30], rax",
            "mov ax, gs",
            "mov [rsi + 0x38], rax",
            // fxsave
            "fxsave [rsi + 0xc0]",
            // ---------------------------------------
            // Restore context
            // Make stack state for iret
            "push QWORD PTR [rdi + 0x28]", // SS
            "push QWORD PTR [rdi + 0x70]", // RSP
            "push QWORD PTR [rdi + 0x10]", // RFLAGS
            "push QWORD PTR [rdi + 0x20]", // CS
            "push QWORD PTR [rdi + 0x08]", // RIP
            // fxrestore
            "fxrstor [rdi + 0xc0]",
            // Special registers/Segment registers
            "mov rax, [rdi + 0x00]",
            "mov cr3, rax",
            "mov rax, [rdi + 0x30]",
            "mov fs, ax",
            "mov rax, [rdi + 0x38]",
            "mov gs, ax",
            // General purpose registers
            "mov rax, [rdi + 0x40]",
            "mov rbx, [rdi + 0x48]",
            "mov rcx, [rdi + 0x50]",
            "mov rdx, [rdi + 0x58]",
            "mov rsi, [rdi + 0x68]",
            "mov rbp, [rdi + 0x78]",
            "mov r8, [rdi + 0x80]",
            "mov r9, [rdi + 0x88]",
            "mov r10, [rdi + 0x90]",
            "mov r11, [rdi + 0x98]",
            "mov r12, [rdi + 0xa0]",
            "mov r13, [rdi + 0xa8]",
            "mov r14, [rdi + 0xb0]",
            "mov r15, [rdi + 0xb8]",
            "mov rdi, [rdi + 0x60]",
            "iretq",
        );
    }
}

#[allow(static_mut_refs)]
pub fn task_b() {
    let mut cnt = 0;
    loop {
        cnt += 1;
        if cnt % 1000000 == 0 {
            let msg = alloc::format!("(Task B) count={}\n", cnt);
            crate::serial_print!("{}", msg);
        }
    }
}

#[allow(static_mut_refs)]
pub fn task_c() {
    let mut cnt = 0;
    loop {
        cnt += 1;
        if cnt % 1000000 == 0 {
            let msg = alloc::format!("(Task C) count={}\n", cnt);
            crate::serial_print!("{}", msg);
        }
    }
}

pub fn add_task_timeout_timer(tick: u64) {
    crate::timer::add_timer(crate::timer::Timer::new(
        tick + TASK_TIMEOUT_INTERVAL,
        TASK_TIMEOUT_MESSAGE,
    ));
}

#[allow(static_mut_refs)]
pub fn initialize_task_switch() {
    unsafe {
        TASK_POOL.set(TaskPool::new()).unwrap();
    }
    let initial_tick = 0;
    add_task_timeout_timer(initial_tick)
}

#[allow(static_mut_refs)]
pub fn add_task(task: Task) {
    unsafe {
        TASK_POOL.get_mut().unwrap().add_task(task);
    }
}

#[allow(static_mut_refs)]
pub unsafe fn switch_task() {
    unsafe {
        TASK_POOL.get_mut().unwrap().switch_task();
    }
}

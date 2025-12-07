#[repr(C, align(16))]
pub struct TaskContext {
    // とりあえずみかん本と全く同じレイアウトで実装
    // わからんところはコメントに残しておく
    // offset 0x00
    cr3: u64,
    rip: u64,
    rflags: u64,
    _reserved: u64, // 必要?
    // offset 0x20
    cs: u64, // Segment register って 16 bit じゃなかったっけ
    // ds, es は保存しないの?
    ss: u64,
    fs: u64,
    gs: u64,
    // offset 0x40
    rax: u64,
    rbx: u64,
    rcx: u64,
    rdx: u64,
    // offset 0x60
    rdi: u64,
    rsi: u64,
    rsp: u64,
    rbp: u64,
    // offset 0x80
    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,
    // offset 0xa0
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,
    // offset 0xc0
    fxsave: [u8; 512],
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
            "mov [rsi + 0x20], rax",
            "mov ax, fs",
            "mov [rsi + 0x20], rax",
            "mov ax, gs",
            "mov [rsi + 0x20], rax",
            // fxsave
            "fxsave [rsi + 0xc0]",
            // ---------------------------------------
            // TODO: Restore context
            "ret",
        );
    }
}

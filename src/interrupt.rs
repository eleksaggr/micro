use x86_64::structures::idt::{ExceptionStackFrame, Idt};

lazy_static! {
    static ref IDT: Idt = {
        let mut idt = Idt::new();

        idt.breakpoint.set_handler_fn(bp_handler);
        idt.double_fault.set_handler_fn(df_handler);

        idt
    };
}

pub fn init() {
    IDT.load();
}

extern "x86-interrupt" fn bp_handler(stack: &mut ExceptionStackFrame) {
    println!("Exception: Breakpoint\n{:#?}", stack);
}

extern "x86-interrupt" fn df_handler(stack: &mut ExceptionStackFrame, _: u64) {
    println!("Exception: Double Fault\n{:#?}", stack);
    loop {}
}

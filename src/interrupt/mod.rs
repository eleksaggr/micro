use memory::MemoryController;
use spin::Once;
use util::log::{Level, Logger};
use x86_64::structures::idt::{ExceptionStackFrame, Idt};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::gdt::SegmentSelector;
use x86_64::instructions::segmentation::set_cs;
use x86_64::instructions::tables::load_tss;

mod gdt;

lazy_static! {
    static ref IDT: Idt = {
        let mut idt = Idt::new();

        idt.breakpoint.set_handler_fn(bp_handler);
        unsafe {
            idt.double_fault.set_handler_fn(df_handler).set_stack_index(DOUBLE_FAULT_IST_INDEX as u16);
        }

        idt
    };
}

static TSS: Once<TaskStateSegment> = Once::new();
static GDT: Once<gdt::GlobalDescriptorTable> = Once::new();

const DOUBLE_FAULT_IST_INDEX: usize = 0;

pub fn init(mcon: &mut MemoryController) {
    use x86_64::VirtualAddress;

    let df_stack = mcon.allocate_stack(1).expect("Could not allocate stack for double fault handler.");

    let tss = TSS.call_once(|| {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX] = VirtualAddress(df_stack.top());
        tss
    });

    let mut code_selector = SegmentSelector(0);
    let mut tss_selector = SegmentSelector(0);
    let gdt = GDT.call_once(|| {
        let mut gdt = gdt::GlobalDescriptorTable::new();
        code_selector = gdt.add(gdt::Descriptor::code());
        tss_selector = gdt.add(gdt::Descriptor::tss(&tss));
        gdt
    });
    gdt.load(); 

    unsafe {
        set_cs(code_selector);
        load_tss(tss_selector);
    }

    IDT.load();
}

extern "x86-interrupt" fn bp_handler(stack: &mut ExceptionStackFrame) {
    log!(Level::Warn, "Caught exception: Breakpoint");
    log!(Level::Warn, "Printing stack frame at point of exception:");
    log!(Level::Warn, "{:#?}", stack);
}

extern "x86-interrupt" fn df_handler(stack: &mut ExceptionStackFrame, _: u64) {
    log!(Level::Warn, "Caught exception: Double Fault");
    log!(Level::Warn, "Printing stack frame at point of exception:");
    log!(Level::Warn, "{:#?}", stack);
    loop{}
}

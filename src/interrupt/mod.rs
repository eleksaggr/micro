use memory::MemoryController;
use spin::Once;
use x86_64::structures::idt::{ExceptionStackFrame, Idt, PageFaultErrorCode};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtualAddress;

mod gdt;

pub fn init(mcon: &mut MemoryController) {}

// static TSS: Once<TaskStateSegment> = Once::new();
// static GDT: Once<gdt::GlobalDescriptorTable> = Once::new();

// const DOUBLE_FAULT_IST_INDEX: usize = 1;

// lazy_static! {
//     static ref IDT: Idt = {
//         let mut idt = Idt::new();

//         idt.breakpoint.set_handler_fn(bp_handler);
//         idt.general_protection_fault.set_handler_fn(gpf_handler);
//         idt.page_fault.set_handler_fn(pf_handler);
//             idt.double_fault.set_handler_fn(df_handler);

//         idt
//     };
// }

// pub fn init(mcon: &mut MemoryController) {
//     use x86_64::structures::gdt::SegmentSelector;
//     use x86_64::instructions::segmentation::set_cs;
//     use x86_64::instructions::tables::load_tss;

//     let df_stack = mcon.allocate_stack(1).expect("Could not allocate double fault stack.");

//     let tss = TSS.call_once(|| {
//         let mut tss = TaskStateSegment::new();
//         tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX] = VirtualAddress(df_stack.top());
//         tss
//     });

//     let mut code_selector = SegmentSelector(0);
//     let mut tss_selector = SegmentSelector(0);
//     let gdt = GDT.call_once(|| {
//         let mut gdt = gdt::GlobalDescriptorTable::new();
//         code_selector = gdt.add(gdt::Descriptor::code());
//         tss_selector = gdt.add(gdt::Descriptor::tss(&tss));
//         gdt
//     });
//     gdt.load();

//     println!("CS: {:?} | TSS: {:?}", code_selector, tss_selector);

//     unsafe {
//         set_cs(code_selector);
//         load_tss(tss_selector);
//     }

//     IDT.load();
// }

// extern "x86-interrupt" fn pf_handler(stack: &mut ExceptionStackFrame, code: PageFaultErrorCode) {
//     println!("Exception: Page Fault\n{:#?}", stack);
//     println!("Code: {:?}", code);
// }

// extern "x86-interrupt" fn bp_handler(stack: &mut ExceptionStackFrame) {
//     println!("Exception: Breakpoint\n{:#?}", stack);
// }

// extern "x86-interrupt" fn df_handler(stack: &mut ExceptionStackFrame, code: u64) {
//     println!("Exception: Double Fault\n{:#?}", stack);
// }

// extern "x86-interrupt" fn gpf_handler(stack: &mut ExceptionStackFrame, code: u64) {
//     println!("Exception: General Protection Fault\n{:#?}", stack);
//     println!("Code: {}", code);
// }

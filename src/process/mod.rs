use x86_64::{ VirtAddr, structures::paging::{ FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB } };
use x86_64::structures::gdt::SegmentSelector;

use crate::thread;

/// ユーザコード
pub const USER_CODE_START: u64 = 0x0000_1000_0000_0000;

/// ユーザスタック
pub const USER_STACK_TOP: u64 = 0x0000_2000_0000_0000;
pub const USER_STACK_PAGES: u64 = 4;

/// ユーザコードのエントリポイント
static USER_CODE: &[u8] = &[
    0x48, 0x31, 0xC0,           // xor rax, rax
    0x48, 0xFF, 0xC0,           // inc rax
    0xEB, 0xFB,                 // jmp -5
];

pub fn create_user_process(user_cs: SegmentSelector, user_ss: SegmentSelector) {
    // スレッド ID を確保
    let tid = thread::next_tid().expect("Thread table is full");

    // カーネルスタックを作成
    let stack = unsafe {
        let layout = alloc::alloc::Layout::from_size_align(thread::STACK_SIZE, 16).unwrap();
        alloc::alloc::alloc(layout)
    };
    let stack_top = stack as u64 + thread::STACK_SIZE as u64;

    let mut table = thread::THREAD_TABLE.lock();
    table[tid].tid = tid;
    table[tid].state = thread::ThreadState::Runnable;
    table[tid].kstack = stack_top;

    // コンテキストを初期化する
    let entry = jump_to_usermode;
    table[tid].context.rsp = stack_top;
    table[tid].context.rip = entry as u64;
    table[tid].context.rflags = 0x200;  // IF (Interrupt Flag) を有効化
}

pub fn map_user_pages(mapper: &mut impl Mapper<Size4KiB>, frame_allocator: &mut impl FrameAllocator<Size4KiB>) -> Result<(), &'static str> {
    // ユーザページのフラグ
    let user_flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

    // ユーザコード用ページ作成
    let code_page = Page::containing_address(VirtAddr::new(USER_CODE_START));
    let frame = frame_allocator.allocate_frame().ok_or("frame alloc failed")?;
    unsafe {
        mapper.map_to(code_page, frame, user_flags, frame_allocator)
    }.map_err(|_| "map_to failed")?.flush();

    // ユーザスタック用ページ作成
    let stack_start = USER_STACK_TOP - USER_STACK_PAGES * 4096;
    for i in 0..USER_STACK_PAGES {
        let stack_page = Page::containing_address(VirtAddr::new(stack_start + i * 4096));
        let frame = frame_allocator.allocate_frame().ok_or("ftame alloc failed")?;
        unsafe {
            mapper.map_to(stack_page, frame, user_flags, frame_allocator)
        }.map_err(|_| "map_to failed")?.flush();
    }

    Ok(())
}

pub fn copy_user_code_to_memory() {
    let dst = USER_CODE_START as *mut u8;
    unsafe {
        core::ptr::copy_nonoverlapping(
            USER_CODE.as_ptr(),
            dst,
            USER_CODE.len(),
        );
    }
}

pub unsafe fn jump_to_usermode(user_cs: SegmentSelector, user_ss: SegmentSelector) -> ! {
    let code_ptr = USER_CODE_START;
    let stack_ptr = USER_STACK_TOP;

    unsafe {
        core::arch::asm!(
            "mov ds, ax",   // ax には user_ss が入っている（後述）
            "mov es, ax",
            "push rax",     // SS
            "push {rsp}",   // RSP
            "push {rflags}",// RFLAGS
            "push {user_cs}",// CS
            "push {rip}",   // RIP
            "iretq",
            inout("ax") user_ss.0 => _,
            user_cs  = in(reg) user_cs.0 as u64,
            rsp      = in(reg) stack_ptr,
            rip      = in(reg) code_ptr,
            rflags   = in(reg) 0x202u64,
        );
    }

    loop {}
}

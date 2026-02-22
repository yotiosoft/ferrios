use x86_64::{ VirtAddr, structures::paging::{ FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB } };
use x86_64::structures::gdt::SegmentSelector;

use super::{ STACK_SIZE, THREAD_TABLE, ThreadState };

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
    let tid = next_tid().expect("Thread table is full");

    // ユーザページのフラグ
    let user_flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

    // カーネルスタックを作成
    let stack = unsafe {
        let layout = alloc::alloc::Layout::from_size_align(STACK_SIZE, 16).unwrap();
        alloc::alloc::alloc(layout)
    };
    let stack_top = stack as u64 + STACK_SIZE as u64;

    let mut table = THREAD_TABLE.lock();
    table[tid].tid = tid;
    table[tid].state = ThreadState::Runnable;
    table[tid].kstack = stack_top;

    // コンテキストを初期化する
    let entry = jump_to_usermode;
    table[tid].context.rsp = stack_top;
    table[tid].context.rip = entry as u64;
    table[tid].context.rflags = 0x200;  // IF (Interrupt Flag) を有効化
}

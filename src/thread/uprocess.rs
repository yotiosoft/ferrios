use x86_64::{ VirtAddr, structures::paging::{ FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB } };
use lazy_static::lazy_static;

use super::{ STACK_SIZE, THREAD_TABLE, ThreadState };

mod uthread;

/// ユーザコード
pub const USER_CODE_START: u64 = 0x0000_1000_0000_0000;

/// ユーザスタック
pub const USER_STACK_TOP: u64 = 0x0000_2000_0000_0000;
pub const USER_STACK_PAGES: u64 = 4;

lazy_static! {
    /// Process ID
    static ref NEXT_PID: spin::Mutex<usize> = spin::Mutex::new(0);
}

pub fn create_user_process(code: &[u8], mapper: &mut impl Mapper<Size4KiB>, frame_allocator: &mut impl FrameAllocator<Size4KiB>) -> Result<(), &'static str> {
    // ユーザページのフラグ
    let user_flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

    // コードページ用領域を用意
    let code_page = Page::containing_address(VirtAddr::new(USER_CODE_START));
    let code_frame = frame_allocator.allocate_frame().expect("frame alloc failed");
    
    // コードページにユーザコードをコピー
    unsafe {
        mapper.map_to(code_page, code_frame, user_flags, frame_allocator).map_err(|_| "code map_to failed")?.flush();
        core::ptr::copy_nonoverlapping(code.as_ptr(), USER_CODE_START as *mut u8, code.len());
    }

    // ユーザスタック用領域を用意
    let stack_start = USER_STACK_TOP - USER_STACK_PAGES * 4096;
    for i in 0..USER_STACK_PAGES {
        let page = Page::containing_address(VirtAddr::new(stack_start + i * 4096));
        let frame = frame_allocator.allocate_frame().ok_or("frame alloc failed")?;
        unsafe {
            mapper.map_to(page, frame, user_flags, frame_allocator).map_err(|_| "stack map_to failed")?.flush();
        }
    }

    // カーネルスタックを作成
    let kstack = unsafe {
        let layout = alloc::alloc::Layout::from_size_align(STACK_SIZE, 16).unwrap();
        alloc::alloc::alloc(layout)
    };
    let kstack_top = kstack as u64 + STACK_SIZE as u64;

    // Process ID を決定
    let pid = 0;

    // init thread を作成
    let thread = uthread::create_user_thread(pid, kstack_top);

    // Thread Table に追加
    let tid = thread.tid;
    let mut table = THREAD_TABLE.lock();
    table[tid] = thread;

    Ok(())
}

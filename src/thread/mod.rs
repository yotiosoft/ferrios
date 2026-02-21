use spin::Mutex;
use lazy_static::lazy_static;

pub mod kthread;
pub mod uthread;

extern crate alloc;

use crate::scheduler::context::Context;

static STACK_SIZE: usize = 4096 * 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadState {
    Unused,
    Embryo,
    Sleeping,
    Runnable,
    Running,
    Zombie,
}

/// Process Control Block
#[derive(Debug, Clone, Copy)]
pub struct Thread {
    pub tid: usize,             // Thread ID
    pub state: ThreadState,    // プロセスの状態
    pub context: Context,       // プロセスのコンテキスト
    pub kstack: u64,            // このプロセス用のカーネルスタック
}

impl Thread {
    pub fn new() -> Self {
        Thread {
            tid: 0,
            state: ThreadState::Unused,
            context: Context::new(),
            kstack: 0,
        }
    }
}

// プロセス数
pub const NPROC: usize = 64;

lazy_static! {
    pub static ref THREAD_TABLE: Mutex<[Thread; NPROC]> = {
        Mutex::new([Thread::new(); NPROC])
    };
}

/// Thread ID 決定
pub fn next_tid() -> Option<usize> {
    let table = THREAD_TABLE.lock();
    for i in 0..NPROC-1 {
        if table[i].state == ThreadState::Unused {
            return Some(i);
        }
    }
    None
}

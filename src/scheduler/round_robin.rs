use super::{ Thread, ThreadState, THREAD_TABLE, NTHREAD, cpu::CPU, SCHEDULER_STARTED };
use super::context::{ Context, switch_context };

pub struct RoundRobin;

impl super::Scheduler for RoundRobin {
    /// スケジューラ
    fn scheduler(&self) -> ! {
        unsafe {
            if SCHEDULER_STARTED {
                panic!("Scheduler already started");
            }
            SCHEDULER_STARTED = true;
        }

        loop {
            let mut table = THREAD_TABLE.lock();
            let mut cpu = CPU.lock();
            
            // 次に実行するスレッドの決定
            let next_tid = {
                find_next_runnable_thread(&table, cpu.current_tid)
            };

            match next_tid {
                None => {
                    x86_64::instructions::interrupts::enable_and_hlt();
                    drop(cpu);
                    drop(table);
                    continue;
                }
                Some(next_tid) => {
                    let (old_context, new_context) = {
                        // スレッド状態を更新
                        table[next_tid].state = ThreadState::Running;
                        if let Some(current_tid) = cpu.current_tid {
                            if table[current_tid].state == ThreadState::Running {
                                table[current_tid].state = ThreadState::Runnable;
                            }
                        }
                        
                        // CPU で実行中のスレッド ID を更新
                        cpu.current_tid = Some(next_tid);
                        
                        let old_context = &mut cpu.scheduler as *mut Context;
                        let new_context = &table[next_tid].context as *const Context;

                        // CPU の syscall_rsp をスレッドの kstack に変更
                        cpu.kernel_syscall_rsp = table[next_tid].kstack;

                        drop(cpu);
                        drop(table);

                        (old_context, new_context)
                    };

                    unsafe {
                        x86_64::instructions::interrupts::enable();
                        //crate::println!("switch");
                        switch_context(old_context, new_context);
                    }
                }
            }
        }
    }

    /// スレッドからスケジューラに戻る
    fn on_yield(&self) {
        x86_64::instructions::interrupts::disable();

        let mut table = THREAD_TABLE.lock();
        let cpu = CPU.lock();

        let current_tid = cpu.current_tid;
        if current_tid.is_none() {
            x86_64::instructions::interrupts::enable();
            return;
        }
        let current_tid = current_tid.unwrap();
        if table[current_tid].state != ThreadState::Running {
            panic!("CPU has current_tid but the thread is not Running");
        }

        let (old_context, new_context) = {
            // Runnable に変更
            table[current_tid].state = ThreadState::Runnable;

            // スケジューラへコンテキストスイッチ
            let old_context = &mut table[current_tid].context as *mut Context;
            let new_context = &cpu.scheduler as *const Context;

            drop(cpu);
            drop(table);

            (old_context, new_context)
        };
        unsafe {
            x86_64::instructions::interrupts::enable();
            switch_context(old_context, new_context);
        }
    }
}

fn find_next_runnable_thread(table: &[Thread; NTHREAD], current_tid: Option<usize>) -> Option<usize> {
    let current_tid = current_tid.unwrap_or(0);
    for i in 1..NTHREAD+1 {
        let tid = (current_tid + i) % NTHREAD;
        if table[tid].state == ThreadState::Runnable {
            return Some(tid);
        }
    }
    None
}

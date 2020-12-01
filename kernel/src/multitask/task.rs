// The MIT License (MIT)
//
// Copyright (c) 2019 Philipp Oppermann
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

use alloc::{boxed::Box, collections::BTreeMap, sync::Arc, task::Wake};
use core::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    task::{Context, Poll, Waker},
};
use crossbeam_queue::ArrayQueue;

pub struct Collection {
    tasks: BTreeMap<Id, Task>,
    woken_task_ids: Arc<ArrayQueue<Id>>,
}
impl Collection {
    pub fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
            woken_task_ids: Arc::new(ArrayQueue::new(100)),
        }
    }

    pub fn add_task_as_woken(&mut self, task: Task) {
        let id = task.id();
        self.push_task(task);
        self.push_woken_task_id(id);
    }

    pub fn add_task_as_sleep(&mut self, task: Task) {
        self.push_task(task);
    }

    fn push_task(&mut self, task: Task) {
        let id = task.id();
        if self.tasks.insert(id, task).is_some() {
            panic!("Task ID confliction.");
        }
    }

    fn push_woken_task_id(&mut self, id: Id) {
        self.woken_task_ids
            .push(id)
            .expect("Woken task id queue is full.");
    }

    pub fn woken_task_exists(&self) -> bool {
        !self.woken_task_ids.is_empty()
    }

    pub fn pop_woken_task_id(&mut self) -> Option<Id> {
        self.woken_task_ids.pop()
    }

    pub fn remove_task(&mut self, id: Id) -> Option<Task> {
        self.tasks.remove(&id)
    }

    pub fn create_waker(&mut self, id: Id) -> Waker {
        Waker::from(Arc::new(TaskWaker::new(id, self.woken_task_ids.clone())))
    }
}

// task::Waker conflicts with alloc::task::Waker.
#[allow(clippy::module_name_repetitions)]
pub struct TaskWaker {
    id: Id,
    woken_task_ids: Arc<ArrayQueue<Id>>,
}

impl TaskWaker {
    pub fn new(id: Id, woken_task_ids: Arc<ArrayQueue<Id>>) -> Self {
        Self { id, woken_task_ids }
    }

    fn wake_task(&self) {
        self.woken_task_ids
            .push(self.id)
            .expect("task_queue is full");
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task()
    }
}

#[derive(PartialOrd, PartialEq, Ord, Eq, Copy, Clone, Debug)]
pub struct Id(u64);

impl Id {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        Id(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub struct Task {
    id: Id,
    future: Pin<Box<dyn Future<Output = ()>>>,
    polling: bool,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Self {
        Self {
            id: Id::new(),
            future: Box::pin(future),
            polling: false,
        }
    }

    pub fn new_poll(future: impl Future<Output = ()> + 'static) -> Self {
        Self {
            id: Id::new(),
            future: Box::pin(future),
            polling: true,
        }
    }

    pub fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }

    pub(super) fn id(&self) -> Id {
        self.id
    }
}

use libpulse_binding::operation::{Operation, State};
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::Poll;
use glib::{MainContext, PRIORITY_DEFAULT_IDLE};

pub(crate) trait OperationExt {
  fn get_state(&self) -> State;
}

impl<T: ?Sized> OperationExt for Operation<T> {
  fn get_state(&self) -> State {
    self.get_state()
  }
}

pub(crate) struct Value<T> {
  pub(crate) error: bool,
  pub(crate) value: Option<T>,
}

impl<T> Value<T> {
  pub(crate) fn new(value: Option<T>) -> Value<T> {
    Value {
      error: false,
      value,
    }
  }
}

pub struct OperationFuture<T> {
  pub(crate) result: Rc<RefCell<Value<T>>>,
  pub(crate) operation: Rc<dyn OperationExt>,
}

impl<T> Future for OperationFuture<T> {
  type Output = Result<T, ()>;

  fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
    let c = MainContext::default();
    let waker = cx.waker().clone();
    c.invoke_local_with_priority(PRIORITY_DEFAULT_IDLE, move || {
      waker.wake_by_ref();
    });

    match self.operation.get_state() {
      State::Running => Poll::Pending,
      State::Done => {
        if self.as_mut().result.borrow().error {
          Poll::Ready(Err(()))
        } else {
          Poll::Ready(Ok(self.as_mut().result.borrow_mut().value.take().unwrap()))
        }
      }
      State::Cancelled => Poll::Ready(Err(())),
    }
  }
}
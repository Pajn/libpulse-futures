use crate::clone;
use crate::introspector::Introspector;
use crate::operation::Value;
use futures::stream::Stream;
pub use libpulse_binding::context;
use libpulse_binding::context::State;
pub use libpulse_binding::def::SpawnApi;
pub use libpulse_binding::error::PAErr;
use libpulse_binding::mainloop::standard::{IterateResult, Mainloop};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::rc::Rc;
use std::task::Poll;

pub use libpulse_binding::context::subscribe::{Facility, InterestMaskSet, Operation};
pub use libpulse_binding::context::{flags, FlagSet};
pub use libpulse_binding::proplist::Proplist;

pub struct Context {
  mainloop: Rc<RefCell<Mainloop>>,
  context: Rc<RefCell<context::Context>>,
}

impl Context {
  /// Instantiates a new connection context with an abstract
  /// mainloop API and an application name, and specify the initial
  /// client property list.
  pub fn new_with_proplist(name: &str, proplist: &Proplist) -> Context {
    let mainloop = Rc::new(RefCell::new(
      Mainloop::new().expect("Failed to create mainloop"),
    ));

    let context = Rc::new(RefCell::new(
      context::Context::new_with_proplist(mainloop.borrow().deref(), name, proplist)
        .expect("Failed to create new context"),
    ));

    Context { mainloop, context }
  }

  /// Connects the context to the specified server.
  ///
  /// If server is None, connect to the default server.
  /// If flags doesn’t have flags::NOAUTOSPAWN set and no specific
  /// server is specified or accessible, a new daemon is spawned.
  /// If api is not None, the functions specified in the structure
  /// are used when forking a new child process.
  pub fn connect(
    &mut self,
    server: Option<&str>,
    flags: FlagSet,
    api: Option<&SpawnApi>,
  ) -> ContextFuture {
    self
      .context
      .borrow_mut()
      .connect(server, flags, api)
      .expect("Failed to connect context");

    ContextFuture {
      mainloop: self.mainloop.clone(),
      context: self.context.clone(),
    }
  }

  /// Terminates the context connection immediately.
  pub fn disconnect(&mut self) {
    self.context.borrow_mut().disconnect();
  }

  /// Gets an introspection object linked to the current context,
  /// giving access to introspection routines.
  pub fn introspect(&self) -> Introspector {
    Introspector {
      mainloop: self.mainloop.clone(),
      introspector: self.context.borrow().introspect(),
    }
  }

  /// Enables event notification.
  ///
  /// The mask parameter is used to specify which facilities you are
  /// interested in being modified about.
  ///
  /// The tuple has three values. The first two are the facility and
  /// operation components of the event type respectively (the
  /// underlying C API provides this information combined into a single
  /// integer, here we extract the two component parts for you); these
  /// are wrapped in Option wrappers should the given values ever not
  /// map to the enum variants, but it’s probably safe to always just
  /// unwrap() them). The third parameter is an associated index value.
  ///
  /// Panics if the underlying C function returns a null pointer.
  pub fn subscribe(&mut self, mask: InterestMaskSet) -> Subscription {
    let events = Rc::new(RefCell::new(Value::new(Some(VecDeque::new()))));

    let callback = Box::new(clone!(events => move |facility, operation, index| {
      events.borrow_mut().value.as_mut().unwrap().push_back((facility, operation, index));
    }));
    self
      .context
      .borrow_mut()
      .set_subscribe_callback(Some(callback));
    self.context.borrow_mut().subscribe(
      mask,
      clone!(events => move |success| {
        if !success {
          events.borrow_mut().error = true;
        }
      }),
    );

    Subscription {
      error_returned: false,
      events,
      mainloop: self.mainloop.clone(),
    }
  }
}

impl Drop for Context {
  fn drop(&mut self) {
    self.disconnect();
  }
}

pub struct ContextFuture {
  mainloop: Rc<RefCell<Mainloop>>,
  context: Rc<RefCell<context::Context>>,
}

impl Future for ContextFuture {
  type Output = Result<(), ()>;

  fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
    cx.waker().wake_by_ref();

    match self.mainloop.borrow_mut().iterate(false) {
      IterateResult::Quit(_) | IterateResult::Err(_) => return Poll::Ready(Err(())),
      IterateResult::Success(_) => {}
    }

    match self.context.borrow().get_state() {
      State::Ready => Poll::Ready(Ok(())),
      State::Failed | State::Terminated => Poll::Ready(Err(())),
      _ => Poll::Pending,
    }
  }
}

pub struct Subscription {
  error_returned: bool,
  events: Rc<RefCell<Value<VecDeque<(Option<Facility>, Option<Operation>, u32)>>>>,
  mainloop: Rc<RefCell<Mainloop>>,
}

impl Stream for Subscription {
  type Item = Result<(Option<Facility>, Option<Operation>, u32), ()>;

  fn poll_next(mut self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Option<Self::Item>> {
    cx.waker().wake_by_ref();

    if self.error_returned {
      return Poll::Ready(None);
    }

    let result = self.mainloop.borrow_mut().iterate(false);
    match result {
      IterateResult::Quit(_) | IterateResult::Err(_) => {
        self.error_returned = true;
        return Poll::Ready(Some(Err(())));
      }
      IterateResult::Success(_) => {}
    }

    if self.events.borrow().error {
      self.error_returned = true;
      return Poll::Ready(Some(Err(())));
    }

    match self.events.borrow_mut().value.as_mut().unwrap().pop_front() {
      Some(event) => Poll::Ready(Some(Ok(event))),
      _ => Poll::Pending,
    }
  }
}

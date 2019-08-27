use crate::introspector::Introspector;
pub use libpulse_binding::context;
use libpulse_binding::context::State;
pub use libpulse_binding::def::SpawnApi;
pub use libpulse_binding::error::PAErr;
use libpulse_binding::mainloop::standard::{IterateResult, Mainloop};
use std::cell::RefCell;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::rc::Rc;
use std::task::Poll;

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

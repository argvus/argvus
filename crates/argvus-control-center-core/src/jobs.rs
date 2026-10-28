//! Asynchronous work and cooperative cancellation.
//!
//! Jobs are deliberately simple: one thread executes the operation, one channel
//! delivers exactly one result, and the token enables cooperative cancellation.
//! This model keeps slow probes out of the render loop; the
//! screen only reads the state available through `try_state`.
use std::sync::{
  Arc,
  atomic::{AtomicBool, Ordering},
  mpsc::{self, Receiver, TryRecvError},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JobId(u64);

#[derive(Debug, Clone)]
pub struct CancellationToken(Arc<AtomicBool>);

thread_local! {
  static CURRENT_CANCELLATION: std::cell::RefCell<Option<CancellationToken>> = const { std::cell::RefCell::new(None) };
}

pub(crate) fn current_cancelled() -> bool {
  CURRENT_CANCELLATION.with(|slot| {
    slot
      .borrow()
      .as_ref()
      .is_some_and(CancellationToken::is_cancelled)
  })
}

impl CancellationToken {
  pub fn is_cancelled(&self) -> bool {
    self.0.load(Ordering::Acquire)
  }
}

#[derive(Debug)]
pub enum JobState<T> {
  /// The producer may still publish the result.
  Running,
  /// The producer finished; errors are carried as the job result as well.
  Finished(Result<T, String>),
}

pub struct JobHandle<T> {
  /// Monotonic identifier used to correlate refreshes and results.
  id: JobId,
  /// The same flag is observed by the worker and set by the UI.
  cancel: Arc<AtomicBool>,
  /// Single-delivery channel that avoids domain-specific shared state.
  receiver: Receiver<Result<T, String>>,
}

impl<T> JobHandle<T> {
  pub const fn id(&self) -> JobId {
    self.id
  }
  pub fn cancel(&self) {
    self.cancel.store(true, Ordering::Release);
  }
  pub fn try_state(&self) -> JobState<T> {
    match self.receiver.try_recv() {
      Ok(result) => JobState::Finished(result),
      Err(TryRecvError::Empty) => JobState::Running,
      Err(TryRecvError::Disconnected) => {
        JobState::Finished(Err("background job disconnected".into()))
      }
    }
  }
}

#[derive(Debug, Default)]
pub struct JobManager {
  next_id: std::sync::atomic::AtomicU64,
}

impl JobManager {
  pub fn spawn<T, F>(&self, task: F) -> JobHandle<T>
  where
    T: Send + 'static,
    F: FnOnce(CancellationToken) -> Result<T, String> + Send + 'static,
  {
    let id = JobId(self.next_id.fetch_add(1, Ordering::Relaxed));
    let cancel = Arc::new(AtomicBool::new(false));
    let token = CancellationToken(Arc::clone(&cancel));
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
      CURRENT_CANCELLATION.with(|slot| *slot.borrow_mut() = Some(token.clone()));
      let _ = sender.send(task(token));
      CURRENT_CANCELLATION.with(|slot| *slot.borrow_mut() = None);
    });
    JobHandle {
      id,
      cancel,
      receiver,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::time::Duration;

  #[test]
  fn job_runs_off_thread_and_can_be_cancelled() {
    let manager = JobManager::default();
    let handle = manager.spawn(|token| {
      std::thread::sleep(Duration::from_millis(20));
      Ok(token.is_cancelled())
    });
    handle.cancel();
    std::thread::sleep(Duration::from_millis(40));
    assert!(matches!(handle.try_state(), JobState::Finished(Ok(true))));
  }
}

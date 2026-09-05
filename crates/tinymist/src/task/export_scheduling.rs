use futures::Future;
use parking_lot::Mutex;
use std::{ops::DerefMut, pin::Pin, sync::Arc};
use tinymist_std::error::prelude::*;

pub(super) type FoldFuture = Pin<Box<dyn Future<Output = Option<()>> + Send>>;

#[derive(Default)]
struct FoldingState {
    running: bool,
    task: Option<(usize, FoldFuture)>,
}

#[derive(Clone, Default)]
pub(super) struct FutureFolder {
    state: Arc<Mutex<FoldingState>>,
}

impl FutureFolder {
    pub(super) async fn compute<OP, R: Send + 'static>(op: OP) -> Result<R>
    where
        OP: FnOnce() -> R + Send + 'static,
    {
        #[cfg(feature = "system")]
        {
            tokio::task::spawn_blocking(move || rayon::in_place_scope(|_| op()))
                .await
                .context_ut("compute error")
        }
        #[cfg(not(feature = "system"))]
        Ok(op())
    }

    #[must_use]
    pub(super) fn spawn(
        &self,
        revision: usize,
        fut: impl FnOnce() -> FoldFuture,
    ) -> Option<impl Future<Output = ()> + Send + 'static> {
        let mut state = self.state.lock();
        let state = state.deref_mut();

        match &mut state.task {
            Some((prev_revision, prev)) => {
                if *prev_revision < revision {
                    *prev = fut();
                    *prev_revision = revision;
                }

                return None;
            }
            next_update => {
                *next_update = Some((revision, fut()));
            }
        }

        if state.running {
            return None;
        }

        state.running = true;

        let state = self.state.clone();
        Some(async move {
            loop {
                let fut = {
                    let mut state = state.lock();
                    let Some((_, fut)) = state.task.take() else {
                        state.running = false;
                        return;
                    };
                    fut
                };
                fut.await;
            }
        })
    }
}

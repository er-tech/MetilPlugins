use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// What an action holds of its call besides the params. It tells the action that the host has
/// cancelled the call (a long action looks at it now and then and stops early; answering a
/// cancelled call is optional, and the host ignores the answer), and it lets the action say it is
/// alive with [`Cancel::progress`].
/// What an action calls to say it is alive.
type ProgressSender = Arc<dyn Fn(&str) + Send + Sync>;

#[derive(Clone, Default)]
pub struct Cancel {
    flag: Arc<AtomicBool>,
    progress: Option<ProgressSender>,
    work_dir: Option<std::path::PathBuf>,
    gpu_device: Option<String>,
}

impl std::fmt::Debug for Cancel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cancel")
            .field("cancelled", &self.is_cancelled())
            .finish()
    }
}

impl Cancel {
    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }

    /// Tells the host the call is alive and, in a few words, what it is doing. A plugin whose
    /// manifest sets `stallSeconds` must do this more often than that, or the host stops it. Does
    /// nothing outside a served call.
    pub fn progress(&self, message: &str) {
        if let Some(send) = &self.progress {
            send(message);
        }
    }

    /// The folder the host gave this plugin to write media in for the next plugin, for this run,
    /// when it offers one: write raw sound there and answer with its description.
    pub fn work_dir(&self) -> Option<&std::path::Path> {
        self.work_dir.as_deref()
    }

    /// The device the host grants for models: `auto`, `cpu`, `gpu` or `gpu:N`, from the `gpu` it
    /// offered in `hello`. `None` when the host offered nothing (no graphics card, or an old host).
    pub fn granted_device(&self) -> Option<&str> {
        self.gpu_device.as_deref()
    }

    /// A token whose `progress` calls `send` and that knows the work folder and the granted device.
    pub(crate) fn for_call(
        send: impl Fn(&str) + Send + Sync + 'static,
        work_dir: Option<std::path::PathBuf>,
        gpu_device: Option<String>,
    ) -> Cancel {
        Cancel {
            flag: Arc::default(),
            progress: Some(Arc::new(send)),
            work_dir,
            gpu_device,
        }
    }

    pub(crate) fn cancel(&self) {
        self.flag.store(true, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_clone_sees_the_cancel() {
        let cancel = Cancel::default();
        let seen_by_the_call = cancel.clone();
        assert!(!seen_by_the_call.is_cancelled());
        cancel.cancel();
        assert!(seen_by_the_call.is_cancelled());
    }

    #[test]
    fn progress_does_nothing_outside_a_served_call_and_calls_the_sender_inside_one() {
        Cancel::default().progress("nobody listens");
        let heard = Arc::new(std::sync::Mutex::new(Vec::new()));
        let sink = Arc::clone(&heard);
        let cancel = Cancel::for_call(
            move |message| sink.lock().unwrap().push(message.to_string()),
            Some(std::path::PathBuf::from("work")),
            Some("gpu:1".to_string()),
        );
        cancel.clone().progress("half way");
        assert_eq!(*heard.lock().unwrap(), ["half way"]);
        assert_eq!(cancel.work_dir(), Some(std::path::Path::new("work")));
        assert_eq!(Cancel::default().work_dir(), None);
        assert_eq!(cancel.granted_device(), Some("gpu:1"));
        assert_eq!(Cancel::default().granted_device(), None);
    }
}

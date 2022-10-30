use std::future::Future;

use color_eyre::Result;
use tokio::signal;
use tracing::info;

/// A future that never resolves, but will call the handler when it detects a shutdown signal.
///
/// Signals: `SIGINT`, `SIGTERM`, `SIGHUP` or explicit `shutdown` call.
///
/// A use-case of this, is to explicitly shutdown the server. Which then cancels this future.
pub async fn on_signal<F, Fut>(confirm_exit: bool, handler: F) -> Result<()>
where
    F: FnOnce(bool) -> Fut,
    Fut: Future<Output = ()>,
{
    let sigint = async {
        if confirm_exit {
            let mut last_signal_timestamp = None;

            loop {
                signal::ctrl_c().await?;

                let now = std::time::Instant::now();
                if let Some(last_signal_timestamp) = last_signal_timestamp {
                    if now.duration_since(last_signal_timestamp) < std::time::Duration::from_secs(5)
                    {
                        info!("[signal] Ctrl-C received");
                        break;
                    }
                }
                info!("[signal] Ctrl-C received, press again to exit");
                last_signal_timestamp = Some(now);
            }
        } else {
            signal::ctrl_c().await?;
        }

        Result::<()>::Ok(())
    };

    let sigterm = async {
        #[cfg(unix)]
        {
            let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())?;
            sigterm.recv().await;
            info!("[signal] SIGTERM received");
        }

        #[cfg(windows)]
        {
            let mut sigterm = signal::windows::ctrl_break()?;
            info!("[signal] Ctrl-Break received");
        }

        Result::<()>::Ok(())
    };

    let sighup = async {
        #[cfg(unix)]
        {
            let mut sighup = signal::unix::signal(signal::unix::SignalKind::hangup())?;
            sighup.recv().await;
            info!("[signal] SIGHUP received");
        }

        #[cfg(windows)]
        std::future::pending::<()>().await;

        Result::<()>::Ok(())
    };

    let res = tokio::select! {
        _ = sighup => handler(false),
        _ = sigint => handler(false),
        _ = sigterm => handler(true),
    };

    tokio::select! {
        _ = async {
            res.await;
            std::future::pending::<()>().await;
        } => {},
        _ = signal::ctrl_c() => {
            info!("[signal] Ctrl-C received while exiting, forcibly exiting...");
        }
    }

    Ok(())
}

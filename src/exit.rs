use std::future::Future;

use color_eyre::Result;
use tokio::signal;
#[cfg(feature = "shutdown-signal")]
use tokio::sync::mpsc;
use tracing::info;

pub async fn on_signal<F, Fut>(
    #[cfg(feature = "shutdown-signal")] shutdown_signal: &mut mpsc::Receiver<()>,
    handler: F,
) -> Result<()>
where
    F: Fn() -> Fut,
    Fut: Future<Output = ()>,
{
    let shutdown = async {
        #[cfg(feature = "shutdown-signal")]
        shutdown_signal.recv().await;

        #[cfg(not(feature = "shutdown-signal"))]
        std::future::pending::<()>().await;
    };

    let sigint = async {
        // let mut last_signal_timestamp = None;

        // loop {
        //     signal::ctrl_c().await?;

        //     let now = std::time::Instant::now();
        //     if let Some(last_signal_timestamp) = last_signal_timestamp {
        //         if now.duration_since(last_signal_timestamp) < std::time::Duration::from_secs(5) {
        //             info!("[signal] Ctrl-C received, shutting down...");
        //             break;
        //         }
        //     }
        //     info!("[signal] Ctrl-C received, press again to exit");
        //     last_signal_timestamp = Some(now);
        // }

        signal::ctrl_c().await?;
        info!("[signal] Ctrl-C received");

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

    tokio::select! {
        _ = sighup => {}
        _ = sigint => {}
        _ = sigterm => {}
        _ = shutdown => {}
    }

    handler().await;
    std::future::pending::<()>().await;

    unreachable!()
}
use color_eyre::Result;
use tokio::{signal, sync::mpsc};
use tracing::info;

pub async fn on_exit(exit_signal: &mpsc::Sender<()>) -> Result<()> {
    let sigint = async {
        let mut last_signal_timestamp = None;

        loop {
            signal::ctrl_c().await?;

            let now = std::time::Instant::now();
            if let Some(last_signal_timestamp) = last_signal_timestamp {
                if now.duration_since(last_signal_timestamp) < std::time::Duration::from_secs(5) {
                    info!("[signal] Ctrl-C received, shutting down...");
                    break;
                }
            }
            info!("[signal] Ctrl-C received, press again to exit");
            last_signal_timestamp = Some(now);
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

    tokio::select! {
        _ = sighup => {}
        _ = sigint => {}
        _ = sigterm => {}
    }

    exit_signal.send(()).await?;
    std::future::pending::<()>().await;
    Ok(())
}

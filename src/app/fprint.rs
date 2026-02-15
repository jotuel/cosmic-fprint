use crate::fprint_dbus::{DeviceProxy, ManagerProxy};
use crate::app::message::Message;
use crate::app::error::AppError;
use futures_util::sink::Sink;
use futures_util::{SinkExt, StreamExt};

pub async fn find_device(
    connection: &zbus::Connection,
) -> zbus::Result<zbus::zvariant::OwnedObjectPath> {
    let manager = ManagerProxy::new(&connection).await?;
    let device = manager.get_default_device().await?;
    Ok(device)
}

pub async fn list_enrolled_fingers_dbus(
    connection: &zbus::Connection,
    path: zbus::zvariant::OwnedObjectPath,
    username: String,
) -> zbus::Result<Vec<String>> {
    let device = DeviceProxy::builder(connection).path(path)?.build().await?;
    device.list_enrolled_fingers(&username).await
}

pub async fn delete_fingerprint_dbus(
    connection: &zbus::Connection,
    path: zbus::zvariant::OwnedObjectPath,
    finger: String,
    username: String,
) -> zbus::Result<()> {
    let device = DeviceProxy::builder(connection).path(path)?.build().await?;

    device.claim(&username).await?;
    let res = device.delete_enrolled_finger(&finger).await;
    let rel_res = device.release().await;
    res.and(rel_res)
}

pub async fn enroll_fingerprint_process<S>(
    connection: zbus::Connection,
    path: zbus::zvariant::OwnedObjectPath,
    finger_name: String,
    username: String,
    output: &mut S,
) -> zbus::Result<()>
where
    S: Sink<Message> + Unpin + Send,
    S::Error: std::fmt::Debug + Send,
{
    let device = DeviceProxy::builder(&connection)
        .path(path)?
        .build()
        .await?;

    // Claim device
    match device.claim(&username).await {
        Ok(_) => {}
        Err(e) => return Err(e),
    };

    let total_stages = match device.num_enroll_stages().await {
        Ok(n) if n > 0 => Some(n as u32),
        _ => None,
    };
    let _ = output.send(Message::EnrollStart(total_stages)).await;

    // Start enrollment
    if let Err(e) = device.enroll_start(&finger_name).await {
        let _ = device.release().await;
        return Err(e);
    }

    // Listen for signals
    let mut stream = match device.receive_enroll_status().await {
        Ok(s) => s,
        Err(e) => {
            let _ = device.release().await;
            return Err(e);
        }
    };

    while let Some(signal) = stream.next().await {
        let args = signal.args();
        match args {
            Ok(args) => {
                let result: String = args.result;
                let done: bool = args.done;

                // Map result string to user friendly message if needed, or pass through
                let _ = output
                    .send(Message::EnrollStatus(result, done))
                    .await;

                if done {
                    break;
                }
            }
            Err(_) => {
                let _ = output
                    .send(Message::OperationError(
                        AppError::Unknown("Failed to parse signal".to_string()),
                    ))
                    .await;
                break;
            }
        }
    }

    // Release device
    let _ = device.release().await;

    Ok(())
}

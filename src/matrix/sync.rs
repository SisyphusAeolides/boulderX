use tokio::sync::mpsc;
use relm4::ComponentSender;
use crate::app::{AppInput, AppModel, Protocol};
use super::client::MatrixEvent;

pub fn bridge_matrix_events(
    mut rx: mpsc::UnboundedReceiver<MatrixEvent>,
    sender: ComponentSender<AppModel>,
) {
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                MatrixEvent::Connected { user_id } => {
                    sender.input(AppInput::MatrixConnected { user_id });
                }
                MatrixEvent::RoomMessage { room_id, room_name, sender: msg_sender, body } => {
                    sender.input(AppInput::ReceiveMessage {
                        channel: room_name,
                        user: msg_sender,
                        body,
                        protocol: Protocol::Matrix { room_id },
                    });
                }
                MatrixEvent::RoomJoined { room_id, room_name } => {
                    sender.input(AppInput::MatrixRoomJoined { room_id, room_name });
                }
                MatrixEvent::RoomLeft { room_id } => {
                    sender.input(AppInput::MatrixRoomLeft { room_id });
                }
                MatrixEvent::SyncError(e) => {
                    sender.input(AppInput::ReceiveServerMessage(format!("[Matrix Error]: {e}")));
                }
                MatrixEvent::Disconnected => {
                    sender.input(AppInput::ReceiveServerMessage("[Matrix]: Disconnected.".to_string()));
                }
            }
        }
    });
}

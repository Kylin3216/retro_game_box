use std::sync::Arc;
use std::sync::mpsc::channel;
use std::thread::spawn;
use std::time::Duration;
use crossbeam_channel::{bounded, Sender};
use flutter_rust_bridge::frb;
use irondash_run_loop::RunLoop;
use irondash_texture::{BoxedPixelData, SendableTexture, Texture};
use crate::payload::NesFrameProvider;

#[frb(opaque)]
pub struct NesTexture {
    id: i64,
    texture: Arc<SendableTexture<BoxedPixelData>>,
    tx: Sender<Vec<u8>>,
}


impl NesTexture {
    pub fn create(handle: i64) -> anyhow::Result<Option<NesTexture>> {
        let (ntx, nrx) = channel();
        RunLoop::sender_for_main_thread()?.send(move || {
            let (tx, rx) = bounded(100);
            let provider = Arc::new(NesFrameProvider::new(rx));
            match Texture::new_with_provider(handle, provider) {
                Ok(texture) => {
                    let id = texture.id();
                    let texture = texture.into_sendable_texture();
                    let mt = texture.clone();
                    spawn(move || {
                        mt.mark_frame_available();
                        loop {}
                    });
                    let nes_texture = NesTexture {
                        id,
                        texture,
                        tx,
                    };
                    let _ = ntx.send(Some(nes_texture));
                }
                Err(_) => {
                    let _ = ntx.send(None);
                }
            }
        });
        let nes_texture = nrx.recv_timeout(Duration::from_secs(1))?;
        Ok(nes_texture)
    }

    pub fn render(&self, data: Vec<u8>) -> anyhow::Result<()> {
        self.tx.send(data)?;
        self.texture.mark_frame_available();
        Ok(())
    }

    #[frb(sync)]
    pub fn id(&self) -> i64 {
        self.id
    }
}
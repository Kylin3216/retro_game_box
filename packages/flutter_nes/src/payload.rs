use crossbeam_channel::{Receiver};
use irondash_texture::{BoxedPixelData, PayloadProvider, SimplePixelData};
use nes_core::ppu::Ppu;
use rgb::{ComponentBytes, FromSlice};

pub struct NesFrameProvider {
    rx: Receiver<Vec<u8>>,
}

impl NesFrameProvider {
    pub fn new(rx: Receiver<Vec<u8>>) -> NesFrameProvider {
        NesFrameProvider {
            rx
        }
    }
}

impl PayloadProvider<BoxedPixelData> for NesFrameProvider {
    fn get_payload(&self) -> BoxedPixelData {
        let buffer = self.rx.recv().unwrap_or_else(|_| {
            vec![0; 245760]
        });
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        {
            /// convert rgba to bgra on macos and ios
            let buffer = buffer.as_rgba().to_vec().iter()
                .map(|rgba| rgba.bgr().alpha(255)).collect::<Vec<_>>().as_bytes().to_vec();
            return SimplePixelData::new_boxed(Ppu::WIDTH as i32, Ppu::HEIGHT as i32, buffer);
        }
        SimplePixelData::new_boxed(Ppu::WIDTH as i32, Ppu::HEIGHT as i32, buffer)
    }
}
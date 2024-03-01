// mod filter;
// mod window_sinc;

pub trait Audio {
    fn output(&self) -> f32;
}
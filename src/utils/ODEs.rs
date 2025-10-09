pub trait ODEFunc {
    fn call(&self, t: f32, y: Vec<f32>) -> Vec<f32>;
}
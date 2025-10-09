use crate::utils::ODEs::ODEFunc;

// This is pretty terribly optimized code to get something working quickly.
// Please don't look at it.
pub fn rk4(ode: &dyn ODEFunc, t: f32, y: Vec<f32>, dt: f32) -> Vec<f32> {
    let k1 = ode.call(t, y.clone());
    let scale = |v: &Vec<f32>, s: f32| v.iter().map(|x| x * s).collect();
    let add = |v1: &Vec<f32>, v2: &Vec<f32>| v1.iter().zip(v2.iter()).map(|(x, y)| x + y).collect();
    let k2 = ode.call(t + dt / 2.0, add(&y, &scale(&k1, dt / 2.0)));
    let k3 = ode.call(t + dt / 2.0, add(&y, &scale(&k2, dt / 2.0)));
    let k4 = ode.call(t + dt, add(&y, &scale(&k3, dt)));
    add(&y, &scale(&add(&add(&k1, &scale(&k2, 2.0)), &add(&scale(&k3, 2.0), &k4)), dt / 6.0))
}
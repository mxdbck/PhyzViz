use crate::utils::ODEs::ODEFunc;

pub fn rk4(ode: &dyn ODEFunc, t: f32, y: Vec<f32>, dt: f32) -> Vec<f32> {
    let n = y.len();
    let half_dt = dt * 0.5;
    let sixth = dt / 6.0;

    // k1
    let k1 = ode.call(t, y.clone());

    // k2 input: y + (dt/2)*k1
    let mut y_tmp = Vec::with_capacity(n);
    for i in 0..n {
        y_tmp.push(y[i] + half_dt * k1[i]);
    }
    let k2 = ode.call(t + half_dt, y_tmp);

    // k3 input: y + (dt/2)*k2
    let mut y_tmp = Vec::with_capacity(n);
    for i in 0..n {
        y_tmp.push(y[i] + half_dt * k2[i]);
    }
    let k3 = ode.call(t + half_dt, y_tmp);

    // k4 input: y + dt*k3
    let mut y_tmp = Vec::with_capacity(n);
    for i in 0..n {
        y_tmp.push(y[i] + dt * k3[i]);
    }
    let k4 = ode.call(t + dt, y_tmp);

    // y_next = y + dt/6 * (k1 + 2*k2 + 2*k3 + k4)
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        out.push(y[i] + sixth * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]));
    }
    out
}

// This is pretty terribly optimized code to get something working quickly.
// Please don't look at it.
pub fn rk4_old(ode: &dyn ODEFunc, t: f32, y: Vec<f32>, dt: f32) -> Vec<f32> {
    let k1 = ode.call(t, y.clone());
    let scale = |v: &Vec<f32>, s: f32| v.iter().map(|x| x * s).collect();
    let add = |v1: &Vec<f32>, v2: &Vec<f32>| v1.iter().zip(v2.iter()).map(|(x, y)| x + y).collect();
    let k2 = ode.call(t + dt / 2.0, add(&y, &scale(&k1, dt / 2.0)));
    let k3 = ode.call(t + dt / 2.0, add(&y, &scale(&k2, dt / 2.0)));
    let k4 = ode.call(t + dt, add(&y, &scale(&k3, dt)));
    add(&y, &scale(&add(&add(&k1, &scale(&k2, 2.0)), &add(&scale(&k3, 2.0), &k4)), dt / 6.0))
}
use crate::utils::ODEs::ODEFunc;

pub struct RK4Prealloc {
    pub y0: Vec<f32>,
    pub k1: Vec<f32>,
    pub k2: Vec<f32>,
    pub k3: Vec<f32>,
    pub k4: Vec<f32>,
    pub out: Vec<f32>,

    pub func: Box<dyn ODEFunc + Send + Sync>,
}

pub fn rk4(
    t: f32,
    dt: f32,
    rk_params: &mut RK4Prealloc
) {
    let ode = &*rk_params.func;
    let y = &rk_params.y0;
    let k1 = &mut rk_params.k1;
    let k2 = &mut rk_params.k2;
    let k3 = &mut rk_params.k3;
    let k4 = &mut rk_params.k4;
    let out = &mut rk_params.out;




    let n = y.len();
    let half_dt = dt * 0.5;
    let sixth = dt / 6.0;

    // k1
    ode.call(t, y, k1);

    // k2 input: y + (dt/2)*k1
    for i in 0..n {
        out[i] = y[i] + half_dt * k1[i];
    }
    ode.call(t + half_dt, &out, k2);

    // k3 input: y + (dt/2)*k2
    for i in 0..n {
        out[i] = y[i] + half_dt * k2[i];
    }
    ode.call(t + half_dt, &out, k3);

    // k4 input: y + dt*k3
    for i in 0..n {
        out[i] = y[i] + dt * k3[i];
    }
    ode.call(t + dt, &out, k4);

    // y_next = y + dt/6 * (k1 + 2*k2 + 2*k3 + k4)
    for i in 0..n {
        out[i] = y[i] + sixth * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]);
    }
}
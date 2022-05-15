pub const fn linear(x: f64)        -> f64 { x }
pub       fn threshold(x: f64)     -> f64 { if x > 0.0 { 1.0 } else { 0.0 } }
pub       fn relu(x: f64)          -> f64 { if x > 0.0 { x } else { 0.0 } } 
pub       fn sign(x: f64)          -> f64 { if x > 0.0 { 1.0 } else if x == 0.0 { 0.0 } else { -1.0 } }
pub       fn sigmoid(x: f64)       -> f64 { 1.0 / (1.0 + (-x).exp()) }
pub       fn tanh(x: f64)          -> f64 { x.tanh() }
pub       fn soft_sign(x: f64)     -> f64 { x / (1.0 + x.abs()) }
pub       fn bent_identity(x: f64) -> f64 { (((x.powi(2) + 1.0).sqrt() - 1.0) / 2.0) + x }
pub const fn linear(x: f32)        -> f32 { x }
pub       fn threshold(x: f32)     -> f32 { if x > 0.0 { 1.0 } else { 0.0 } }
pub       fn relu(x: f32)          -> f32 { if x > 0.0 { x } else { 0.0 } } 
pub       fn sign(x: f32)          -> f32 { if x > 0.0 { 1.0 } else if x == 0.0 { 0.0 } else { -1.0 } }
pub       fn sigmoid(x: f32)       -> f32 { 1.0 / (1.0 + (-x).exp()) }
pub       fn tanh(x: f32)          -> f32 { x.tanh() }
pub       fn soft_sign(x: f32)     -> f32 { x / (1.0 + x.abs()) }
pub       fn bent_identity(x: f32) -> f32 { (((x.powi(2) + 1.0).sqrt() - 1.0) / 2.0) + x }
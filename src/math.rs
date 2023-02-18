use std::f32::consts::{PI, TAU};

/// Make sure the angle `ang` is within +/- PI
pub fn clean_angle_radians(ang: f32) -> f32 {
    // https://stackoverflow.com/questions/2320986/easy-way-to-keeping-angles-between-179-and-180-degrees
    // sigh...

    let ang = ang % TAU;
    let ang = (ang + TAU) % TAU;
    if ang > PI {
        ang - TAU
    } else {
        ang
    }
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    ((1.0 - t) * a) + (t * b)
}

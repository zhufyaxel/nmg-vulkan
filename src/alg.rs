#![allow(dead_code)] // Library

use std;

fn inverse_sqrt(x: f32) -> f32 {
    let half = x * 0.5;

    let cast: u32 = unsafe {
        std::mem::transmute(x)
    };

    let guess = 0x5f3759df - (cast >> 1);
    let guess = f32::from_bits(guess);

    let iteration = guess * (1.5 - half * guess * guess);
    iteration * (1.5 - half * iteration * iteration)
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Vec2 {
        Vec2 { x, y }
    }

    #[inline]
    pub fn up() -> Vec2 {
        Vec2 {
            x: 0.0,
            y: 1.0,
        }
    }

    #[inline]
    pub fn right() -> Vec2 {
        Vec2 {
            x: 1.0,
            y: 0.0,
        }
    }

    #[inline]
    pub fn dot(self, other: Vec2) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn norm(self) -> Vec2 {
        let inverse_len = inverse_sqrt(self.mag_squared());

        Vec2 {
            x: self.x * inverse_len,
            y: self.y * inverse_len,
        }
    }

    pub fn mag_squared(self) -> f32 {
        self.x * self.x + self.y * self.y
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(C)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3 { x, y, z }
    }

    #[inline]
    pub fn right() -> Vec3 {
        Vec3::new(1., 0., 0.)
    }

    #[inline]
    pub fn up() -> Vec3 {
        Vec3::new(0., 1., 0.)
    }

    #[inline]
    pub fn fwd() -> Vec3 {
        Vec3::new(0., 0., 1.)
    }

    #[inline]
    pub fn zero() -> Vec3 {
        Vec3::new(0., 0., 0.,)
    }

    #[inline]
    pub fn one() -> Vec3 {
        Vec3::new(1., 1., 1.)
    }

    pub fn norm(self) -> Vec3 {
        let inverse_len = inverse_sqrt(self.mag_squared());

        Vec3::new(
            self.x * inverse_len,
            self.y * inverse_len,
            self.z * inverse_len,
        )
    }

    pub fn mag_squared(self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn mag(self) -> f32 {
        self.mag_squared().sqrt()
    }

    pub fn dist_squared(self, other: Vec3) -> f32 {
        (self - other).mag_squared()
    }

    pub fn dist(self, other: Vec3) -> f32 {
        self.dist_squared(other).sqrt()
    }

    #[inline]
    pub fn cross(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    #[inline]
    pub fn dot(self, other: Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    #[inline]
    pub fn lerp(self, other: Vec3, t: f32) -> Vec3 {
        self * (1. - t) + other * t
    }
}

impl std::ops::Add for Vec3 {
    type Output = Vec3;

    fn add(self, other: Vec3) -> Vec3 {
        Vec3::new(
            self.x + other.x,
            self.y + other.y,
            self.z + other.z,
        )
    }
}

impl std::ops::Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, other: Vec3) -> Vec3 {
        Vec3::new(
            self.x - other.x,
            self.y - other.y,
            self.z - other.z,
        )
    }
}

impl std::ops::Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Vec3 {
        Vec3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl std::ops::Mul<f32> for Vec3 {
    type Output = Vec3;

    fn mul(self, scalar: f32) -> Vec3 {
        Vec3::new(
            self.x * scalar,
            self.y * scalar,
            self.z * scalar,
        )
    }
}

impl std::ops::Div<f32> for Vec3 {
    type Output = Vec3;

    fn div(self, scalar: f32) -> Vec3 {
        Vec3::new(
            self.x / scalar,
            self.y / scalar,
            self.z / scalar,
        )
    }
}

impl std::fmt::Display for Vec3 {
    fn fmt(&self, out: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            out,
            "( {}, {}, {} )",
            self.x, self.y, self.z,
        )
    }
}

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Mat {

    /*
     * GLSL expects matrices in column-major order
     * Calculations below are formatted in row-major order
     */

    pub x0: f32, pub y0: f32, pub z0: f32, pub w0: f32,
    pub x1: f32, pub y1: f32, pub z1: f32, pub w1: f32,
    pub x2: f32, pub y2: f32, pub z2: f32, pub w2: f32,
    pub x3: f32, pub y3: f32, pub z3: f32, pub w3: f32,
}

impl Mat {
    pub fn new(
        x0: f32, x1: f32, x2: f32, x3: f32,
        y0: f32, y1: f32, y2: f32, y3: f32,
        z0: f32, z1: f32, z2: f32, z3: f32,
        w0: f32, w1: f32, w2: f32, w3: f32,
    ) -> Mat {
        Mat {
            x0, x1, x2, x3,
            y0, y1, y2, y3,
            z0, z1, z2, z3,
            w0, w1, w2, w3,
        }
    }

    #[inline]
    pub fn id() -> Mat {
        Mat::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        )
    }

    pub fn translation(x: f32, y: f32, z: f32) -> Mat {
        Mat::new(
            1.0, 0.0, 0.0,   x,
            0.0, 1.0, 0.0,   y,
            0.0, 0.0, 1.0,   z,
            0.0, 0.0, 0.0, 1.0,
        )
    }

    pub fn translation_vec(translation: Vec3) -> Mat {
        Mat::translation(translation.x, translation.y, translation.z)
    }

    pub fn rotation_x(rad: f32) -> Mat {
        Mat::new(
            1.0,       0.0,        0.0, 0.0,
            0.0, rad.cos(), -rad.sin(), 0.0,
            0.0, rad.sin(),  rad.cos(), 0.0,
            0.0,       0.0,        0.0, 1.0,
        )
    }

    pub fn rotation_y(rad: f32) -> Mat {
        Mat::new(
             rad.cos(), 0.0, rad.sin(), 0.0,
                   0.0, 1.0,       0.0, 0.0,
            -rad.sin(), 0.0, rad.cos(), 0.0,
                   0.0, 0.0,       0.0, 1.0,
        )
    }

    pub fn rotation_z(rad: f32) -> Mat {
        Mat::new(
             rad.cos(), rad.sin(), 0.0, 0.0,
            -rad.sin(), rad.cos(), 0.0, 0.0,
                  0.0,        0.0, 1.0, 0.0,
                  0.0,        0.0, 0.0, 1.0,
        )
    }

    pub fn rotation(x: f32, y: f32, z: f32) -> Mat {
        Mat::rotation_x(x) * Mat::rotation_y(y) * Mat::rotation_z(z)
    }

    pub fn scale(x: f32, y: f32, z: f32) -> Mat {
        Mat::new(
              x, 0.0, 0.0, 0.0,
            0.0,   y, 0.0, 0.0,
            0.0, 0.0,   z, 0.0,
            0.0, 0.0, 0.0, 1.0
        )
    }

    pub fn scale_vec(scale: Vec3) -> Mat {
        Mat::scale(scale.x, scale.y, scale.z)
    }

    pub fn axes(right: Vec3, up: Vec3, fwd: Vec3) -> Mat {
        Mat::new(
            right.x, up.x, fwd.x, 0.0,
            right.y, up.y, fwd.y, 0.0,
            right.z, up.z, fwd.z, 0.0,
                0.0,  0.0,   0.0, 1.0,
        )
    }

    pub fn inverse_axes(right: Vec3, up: Vec3, fwd: Vec3) -> Mat {
        Mat::new(
            right.x, right.y, right.z, 0.0,
               up.x,    up.y,    up.z, 0.0,
              fwd.x,   fwd.y,   fwd.z, 0.0,
                0.0,     0.0,     0.0, 1.0,
        )
    }

    pub fn to_cardan(self) -> (f32, f32, f32) {
        let cy = (
            self.x0 * self.x0 + self.x1 * self.x1
        ).sqrt();

        if cy < 16. * std::f32::EPSILON { // Singular matrix
            (
               -(-self.z1).atan2(self.y1),
               -(-self.x2).atan2(cy),
                0.0, // Fix for gimbal lock
            )
        } else {
            (
               -( self.y2).atan2(self.z2),
               -(-self.x2).atan2(cy),
                ( self.x1).atan2(self.x0),
            )
        }
    }

    pub fn to_cardan_safe(self) -> (f32, f32, f32) {
        let cy = (
            self.x0 * self.x0 + self.x1 * self.x1
        ).sqrt();

        let ax = -(self.y2).atan2(self.z2);
        let cx = ax.cos();
        let sx = ax.sin();

        (
            ax,
            -(-self.x2).atan2(cy),
            (sx * self.z0 - cx * self.y0).atan2(cx * self.y1 - sx * self.z1),
        )
    }

    // Returns view matrix (inverted)
    pub fn look_at_view(position: Vec3, target: Vec3, up: Vec3) -> Mat {
        let fwd = (target - position).norm();
        let right = up.cross(fwd).norm();
        let up = fwd.cross(right);

        // Transpose orthogonal matrix to get inverse
        let inverse_rotation = Mat::inverse_axes(right, up, fwd);

        // Reverse position input
        let inverse_position = Mat::translation(
            -position.x,
            -position.y,
            -position.z,
        );

        inverse_rotation * inverse_position
    }

    // Input: vertical field of view, screen aspect ratio, near and far planes
    pub fn perspective(fov: f32, aspect: f32, near: f32, far: f32) -> Mat {
        // Perspective scaling (rectilinear)
        let y_scale = 1. / (0.5 * fov).to_radians().tan();
        let x_scale = y_scale / aspect;

        // Fit into Vulkan clip space (0-1)
        let z_scale = 1. / (far - near);
        let z_offset = -near / (far - near);

        Mat::new(
            x_scale,      0.0,     0.0,      0.0,
                0.0, -y_scale,     0.0,      0.0, // Flip for Vulkan
                0.0,      0.0, z_scale, z_offset,
                0.0,      0.0,     1.0,      0.0, // Left-handed (scaling factor)
        )
    }

    pub fn transpose(self) -> Mat {
        Mat::new(
            self.x0, self.y0, self.z0, self.w0,
            self.x1, self.y1, self.z1, self.w1,
            self.x2, self.y2, self.z2, self.w2,
            self.x3, self.y3, self.z3, self.w3,
        )
    }
}

impl std::ops::Mul for Mat {
    type Output = Mat;

    // Naive matrix multiply
    fn mul(self, m: Mat) -> Mat {
        let x0 = self.x0 * m.x0 + self.x1 * m.y0 + self.x2 * m.z0 + self.x3 * m.w0;
        let x1 = self.x0 * m.x1 + self.x1 * m.y1 + self.x2 * m.z1 + self.x3 * m.w1;
        let x2 = self.x0 * m.x2 + self.x1 * m.y2 + self.x2 * m.z2 + self.x3 * m.w2;
        let x3 = self.x0 * m.x3 + self.x1 * m.y3 + self.x2 * m.z3 + self.x3 * m.w3;

        let y0 = self.y0 * m.x0 + self.y1 * m.y0 + self.y2 * m.z0 + self.y3 * m.w0;
        let y1 = self.y0 * m.x1 + self.y1 * m.y1 + self.y2 * m.z1 + self.y3 * m.w1;
        let y2 = self.y0 * m.x2 + self.y1 * m.y2 + self.y2 * m.z2 + self.y3 * m.w2;
        let y3 = self.y0 * m.x3 + self.y1 * m.y3 + self.y2 * m.z3 + self.y3 * m.w3;

        let z0 = self.z0 * m.x0 + self.z1 * m.y0 + self.z2 * m.z0 + self.z3 * m.w0;
        let z1 = self.z0 * m.x1 + self.z1 * m.y1 + self.z2 * m.z1 + self.z3 * m.w1;
        let z2 = self.z0 * m.x2 + self.z1 * m.y2 + self.z2 * m.z2 + self.z3 * m.w2;
        let z3 = self.z0 * m.x3 + self.z1 * m.y3 + self.z2 * m.z3 + self.z3 * m.w3;

        let w0 = self.w0 * m.x0 + self.w1 * m.y0 + self.w2 * m.z0 + self.w3 * m.w0;
        let w1 = self.w0 * m.x1 + self.w1 * m.y1 + self.w2 * m.z1 + self.w3 * m.w1;
        let w2 = self.w0 * m.x2 + self.w1 * m.y2 + self.w2 * m.z2 + self.w3 * m.w2;
        let w3 = self.w0 * m.x3 + self.w1 * m.y3 + self.w2 * m.z3 + self.w3 * m.w3;

        Mat::new(
            x0, x1, x2, x3,
            y0, y1, y2, y3,
            z0, z1, z2, z3,
            w0, w1, w2, w3,
        )
    }
}

impl std::ops::Mul<Vec3> for Mat {
    type Output = Vec3;

    fn mul(self, vec: Vec3) -> Vec3 {
        // Assume scaling factor of one (position)
        Vec3::new(
            self.x0 * vec.x + self.x1 * vec.y + self.x2 * vec.z + self.x3,
            self.y0 * vec.x + self.y1 * vec.y + self.y2 * vec.z + self.y3,
            self.z0 * vec.x + self.z1 * vec.y + self.z2 * vec.z + self.z3,
        )
    }
}

impl std::fmt::Display for Mat {
    fn fmt(&self, out: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            out,
            "[ {}, {}, {}, {} ]\n[ {}, {}, {}, {} ]\n\
            [ {}, {}, {}, {} ]\n[ {}, {}, {}, {} ]",
            self.x0, self.x1, self.x2, self.x3,
            self.y0, self.y1, self.y2, self.y3,
            self.z0, self.z1, self.z2, self.z3,
            self.w0, self.w1, self.w2, self.w3,
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quat {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Quat {
        Quat { x, y, z, w }
    }

    pub fn from_mat(m: Mat) -> Quat {
        let w = (1.0 + m.x0 + m.y1 + m.z2).sqrt() * 0.5;
        let x4 = w * 4.0;

        Quat {
            x: (m.z1 - m.y2) / x4,
            y: (m.x2 - m.z0) / x4,
            z: (m.y0 - m.x1) / x4,
            w: w,
        }
    }

    #[inline]
    pub fn id() -> Quat {
        Quat {
            x: 0.,
            y: 0.,
            z: 0.,
            w: 1.,
        }
    }

    pub fn axis_angle(axis: Vec3, angle: f32) -> Quat {
        Quat::axis_angle_raw(axis.norm(), angle) // Normalize first
    }

    pub fn axis_angle_raw(axis: Vec3, angle: f32) -> Quat {
        let half = 0.5 * angle;
        let half_sin = half.sin();
        let half_cos = half.cos();

        Quat {
            x: axis.x * half_sin,
            y: axis.y * half_sin,
            z: axis.z * half_sin,
            w: half_cos,
        }
    }

    // Hestenes "simple" rotation (does not preserve Z twist)
    pub fn simple(from: Vec3, to: Vec3) -> Quat {
        let axis = from.cross(to);
        let dot = from.dot(to);

        if dot < -1.0 + std::f32::EPSILON {
            Quat {
                x: 0.0,
                y: 1.0,
                z: 0.0,
                w: 0.0,
            }
        } else {
            Quat {
                x: axis.x,
                y: axis.y,
                z: axis.z,
                w: (dot + 1.0),
            }.norm()
        }
    }

    pub fn norm(self) -> Quat {
        let inverse_len = inverse_sqrt(self.mag_squared());

        Quat {
            x: self.x * inverse_len,
            y: self.y * inverse_len,
            z: self.z * inverse_len,
            w: self.w * inverse_len,
        }
    }

    pub fn mag_squared(self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    pub fn mag(self) -> f32 {
        self.mag_squared().sqrt()
    }

    pub fn conjugate(self) -> Quat {
        Quat {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w:  self.w,
        }
    }

    pub fn to_mat(self) -> Mat {
        self.norm().to_mat_raw() // Normalize first
    }

    pub fn to_mat_raw(self) -> Mat {
        let x0 = 1. - 2. * self.y * self.y - 2. * self.z * self.z;
        let y0 = 2. * self.x * self.y - 2. * self.z * self.w;
        let z0 = 2. * self.x * self.z + 2. * self.y * self.w;

        let x1 = 2. * self.x * self.y + 2. * self.z * self.w;
        let y1 = 1. - 2. * self.x * self.x - 2. * self.z * self.z;
        let z1 = 2. * self.y * self.z - 2. * self.x * self.w;

        let x2 = 2. * self.x * self.z - 2. * self.y * self.w;
        let y2 = 2. * self.y * self.z + 2. * self.x * self.w;
        let z2 = 1. - 2. * self.x * self.x - 2. * self.y * self.y;

        Mat::new(
            x0, y0, z0, 0.,
            x1, y1, z1, 0.,
            x2, y2, z2, 0.,
            0., 0., 0., 1.,
        )
    }

    pub fn to_axis_angle(self) -> (Vec3, f32) {
        let this = if self.w > 1.0 { self.norm() } else { self };
        this.to_axis_angle_raw()
    }

    pub fn to_axis_angle_raw(self) -> (Vec3, f32) {
        let div = inverse_sqrt(1.0 - self.w * self.w);

        if 1.0 / div < std::f32::EPSILON {
            (
                Vec3::new(
                    self.x,
                    self.y,
                    self.z,
                ),
                self.angle(),
            )
        } else {
            (
                Vec3::new(
                    self.x * div,
                    self.y * div,
                    self.z * div,
                ),
                self.angle(),
            )
        }
    }

    #[inline]
    pub fn angle(self) -> f32 {
        self.w.acos() * 2.0
    }

    pub fn pow(self, t: f32) -> Quat {
        let mag = Vec3::new(self.x, self.y, self.z).mag();

        let scale = if mag > std::f32::EPSILON {
            mag.atan2(self.w) * t
        } else { 0.0 };

        let x = self.x * scale;
        let y = self.y * scale;
        let z = self.z * scale;
        let w = self.mag_squared().ln() * 0.5 * t;

        let mag = Vec3::new(x, y, z).mag();
        let wexp = w.exp();

        let scale = if mag >= std::f32::EPSILON {
            wexp * mag.sin() / mag
        } else { 0.0 };

        Quat {
            x: x * scale,
            y: y * scale,
            z: z * scale,
            w: wexp * mag.cos(),
        }
    }
}

impl std::cmp::PartialEq for Quat {
    fn eq(&self, other: &Quat) -> bool {
        let equal = {
               self.x == other.x
            && self.y == other.y
            && self.z == other.z
            && self.w == other.w
        };

        if !equal {
               self.x == -other.x
            && self.y == -other.y
            && self.z == -other.z
            && self.w == -other.w
        } else { true }
    }
}

impl std::ops::Mul<Vec3> for Quat {
    type Output = Vec3;

    fn mul(self, vec: Vec3) -> Vec3 {
        self.to_mat() * vec
    }
}

impl std::ops::Mul for Quat {
    type Output = Quat;

    fn mul(self, other: Quat) -> Quat {
        let x = self.w * other.x
            + self.x * other.w
            + self.y * other.z
            - self.z * other.y;

        let y = self.w * other.y
            + self.y * other.w
            + self.z * other.x
            - self.x * other.z;

        let z = self.w * other.z
            + self.z * other.w
            + self.x * other.y
            - self.y * other.x;

        let w = self.w * other.w
            - self.x * other.x
            - self.y * other.y
            - self.z * other.z;

        Quat { x, y, z, w, }
    }
}

impl std::ops::Mul<f32> for Quat {
    type Output = Quat;

    fn mul(self, scalar: f32) -> Quat {
        Quat {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
            w: self.w * scalar,
        }
    }
}

impl std::ops::Add for Quat {
    type Output = Quat;

    fn add(self, other: Quat) -> Quat {
        Quat {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.y,
            w: self.w + other.w,
        }
    }
}

impl std::fmt::Display for Quat {
    fn fmt(&self, out: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(out, "( {}, {}, {}, {} )", self.x, self.y, self.z, self.w)
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Plane {
    pub normal: Vec3,
    pub offset: f32,
}

impl Plane {
    pub fn new(normal: Vec3, offset: f32) -> Plane {
        Plane {
            normal: normal.norm(),
            offset: offset,
        }
    }

    pub fn new_raw(normal: Vec3, offset: f32) -> Plane {
        Plane { normal, offset }
    }

    #[inline]
    pub fn contains(self, point: Vec3) -> bool {
        self.normal.dot(point) > 0.0
    }

    #[inline]
    pub fn intersects(self, start: Vec3, ray: Vec3) -> bool {
        self.normal.dot(start + ray) < 0.0
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Line {
    pub start: Vec3,
    pub end: Vec3,
}

impl Line {
    pub fn new(start: Vec3, end: Vec3) -> Line {
        Line {
            start,
            end,
        }
    }
}

#[cfg(test)]
mod tests {
    use alg::*;

    #[test]
    fn mul_mat() {
        let translation = Mat::translation(1.0, 2.0, 3.0);

        assert!(translation * Mat::id() == translation);
        assert!(Mat::id() * translation == translation);
    }

    #[test]
    fn mul_vec() {
        let vec = Vec3::new(9., -4., 0.);
        let scale = Mat::scale(-1., 3., 2.);

        assert!(Mat::id() * vec == vec);
        assert!(scale * vec == Vec3::new(-9., -12., 0.));

        let mat = Mat::new(
            1., 1., 1., 0.,
            0., 1., 0., 0.,
            0., 0., 0., 0.,
            0., 0., 0., 0.,
        );

        assert!(mat * vec == Vec3::new(5., -4., 0.,));

        let translation = Mat::translation(2., -7., 0.5);
        assert!(translation * Vec3::zero() == Vec3::new(2., -7., 0.5));
    }

    #[test]
    fn convert_quat() {
        assert!(Quat::id().to_mat() == Mat::id());

        let quat = Quat::axis_angle(Vec3::right(), -6.3);
        let mat = Mat::rotation_x(-6.3);

        let error = mat_error(quat.to_mat(), mat);
        eprintln!("Error: {}", error);
        assert!(error < 0.0001);
    }

    #[test]
    fn mul_quat_vec() {
        let quat = Quat::axis_angle(Vec3::up(), 7.1);
        let mat = Mat::rotation_y(7.1);
        let vec = Vec3::new(1., 2., 3.);

        let error = vec3_error(quat.to_mat() * vec , mat * vec);
        eprintln!("Error: {}", error);
        assert!(error < 0.0001);
    }

    #[test]
    fn mul_quat() {
        assert!(Quat::id() * Quat::id() == Quat::id());

        let m1 = Mat::rotation_y(1.0);
        let m2 = Mat::rotation_z(-7.0);

        let q1 = Quat::from_mat(m1);
        let q2 = Quat::from_mat(m2);

        let error = quat_error(
            Quat::from_mat(m1 * m2),
            q1 * q2,
        );

        assert!(error < 0.0001);
    }

    #[test]
    fn invert_quat() {
        assert!(Quat::id() * Quat::id().conjugate() == Quat::id());

        let q1 = Quat::axis_angle(Vec3::one(), 3.5);

        let error = quat_error(
            q1 * q1.conjugate(),
            Quat::id(),
        );

        assert!(error < 0.0001);

        let q2 = Quat::new(-1.0, -2.0, -3.0, -4.0).norm();
        let diff = q2 * q1.conjugate();

        let error = quat_error(
            diff * q1,
            q2,
        );

        assert!(error < 0.0001);
    }

    #[test]
    fn pow_quat() {
        let error = quat_error(
            Quat::id().pow(0.1),
            Quat::id(),
        );

        eprintln!("Error: {}", error);
        assert!(error < 0.0001);

        let q1 = Quat::new(-4.0, -3.0, -2.0, -1.0).norm();

        let error = quat_error(
            q1.pow(0.5) * q1.pow(0.5),
            q1,
        );

        eprintln!("Error: {}", error);
        assert!(error < 0.1); // TODO
    }

    #[test]
    fn quat_eq() {
        let q1 = Quat::new(-1.0, -2.0, -3.0, -4.0).norm();

        assert_eq!(q1, q1);

        let q2 = Quat::new(1.0, 2.0, 3.0, 4.0).norm();

        assert_eq!(q2, q2);
        assert_eq!(q1, q2);

        let q3 = Quat::new(4.0, 3.0, 2.0, 1.0).norm();

        assert_ne!(q1, q3);
        assert_ne!(q2, q3);
    }

    fn mat_error(a: Mat, b: Mat) -> f32 {
        let mut total = 0f32;

        {
            let mut error = |x: f32, y: f32| total += (x - y).abs();

            error(a.x0, b.x0);
            error(a.y0, b.y0);
            error(a.z0, b.z0);
            error(a.w0, b.w0);

            error(a.x1, b.x1);
            error(a.y1, b.y1);
            error(a.z1, b.z1);
            error(a.w1, b.w1);

            error(a.x2, b.x2);
            error(a.y2, b.y2);
            error(a.z2, b.z2);
            error(a.w2, b.w2);

            error(a.x3, b.x3);
            error(a.y3, b.y3);
            error(a.z3, b.z3);
            error(a.w3, b.w3);
        }

        total
    }

    fn vec3_error(a: Vec3, b: Vec3) -> f32 {
        let mut total = 0f32;

        {
            let mut error = |x: f32, y: f32| total += (x - y).abs();

            error(a.x, b.x);
            error(a.y, b.y);
            error(a.z, b.z);
        }

        total
    }

    fn quat_error(a: Quat, b: Quat) -> f32 {
        let mut total = 0f32;

        {
            // Two-norm
            let mut error = |x: f32, y: f32| total += (x - y) * (x - y);

            error(a.x, b.x);
            error(a.y, b.y);
            error(a.z, b.z);
            error(a.w, b.w);
        }

        total.sqrt()
    }

    #[test]
    fn norm_quat() {
        // Baseline
        let error = (
            Quat::id().norm().mag() - Quat::id().mag()
        ).abs();

        eprintln!("Error: {}", error);
        assert!(error < 0.0001);

        let quat = Quat::new(-1., 3., 5., 0.);
        let error = (quat.norm().mag() - 1.).abs();

        eprintln!("Error: {}", error);
        assert!(error < 0.0001);
    }

    #[test]
    fn norm_vec() {
        // Baseline
        let error = (Vec3::up().norm().mag() - Vec3::up().mag()).abs();

        eprintln!("Error: {}", error);
        assert!(error < 0.0001);

        let vec = Vec3::new(-1., 3., 5.);
        let error = (vec.norm().mag() - 1.).abs();

        eprintln!("Error: {}", error);
        assert!(error < 0.0001);
    }

    #[test]
    fn cross_vec() {
        assert!(Vec3::right().cross(Vec3::up()) == Vec3::fwd());
    }
}

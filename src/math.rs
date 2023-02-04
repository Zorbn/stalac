/*
pub struct Vec3 {
    values: [f32; 3],
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { values: [x, y, z] }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub fn cross(&self, other: &Vec3) -> Self {
        Vec3::new(
            self.values[1] * other.values[2] - self.values[2] * other.values[1],
            self.values[2] * other.values[0] - self.values[0] * other.values[2],
            self.values[0] * other.values[1] - self.values[1] * other.values[0],
        )
    }

    pub fn dot(&self, other: &Vec3) -> f32 {
        self.values[0] * other.values[0]
            + self.values[1] * other.values[1]
            + self.values[2] * other.values[2]
    }

    pub fn normalized(&self) -> Self {
        let magnitude = (self.values[0] * self.values[0]
            + self.values[1] * self.values[1]
            + self.values[2] * self.values[2])
            .sqrt();

        Vec3::new(
            self.values[0] / magnitude,
            self.values[1] / magnitude,
            self.values[2] / magnitude,
        )
    }
}

impl std::ops::Add<Vec3> for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Vec3) -> Self::Output {
        Vec3::new(
            self.values[0] + rhs.values[0],
            self.values[1] + rhs.values[1],
            self.values[2] + rhs.values[2],
        )
    }
}

impl std::ops::Add<&Vec3> for &Vec3 {
    type Output = Vec3;

    fn add(self, rhs: &Vec3) -> Self::Output {
        Vec3::new(
            self.values[0] + rhs.values[0],
            self.values[1] + rhs.values[1],
            self.values[2] + rhs.values[2],
        )
    }
}

impl std::ops::Sub<Vec3> for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Vec3) -> Self::Output {
        Vec3::new(
            self.values[0] - rhs.values[0],
            self.values[1] - rhs.values[1],
            self.values[2] - rhs.values[2],
        )
    }
}

impl std::ops::Sub<&Vec3> for &Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: &Vec3) -> Self::Output {
        Vec3::new(
            self.values[0] - rhs.values[0],
            self.values[1] - rhs.values[1],
            self.values[2] - rhs.values[2],
        )
    }
}

// impl std::ops::Neg for Vec3 {
//     type Output = Vec3;

//     fn neg(self) -> Self::Output {
//         Vec3::new(
//             -self.values[0],
//             -self.values[1],
//             -self.values[2],
//         )
//     }
// }

pub struct Mat4 {
    // [Column][Row]
    values: [[f32; 4]; 4],
}

impl Mat4 {
    pub fn zero() -> Self {
        Self {
            values: [
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
            ],
        }
    }

    pub fn look_at(eye: &Vec3, center: &Vec3, up: &Vec3) -> Self {
        let mut mat = Self::zero();

        let z = (eye - center).normalized();
        let x = up.cross(&z).normalized();
        let y = &z.cross(&x).normalized();

        mat.values[0][0] = x.values[0];
        mat.values[1][0] = x.values[1];
        mat.values[2][0] = x.values[2];
        mat.values[3][0] = -x.dot(eye);

        mat.values[0][1] = y.values[0];
        mat.values[1][1] = y.values[1];
        mat.values[2][1] = y.values[2];
        mat.values[3][1] = -y.dot(eye);

        mat.values[0][2] = z.values[0];
        mat.values[1][2] = z.values[1];
        mat.values[2][2] = z.values[2];
        mat.values[3][2] = -z.dot(eye);

        mat.values[0][3] = 0.0;
        mat.values[1][3] = 0.0;
        mat.values[2][3] = 0.0;
        mat.values[3][3] = 1.0;

        mat
    }

    pub fn perspective(fov_y: f32, aspect: f32, z_near: f32, z_far: f32) -> Self {
        let mut mat = Self::zero();

        let tan_half_fov_y = (fov_y * 0.5).tan();
        mat.values[0][0] = 1.0 / (aspect * tan_half_fov_y);
        mat.values[1][1] = 1.0 / tan_half_fov_y;
        mat.values[2][2] = -(z_far + z_near) / (z_far - z_near);
        mat.values[2][3] = -1.0;
        mat.values[3][2] = (2.0 * z_near * z_near) / (z_far - z_near);

        mat
    }
}
*/

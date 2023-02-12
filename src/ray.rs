pub struct Ray {
    pub position: cgmath::Vector3<f32>,
    pub dir: cgmath::Vector3<f32>,
}

impl Ray {
    pub fn intersects(&self, target_position: cgmath::Vector3<f32>, target_size: cgmath::Vector3<f32>) -> bool {
        self.intersection_distance(target_position, target_size).is_some()
    }

    pub fn intersection_point(&self, target_position: cgmath::Vector3<f32>, target_size: cgmath::Vector3<f32>) -> Option<cgmath::Vector3<f32>> {
        let dist = self.intersection_distance(target_position, target_size);

        dist.map(|d| d * self.dir + self.position)
    }

    // A ray will intersect a rect unless either:
    // - The ray passes the rect on the z axis before reaching it on the x axis,
    // - The ray passes the rect on the x axis before reaching it on the z axis,
    pub fn intersection_distance(&self, target_position: cgmath::Vector3<f32>, target_size: cgmath::Vector3<f32>) -> Option<f32> {
        let mut t_min = -f32::INFINITY;
        let mut t_max = f32::INFINITY;

        let target_min = target_position - target_size * 0.5;
        let target_max = target_position + target_size * 0.5;

        if self.dir.x != 0.0 {
            let tx1 = (target_min.x - self.position.x) / self.dir.x;
            let tx2 = (target_max.x - self.position.x) / self.dir.x;

            t_min = t_min.max(tx1.min(tx2));
            t_max = t_max.min(tx1.max(tx2));
        }

        if self.dir.z != 0.0 {
            let tz1 = (target_min.z - self.position.z) / self.dir.z;
            let tz2 = (target_max.z - self.position.z) / self.dir.z;

            t_min = t_min.max(tz1.min(tz2));
            t_max = t_max.min(tz1.max(tz2));
        }

        let hit = t_max >= 0.0 && t_max >= t_min;

        if hit {
            Some(t_min)
        } else {
            None
        }
    }
}
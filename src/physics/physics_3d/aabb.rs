use crate::math::Number;
extern crate test;

pub struct AabbTopLeftOrigin<T>
where
    T: Number,
{
    pub position: [T; 3],
    pub size: [T; 3],
}

impl<T> AabbTopLeftOrigin<T>
where
    T: Number,
{
    pub fn is_intersected_by_point(&self, point: [T; 3]) -> bool {
        point[0] < self.position[0] + self.size[0]
            && point[0] > self.position[0]
            && point[1] < self.position[1] + self.size[1]
            && point[1] > self.position[1]
            && point[2] < self.position[2] + self.size[2]
            && point[2] > self.position[2]
    }

    pub fn is_intersected_by_aabb(&self, aabb: AabbTopLeftOrigin<T>) -> bool {
        self.position[0] < aabb.position[0] + aabb.size[0]
            && self.position[0] + self.size[0] > aabb.position[0]
            && self.position[1] < aabb.position[1] + aabb.size[1]
            && self.position[1] + self.size[1] > aabb.position[1]
            && self.position[2] < aabb.position[2] + aabb.size[2]
            && self.position[2] + self.size[2] > aabb.position[2]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct AabbCentredOrigin<T>
where
    T: Number,
{
    pub position: [T; 3],
    pub half_size: [T; 3],
}

impl<T> AabbCentredOrigin<T>
where
    T: Number,
{
    pub fn is_intersected_by_point(&self, point: [T; 3]) -> bool {
        if (self.position[0] - point[0]).abs() > self.half_size[0] {
            return false;
        }
        if (self.position[1] - point[1]).abs() > self.half_size[1] {
            return false;
        }
        if (self.position[2] - point[2]).abs() > self.half_size[2] {
            return false;
        }
        true
    }

    pub fn is_intersected_by_aabb(&self, aabb: AabbCentredOrigin<T>) -> bool {
        if (self.position[0] - aabb.position[0]).abs() > self.half_size[0] + aabb.half_size[0] {
            return false;
        }
        if (self.position[1] - aabb.position[1]).abs() > self.half_size[1] + aabb.half_size[1] {
            return false;
        }
        if (self.position[2] - aabb.position[2]).abs() > self.half_size[2] + aabb.half_size[2] {
            return false;
        }
        true
    }

    pub fn get_collision_axis(&self, other: AabbCentredOrigin<T>) -> [bool; 3] {
        // Run this on previous position instead, so you can see what axis wasn't intersecting before the collision.
        [
            (self.position[0] - other.position[0]).abs() > self.half_size[0] + other.half_size[0],
            (self.position[1] - other.position[1]).abs() > self.half_size[1] + other.half_size[1],
            (self.position[2] - other.position[2]).abs() > self.half_size[2] + other.half_size[2],
        ]
    }
}

pub struct AabbMinMax<T>
where
    T: Number,
{
    pub min: [T; 3],
    pub max: [T; 3],
}

impl<T> AabbMinMax<T>
where
    T: Number,
{
    pub fn is_intersected_by_point(&self, point: [T; 3]) -> bool {
        point[0] >= self.min[0]
            && point[0] <= self.max[0]
            && point[1] >= self.min[1]
            && point[1] <= self.max[1]
            && point[2] >= self.min[2]
            && point[2] <= self.max[2]
    }

    pub fn is_intersected_by_aabb(&self, aabb: AabbMinMax<T>) -> bool {
        self.min[0] <= aabb.max[0]
            && self.max[0] >= aabb.min[0]
            && self.min[1] <= aabb.max[1]
            && self.max[1] >= aabb.min[1]
            && self.min[2] <= aabb.max[2]
            && self.max[2] >= aabb.min[2]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_aabb_top_left_origin_is_intersected_by_point(b: &mut Bencher) {
        b.iter(|| {
            let aabb = test::black_box(AabbTopLeftOrigin {
                position: [4.0954, 7.823, 2.2389],
                size: [2.0, 3.0, 8.78],
            });

            return aabb.is_intersected_by_point([5.0, 6.3, 2.1]);
        })
    }

    #[bench]
    fn bench_aabb_top_left_origin_is_intersected_by_aabb(b: &mut Bencher) {
        b.iter(|| {
            let aabb1 = test::black_box(AabbTopLeftOrigin {
                position: [3.0, 2.0, 2.2389],
                size: [10.0, 5.0, 8.78],
            });

            let aabb2 = test::black_box(AabbTopLeftOrigin {
                position: [3.0, 2.0, 3.2389],
                size: [10.0, 5.0, 2.8],
            });

            return aabb1.is_intersected_by_aabb(aabb2);
        })
    }

    #[bench]
    fn bench_aabb_centred_origin_is_intersected_by_point(b: &mut Bencher) {
        b.iter(|| {
            let aabb = test::black_box(AabbCentredOrigin {
                position: [4.0954, 7.823, 2.2389],
                half_size: [2.0, 3.0, 5.5],
            });

            return aabb.is_intersected_by_point([5.0, 6.3, 8.88]);
        })
    }

    #[bench]
    fn bench_aabb_centred_origin_is_intersected_by_aabb(b: &mut Bencher) {
        b.iter(|| {
            let aabb1 = test::black_box(AabbCentredOrigin {
                position: [3.0, 2.0, 2.2389],
                half_size: [10.0, 5.0, 7.1],
            });

            let aabb2 = test::black_box(AabbCentredOrigin {
                position: [3.0, 2.0, 23.1],
                half_size: [10.0, 5.0, 2.2389],
            });

            return aabb1.is_intersected_by_aabb(aabb2);
        })
    }

    #[bench]
    fn bench_aabb_min_max_is_intersected_by_point(b: &mut Bencher) {
        b.iter(|| {
            let aabb = test::black_box(AabbMinMax {
                min: [3.0, 2.0, 3.33],
                max: [10.0, 5.0, 4.01],
            });

            return aabb.is_intersected_by_point([5.0, 6.3, 2.2389]);
        })
    }

    #[bench]
    fn bench_aabb_min_max_is_intersected_by_aabb(b: &mut Bencher) {
        b.iter(|| {
            let aabb1 = test::black_box(AabbMinMax {
                min: [3.0, 2.0, 4.5],
                max: [10.0, 5.0, 30.0],
            });

            let aabb2 = test::black_box(AabbMinMax {
                min: [3.0, 2.0, 5.8],
                max: [10.0, 5.0, 22.22],
            });

            return aabb1.is_intersected_by_aabb(aabb2);
        })
    }
}
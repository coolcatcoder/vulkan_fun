use std::mem::MaybeUninit;

use crate::math;

use super::bodies::Body;

use rayon::prelude::*;

extern crate test;

/// A generic solver capable of handling most basic physics simulations.
pub struct CpuSolver<T, B>
where
    T: math::Float,
    B: Body<T>,
{
    pub gravity: [T; 3],
    pub dampening: [T; 3], // Where 1.0 is no dampening. Perhaps displacement_kept is a better name?
    pub bodies: Vec<B>,
    pub grid_size: [usize; 3], // This is in cell size units. This should probably be clarified.
    pub cell_size: [usize; 3], // TODO: asap work out how the usize vs isize nonsense will work, as we want this to work for negatives. Perhaps we can plus some sort of offset for the particle?
    pub grid_origin: [T; 3], // Remember that the origin is the bottom left corner of the grid, I think.
    pub grid: Vec<Vec<usize>>,
    pub outside_of_grid_bounds_behaviour: OutsideOfGridBoundsBehaviour<T>,
}

impl<T, B> CpuSolver<T, B>
where
    T: math::Float,
    B: Body<T>,
{
    pub fn new(
        gravity: [T; 3],
        dampening: [T; 3],
        grid_size: [usize; 3],
        grid_origin: [T; 3],
        cell_size: [usize; 3],
        outside_of_grid_bounds_behaviour: OutsideOfGridBoundsBehaviour<T>,
        bodies: Vec<B>,
    ) -> CpuSolver<T, B> {
        CpuSolver {
            gravity,
            dampening,
            bodies,
            grid_size,
            cell_size,
            grid_origin,
            grid: vec![vec![]; grid_size[0] * grid_size[1] * grid_size[2]],
            outside_of_grid_bounds_behaviour,
        }
    }

    pub fn update(&mut self, delta_time: T) {
        let real_grid_width = self.grid_size[0] * self.cell_size[0];
        let real_grid_height = self.grid_size[1] * self.cell_size[1];
        let real_grid_length = self.grid_size[2] * self.cell_size[2];

        self.bodies.par_iter_mut().for_each(|body| {
            if body.is_none() {
                return;
            }

            body.update(self.gravity, self.dampening, delta_time);
        });

        for (verlet_body_index, verlet_body) in self.bodies.iter_mut().enumerate() {
            if verlet_body.is_none() {
                continue;
            }

            let verlet_body_position = verlet_body.position_unchecked();

            let corrected_position = [
                verlet_body_position[0] - self.grid_origin[0], // + or - ????
                verlet_body_position[1] - self.grid_origin[1],
                verlet_body_position[2] - self.grid_origin[2],
            ];

            let corrected_position_as_isize = [
                corrected_position[0].to_isize(),
                corrected_position[1].to_isize(),
                corrected_position[2].to_isize(),
            ];

            let corrected_position_as_usize = [
                corrected_position[0].to_usize(),
                corrected_position[1].to_usize(),
                corrected_position[2].to_usize(),
            ];

            let outside_side = [
                corrected_position_as_isize[0] as usize > real_grid_width - 1,
                corrected_position_as_isize[0] < 0,
                corrected_position_as_isize[1] as usize > real_grid_height - 1,
                corrected_position_as_isize[1] < 0,
                corrected_position_as_isize[2] as usize > real_grid_length - 1,
                corrected_position_as_isize[2] < 0,
            ];

            if outside_side[0]
                || outside_side[1]
                || outside_side[2]
                || outside_side[3]
                || outside_side[4]
                || outside_side[5]
            {
                //println!("corrected position: {:?}", corrected_position); // Very useful debug!
                // Perhaps have this per verlet body?
                match self.outside_of_grid_bounds_behaviour {
                    OutsideOfGridBoundsBehaviour::SwapDeleteParticle => {
                        todo!();
                        //self.particles.swap_remove(particle_index);
                        //continue;
                    }
                    OutsideOfGridBoundsBehaviour::DeleteParticle => {
                        todo!();
                        //self.particles.remove(particle_index);
                        //continue;
                    }
                    OutsideOfGridBoundsBehaviour::PutParticleInBounds => {
                        todo!();
                    }
                    OutsideOfGridBoundsBehaviour::TeleportParticleToPosition(_position) => {
                        todo!()
                        //particle.previous_position = position;
                        //particle.position = position;
                    }
                    OutsideOfGridBoundsBehaviour::ContinueUpdating => {
                        continue;
                    }
                }
            }

            let grid_cell_position = [
                corrected_position_as_usize[0] / self.cell_size[0],
                corrected_position_as_usize[1] / self.cell_size[1],
                corrected_position_as_usize[2] / self.cell_size[2],
            ];

            let grid_cell_index = math::index_from_position_3d(
                grid_cell_position,
                self.grid_size[0],
                self.grid_size[1],
            );

            // If something is wrong, this debug information is usually helpful.
            self.grid
                .get_mut(grid_cell_index)
                .unwrap_or_else(|| {
                    println!("verlet_body_position: {:?}", verlet_body_position);
                    println!(
                        "corrected_position_as_isize: {:?}",
                        corrected_position_as_isize
                    );
                    println!("grid_cell_position: {:?}", grid_cell_position);
                    panic!()
                })
                .push(verlet_body_index);
        }

        self.collide_bodies(delta_time);

        for cell in &mut self.grid {
            if cell.capacity() == 0 {
                continue;
            }
            // This is meant to keep memory usage low, with only a minor performance cost, but I'm not convinced.
            // Even though we check for 0, this still seems dodgy. Perhaps this should be a choice for the user.
            if cell.len() <= cell.capacity() / 2 {
                //println!("len: {}, capacity: {}", cell.len(), cell.capacity());
                cell.shrink_to_fit();
            }
            cell.clear();
        }
    }

    #[inline]
    fn collide_bodies(&mut self, delta_time: T) {
        // TODO: How the hell can we multithread this?
        // TODO: Consider having substeps that affect only collision?
        for cell_index in 0..self.grid.len() {
            let cell = &self.grid[cell_index];
            let cell_position =
                math::position_from_index_3d(cell_index, self.grid_size[0], self.grid_size[1]);

            // Debating how much I like performance. I don't want to write by hand 26 different cell lets. This will do:
            // This code seems dodgy, but I reckon it is better than using a vec. Gotten from: https://users.rust-lang.org/t/uninitialized-array/50278/3
            // len of 27 because this includes the center cell
            let mut neighbours: [MaybeUninit<&Vec<usize>>; 27] =
                unsafe { MaybeUninit::uninit().assume_init() };
            let mut neighbours_len: u8 = 0;

            for x in -1..=1 {
                for y in -1..=1 {
                    for z in -1..=1 {
                        let position = [
                            cell_position[0] as isize + x,
                            cell_position[1] as isize + y,
                            cell_position[2] as isize + z,
                        ];

                        if position[0] >= 0
                            && position[0] < self.grid_size[0] as isize
                            && position[1] >= 0
                            && position[1] < self.grid_size[1] as isize
                            && position[2] >= 0
                            && position[2] < self.grid_size[2] as isize
                        {
                            neighbours[neighbours_len as usize] = MaybeUninit::new(
                                &self.grid[math::index_from_position_3d(
                                    [
                                        position[0] as usize,
                                        position[1] as usize,
                                        position[2] as usize,
                                    ],
                                    self.grid_size[0],
                                    self.grid_size[1],
                                )],
                            );

                            neighbours_len += 1;
                        }
                    }
                }
            }

            // Leaving this here as a lesson. For edge cells there will be less than 27 neighbours, as such it must remain maybe uninit.
            // let neighbours = unsafe {
            //     mem::transmute::<_, [&Vec<usize>; 27]>(neighbours)
            // };

            for lhs_verlet_body_index in cell {
                if !self.bodies[*lhs_verlet_body_index].collide_with_others() {
                    continue;
                }

                for neighbour_index in 0..neighbours_len {
                    // we are certain that assume_init() is safe due to iterating from 0 to neighbours_len.
                    let neighbour = unsafe { &neighbours[neighbour_index as usize].assume_init() };
                    for rhs_verlet_body_index in *neighbour {
                        if lhs_verlet_body_index == rhs_verlet_body_index {
                            continue;
                        }

                        // This code is simple and elgant. By splitting at the largest index, it allows us to safely and &mutably yoink the verlet bodies.
                        if lhs_verlet_body_index > rhs_verlet_body_index {
                            let (lhs_verlet_bodies, rhs_verlet_bodies) =
                                self.bodies.split_at_mut(*lhs_verlet_body_index);
                            rhs_verlet_bodies[0].collide(
                                &mut lhs_verlet_bodies[*rhs_verlet_body_index],
                                *rhs_verlet_body_index,
                                delta_time,
                            );
                        } else {
                            let (lhs_verlet_bodies, rhs_verlet_bodies) =
                                self.bodies.split_at_mut(*rhs_verlet_body_index);
                            lhs_verlet_bodies[*lhs_verlet_body_index].collide(
                                &mut rhs_verlet_bodies[0],
                                *rhs_verlet_body_index,
                                delta_time,
                            );
                        }
                    }
                }

                // cell is already part of the neighbours, so we don't have to worry about it.
            }
        }
    }
}

/// If a body is outside of the grid, what should it do?
pub enum OutsideOfGridBoundsBehaviour<T: math::Number> {
    SwapDeleteParticle,
    DeleteParticle,
    PutParticleInBounds,
    TeleportParticleToPosition([T; 3]),
    ContinueUpdating,
    // replace with body?
}

#[cfg(test)]
mod tests {
    use crate::physics::physics_3d::bodies::CommonBody;
    use crate::physics::physics_3d::bodies::Cuboid;
    use crate::physics::physics_3d::verlet::Particle;

    use super::*;
    use rand::thread_rng;
    use rand::Rng;
    use test::Bencher;

    #[bench]
    fn bench_cpu_solver_50000_particles(b: &mut Bencher) {
        let amount = 50000;
        let mut verlet_bodies = Vec::with_capacity(amount);
        let mut rng = thread_rng();

        for _ in 0..amount {
            verlet_bodies.push(CommonBody::Cuboid(Cuboid {
                particle: Particle::from_position([
                    rng.gen_range(-50.0..50.0),
                    rng.gen_range(-50.0..50.0),
                    rng.gen_range(-50.0..50.0),
                ]),

                half_size: [0.5, 0.5, 0.5],
            }));
        }

        let mut solver = CpuSolver::new(
            [0.0, 50.0, 0.0],
            [0.8, 1.0, 0.8],
            [10, 10, 10],
            [-50.0, -50.0, -50.0],
            [10, 10, 10],
            OutsideOfGridBoundsBehaviour::ContinueUpdating,
            verlet_bodies,
        );
        b.iter(|| {
            solver.update(0.04);
        })
    }

    #[bench]
    fn bench_cpu_solver_1000_none_particles(b: &mut Bencher) {
        let mut verlet_bodies = Vec::with_capacity(1000);

        for _ in 0..1000 {
            verlet_bodies.push(CommonBody::None);
        }

        let mut solver = CpuSolver::new(
            [0.0, 50.0, 0.0],
            [0.8, 1.0, 0.8],
            [10, 10, 10],
            [-50.0, -50.0, -50.0],
            [10, 10, 10],
            OutsideOfGridBoundsBehaviour::ContinueUpdating,
            verlet_bodies,
        );
        b.iter(|| {
            solver.update(0.04);
        })
    }
}

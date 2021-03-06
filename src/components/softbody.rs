use alg;
use entity;
use render;
use graphics;
use components;
use debug;

use std;

use ::FIXED_DT; // Import from lib
use components::transform;

// Constraint solver iterations
const ITERATIONS: usize = 1;

// Range 0 - inf; "Realistic" = 2.0
// Values < 2 become force zones, values > 2 add impossible force
// A value of zero nullifies all collisions
const BOUNCE: f32 = 0.05;

// Range 0 - 1; 1.0 = cannot be deformed
// A value of zero nullifies all rods in the instance
const DEFORM: f32 = 1.000;

// Range 0 - 0.499; "Rigid" = 0.499
// Lower values produce springier joints
// A value of zero nullifies the translational constraints of all joints
const JOINT_POS_RIGID: f32 = 0.499;

// Range 0 - 0.5; "Rigid" = 0.5
// Lower values produce springier joints
// A value of zero nullifies the angular constraints of all joints
const JOINT_ANG_RIGID: f32 = 0.5;

struct Particle {
    position: alg::Vec3,
    last: alg::Vec3,
}

impl Particle {
    fn new(position: alg::Vec3) -> Particle {
        Particle {
            position: position,
            last: position,
        }
    }
}

struct Rod {
    left: usize,
    right: usize,
    length: f32,
}

impl Rod {
    fn new(left: usize, right: usize, particles: &[Particle]) -> Rod {
        debug_assert!(left < particles.len());
        debug_assert!(right < particles.len());

        let length = alg::Vec3::dist(
            particles[left].position,
            particles[right].position,
        );

        Rod {
            left,
            right,
            length,
        }
    }
}

type Falloff = fn(alg::Vec3, alg::Vec3) -> alg::Vec3;

struct Magnet {
    target: alg::Vec3,
    serf: usize,
    falloff: Falloff,
}

impl Magnet {
    fn new(serf: usize, falloff: Falloff) -> Magnet {
        Magnet {
            target: alg::Vec3::zero(),
            serf: serf,
            falloff: falloff,
        }
    }
}

struct Range {
    min: f32,
    max: f32,
}

struct Joint {
    parent:  usize,
    child:   usize,
    x_limit: Range,
    y_limit: Range,
    z_limit: Range,
}

impl Joint {
    fn new(
        parent:  usize,
        child:   usize,
        x_limit: Range,
        y_limit: Range,
        z_limit: Range,
    ) -> Joint {
        Joint {
            parent,
            child,
            x_limit,
            y_limit,
            z_limit,
        }
    }
}

#[allow(dead_code)]
struct Instance {
    particles: Vec<Particle>,
    rods: Vec<Rod>,
    magnets: Vec<Magnet>,

    force: alg::Vec3,
    accel_dt: alg::Vec3, // Cached value, dependent on force
    position: alg::Vec3, // Updated every frame

    /* "Constants" */

    mass: f32,
    model: Vec<alg::Vec3>, // Vertices reference

    // Range 0 - 0.5; "Rigid" = 0.5
    // Lower values produce springier meshes
    // A value of zero nullifies all rods in the instance
    rigidity: f32,
}

impl Instance {
    fn new(
        points: &[alg::Vec3],
        bindings: &[(usize, usize)],
        zones: &[(usize, Falloff)],
        mass: f32,
        rigidity: f32,
        gravity: alg::Vec3,
    ) -> Instance {
        debug_assert!(mass > 0.0);
        debug_assert!(rigidity > 0.0 && rigidity <= 0.5);

        let mut particles = Vec::with_capacity(points.len());
        let mut model = Vec::with_capacity(points.len());

        for point in points {
            particles.push(Particle::new(*point));
            model.push(*point);
        }

        let mut rods = Vec::with_capacity(bindings.len());
        for binding in bindings {
            rods.push(Rod::new(binding.0, binding.1, &particles));
        }

        let mut magnets = Vec::with_capacity(zones.len());
        for zone in zones {
            magnets.push(Magnet::new(zone.0, zone.1));
        }

        Instance {
            particles: particles,
            rods: rods,
            magnets: magnets,

            force: alg::Vec3::zero(),
            accel_dt: gravity * FIXED_DT * FIXED_DT,
            position: alg::Vec3::zero(),

            mass: mass,
            rigidity: rigidity,
            model,
        }
    }

    // Get offset from center for specific particle
    fn offset(&self, index: usize) -> alg::Vec3 {
        self.particles[index].position - self.position - self.model[index]
    }

    #[inline]
    // Must be called when gravity or force changes
    fn update_cache(&mut self, gravity: alg::Vec3) {
        self.accel_dt = ((self.force / self.mass) + gravity)
            * FIXED_DT * FIXED_DT;
    }

    /* Limb helper methods */

    #[inline]
    fn start(&self) -> alg::Vec3 {
        (     self.particles[0].position
            + self.particles[1].position
            + self.particles[2].position
            + self.particles[3].position
        ) * 0.25
    }

    #[inline]
    fn end(&self) -> alg::Vec3 {
        (     self.particles[4].position
            + self.particles[5].position
            + self.particles[6].position
            + self.particles[7].position
        ) * 0.25
    }

    #[inline]
    fn top(&self) -> alg::Vec3 {
        (     self.particles[0].position
            + self.particles[1].position
            + self.particles[4].position
            + self.particles[5].position
        ) * 0.25
    }

    #[inline]
    fn bot(&self) -> alg::Vec3 {
        (     self.particles[2].position
            + self.particles[3].position
            + self.particles[6].position
            + self.particles[7].position
        ) * 0.25
    }

    #[inline]
    fn right_side(&self) -> alg::Vec3 {
        (     self.particles[1].position
            + self.particles[2].position
            + self.particles[5].position
            + self.particles[6].position
        ) * 0.25
    }

    #[inline]
    fn left_side(&self) -> alg::Vec3 {
        (     self.particles[0].position
            + self.particles[3].position
            + self.particles[4].position
            + self.particles[7].position
        ) * 0.25
    }

    #[inline]
    fn fwd(&self) -> alg::Vec3 {
        (self.end() - self.start()).norm()
    }

    #[inline]
    fn up(&self) -> alg::Vec3 {
        (self.top() - self.bot()).norm()
    }

    #[inline]
    fn right(&self) -> alg::Vec3 {
        (self.right_side() - self.left_side()).norm()
    }

    // Get limb orientation as matrix
    fn orientation(&self) -> alg::Mat {
        // Build orthogonal rotation matrix
        let fwd = self.fwd();
        let up = self.up(); // Approximation
        let right = up.cross(fwd); // Resistant to x-axis deformity
        let up = fwd.cross(right); // Recreate up vector

        alg::Mat::axes(right, up, fwd)
    }

    // Get inverse limb orientation as matrix
    fn inverse_orientation(&self) -> alg::Mat {
        // Build orthogonal rotation matrix
        let fwd = self.fwd();
        let up = self.up(); // Approximation
        let right = up.cross(fwd); // Resistant to x-axis deformity
        let up = fwd.cross(right); // Recreate up vector

        alg::Mat::inverse_axes(right, up, fwd)
    }

    fn rotate_start(&mut self, x: f32, y: f32, z: f32) {
        let point = self.start();
        self.rotate_around(point, x, y, z);
    }

    fn rotate_end(&mut self, x: f32, y: f32, z: f32) {
        let point = self.end();
        self.rotate_around(point, x, y, z);
    }

    #[inline]
    fn rotate_around(&mut self, point: alg::Vec3, x: f32, y: f32, z: f32) {
        // Center axis of rotation
        for i in 0..8 {
            self.particles[i].position = self.particles[i].position - point;
        }

        let rx = alg::Mat::rotation_x(x);
        for i in 0..8 {
            self.particles[i].position = rx * self.particles[i].position;
        }

        let ry = alg::Mat::rotation_y(y);
        for i in 0..8 {
            self.particles[i].position = ry * self.particles[i].position;
        }

        let rz = alg::Mat::rotation_z(z);
        for i in 0..8 {
            self.particles[i].position = rz * self.particles[i].position;
        }

        // Move back to world space
        for i in 0..8 {
            self.particles[i].position = self.particles[i].position + point;
        }
    }

    #[inline]
    fn transform_around(&mut self, point: alg::Vec3, transform: alg::Mat) {
        // Center axis
        for i in 0..8 {
            self.particles[i].position = self.particles[i].position - point;
        }

        for i in 0..8 {
            self.particles[i].position = transform
                * self.particles[i].position;
        }

        // Move back to worldspace
        for i in 0..8 {
            self.particles[i].position = self.particles[i].position + point;
        }
    }
}

#[derive(Clone, Copy)]
struct ReachPlane {
    normal: alg::Vec3,
}

// Specialized plane struct for joint constraints
impl ReachPlane {
    fn new(left: alg::Vec3, right: alg::Vec3) -> ReachPlane {
        ReachPlane {
            normal: left.cross(right),
        }
    }

    #[inline]
    fn contains(self, point: alg::Vec3) -> bool {
        // Magic number found through trial and error
        self.normal.dot(point) >= 0.0 - 0.0001
    }

    #[inline]
    fn intersects(self, ray: alg::Vec3) -> bool {
        self.normal.dot(alg::Vec3::fwd() + ray) < 0.0
    }

    #[inline]
    fn intersection(self, ray: alg::Vec3) -> alg::Vec3 {
        let div = self.normal.dot(ray) + std::f32::EPSILON;
        let scalar = (-alg::Vec3::fwd()).dot(self.normal) / div;

        ray * scalar
    }
}

// Data layout assumes many physics objects (but may still be sparse)
pub struct Manager {
    instances: Vec<Option<Instance>>,
    joints: Vec<Joint>,
    planes: Vec<alg::Plane>,
    gravity: alg::Vec3,
}

impl components::Component for Manager {
    fn register(&mut self, entity: entity::Handle) {
        let i = entity.get_index() as usize;

        // Resize array to fit new entity
        loop {
            if i >= self.instances.len() {
                self.instances.push(None);
                continue;
            }

            break;
        }
    }

    // TODO: This currently only returns the length of the underlying data
    // structure, not the count of the registered entities
    fn count(&self) -> usize {
        self.instances.len()
    }
}

impl Manager {
    pub fn new(
        instance_hint: usize,
        joint_hint: usize,
        plane_hint: usize,
    ) -> Manager {
        Manager {
            instances: Vec::with_capacity(instance_hint),
            joints: Vec::with_capacity(joint_hint),
            planes: Vec::with_capacity(plane_hint),
            gravity: alg::Vec3::new(0., -9.8, 0.),
        }
    }

    pub fn init_instance(
        &mut self,
        entity: entity::Handle,
        mass: f32,
        rigidity: f32, // Expects a value between 0-1
        points: &[alg::Vec3],
        bindings: &[(usize, usize)],
        magnets: &[(usize, Falloff)],
    ) {
        let i = entity.get_index() as usize;
        debug_assert!(i < self.instances.len());

        self.instances[i] = Some(
            Instance::new(
                points,
                bindings,
                magnets,
                mass,
                rigidity * 0.5, // Scale rigidity properly
                self.gravity,
            )
        );
    }

    pub fn init_limb(
        &mut self,
        entity: entity::Handle,
        mass: f32,
        rigidity: f32, // Expects a value between 0-1
        scale: alg::Vec3,
    ) {
        let i = entity.get_index() as usize;
        debug_assert!(i < self.instances.len());

        let scale = scale * 0.5;

        self.instances[i] = Some(
            Instance::new(
                &[
                    // Front face
                    alg::Vec3::new(-scale.x,  scale.y, -scale.z), // 0
                    alg::Vec3::new( scale.x,  scale.y, -scale.z), // 1
                    alg::Vec3::new( scale.x, -scale.y, -scale.z), // 2
                    alg::Vec3::new(-scale.x, -scale.y, -scale.z), // 3

                    // Back face
                    alg::Vec3::new(-scale.x,  scale.y,  scale.z), // 4
                    alg::Vec3::new( scale.x,  scale.y,  scale.z), // 5
                    alg::Vec3::new( scale.x, -scale.y,  scale.z), // 6
                    alg::Vec3::new(-scale.x, -scale.y,  scale.z), // 7
                ],
                &[
                    // Front face
                    (0, 1), (1, 2), (2, 3), (3, 0),
                    // Back face
                    (4, 5), (5, 6), (6, 7), (7, 4),
                    // Connectors
                    (0, 4), (1, 5), (2, 6), (3, 7),
                    // Crosspieces
                    (0, 2), (0, 5), (0, 7),
                    (6, 4), (6, 1), (6, 3),
                ],
                &[],
                mass,
                rigidity * 0.5, // Scale rigidity properly
                self.gravity,
            )
        );
    }

    pub fn set_force(&mut self, entity: entity::Handle, force: alg::Vec3) {
        let i = entity.get_index() as usize;
        debug_assert!(i < self.instances.len());

        if let Some(ref mut instance) = self.instances[i] {
            instance.force = force;
            instance.update_cache(self.gravity);
        }
    }

    pub fn set_magnet(
        &mut self,
        entity: entity::Handle,
        index: usize,
        target: alg::Vec3,
    ) {
        let i = entity.get_index() as usize;
        debug_assert!(i < self.instances.len());

        if let Some(ref mut instance) = self.instances[i] {
            debug_assert!(index < instance.magnets.len());

            instance.magnets[index].target = target;
        }
    }

    pub fn get_particle(
        &self,
        entity: entity::Handle,
        index: usize,
    ) -> alg::Vec3 {
        let i = entity.get_index() as usize;
        debug_assert!(i < self.instances.len());

        let mut position = alg::Vec3::zero();

        if let Some(ref instance) = self.instances[i] {
            debug_assert!(index < instance.particles.len());
            position = instance.particles[index].position;
        }

        position
    }

    pub fn get_offsets(
        &self,
        entity: entity::Handle,
    ) -> [render::PaddedVec3; render::MAX_SOFTBODY_VERT] {
        let i = entity.get_index() as usize;
        debug_assert!(i < self.instances.len());

        // Default to no offsets (identity)
        let mut offsets = [
            render::PaddedVec3::default();
            render::MAX_SOFTBODY_VERT
        ];

        // Space has not been allocated for this component (does not exist)
        if i >= self.instances.len() {
            return offsets;
        }

        // If the entity has a softbody component, fill the offsets array
        if let Some(ref instance) = self.instances[i] {
            for i in 0..instance.particles.len() {
                offsets[i] = render::PaddedVec3::new(instance.offset(i));
            }
        }

        offsets
    }

    pub fn add_joint(
        &mut self,
        parent: entity::Handle,
        child: entity::Handle,
        x_limit: (f32, f32), // Degrees
        y_limit: (f32, f32), // Degrees
        z_limit: (f32, f32), // Degrees
    ) {
        let (i, j) = (
            child.get_index() as usize,
            parent.get_index() as usize,
        );

        debug_assert!(i < self.instances.len());
        debug_assert!(j < self.instances.len());

        if self.instances[i].is_none() { return; }
        if self.instances[j].is_none() { return; }

        let x_min = x_limit.0.to_radians();
        let x_max = x_limit.1.to_radians();

        let y_min = y_limit.0.to_radians();
        let y_max = y_limit.1.to_radians();

        let z_min = z_limit.0.to_radians();
        let z_max = z_limit.1.to_radians();

        let joint = Joint::new(
            i, j,
            Range { min: x_min, max: x_max },
            Range { min: y_min, max: y_max },
            Range { min: z_min, max: z_max },
        );

        self.joints.push(joint);
    }

    pub fn add_plane(&mut self, plane: alg::Plane) {
        self.planes.push(plane);
    }

    // Heavier call, but will force-update all instances
    pub fn set_gravity(&mut self, gravity: alg::Vec3) {
        self.gravity = gravity;

        for i in 0..self.instances.len() {
            if let Some(ref mut instance) = self.instances[i] {
                instance.update_cache(self.gravity);
            }
        }
    }

    // May not immediately affect all instances
    pub fn set_gravity_raw(&mut self, gravity: alg::Vec3) {
        self.gravity = gravity;
    }

    pub fn simulate(&mut self, transforms: &mut transform::Manager) {
        // Update instances
        for i in 0..self.instances.len() {
            let mut instance = match self.instances[i] {
                Some(ref mut instance) => instance,
                None => continue,
            };

            // Update particles in instance
            for particle in &mut instance.particles {
                // Position Verlet
                let target = particle.position * 2. - particle.last;
                particle.last = particle.position;
                particle.position = target + instance.accel_dt;
            }

            // Solve constraints
            for _ in 0..ITERATIONS {
                // Rods
                for rod in &instance.rods {
                    let left = instance.particles[rod.left].position;
                    let right = instance.particles[rod.right].position;

                    let difference = right - left;
                    let distance = difference.mag();

                    let offset = difference * instance.rigidity
                        * (rod.length / distance - 1.);

                    instance.particles[rod.left].position = left - offset;
                    instance.particles[rod.right].position = right + offset;
                }

                // Planes
                for plane in &self.planes {
                    for particle in &mut instance.particles {
                        let distance = plane.normal.dot(particle.position)
                            + plane.offset;

                        if distance > 0. {
                            continue;
                        }

                        particle.position = particle.position
                            - plane.normal * BOUNCE * distance;
                    }
                }

                // Deformity
                for rod in &mut instance.rods {
                    let left = instance.particles[rod.left].position;
                    let right = instance.particles[rod.right].position;

                    rod.length = f32::min(
                        f32::max(left.dist(right), rod.length * DEFORM),
                        rod.length,
                    );
                }

                // Magnets
                for magnet in &instance.magnets {
                    let serf = &mut instance.particles[magnet.serf];

                    serf.position = (magnet.falloff)(
                        serf.position,
                        magnet.target,
                    );
                }
            }
        }

        // Solve joint constraints
        for joint in &self.joints {
            debug_assert!(joint.parent != joint.child);

            /* Unsafely acquire mutable references to vector elements.
             * Unfortunately, the only safe Rust alternative (split_at_mut())
             * is slower.
             */

            let mut parent = unsafe {
                let ptr = self.instances.as_mut_ptr()
                    .offset(joint.parent as isize);

                (*ptr).as_mut().unwrap()
            };

            let mut child = unsafe {
                let ptr = self.instances.as_mut_ptr()
                    .offset(joint.child as isize);

                (*ptr).as_mut().unwrap()
            };

            /* Constrain positions */

            let offset = (child.start() - parent.end()) * -JOINT_POS_RIGID;

            for i in 0..8 {
                // Correct parent
                let new_position = parent.particles[i].position - offset;
                parent.particles[i].position = new_position;
            }

            for i in 0..8 {
                // Correct child
                let new_position = child.particles[i].position + offset;
                child.particles[i].position = new_position;
            }

            /* Constrain rotations */

            // Get child in local space
            let child_orient = child.orientation();
            let local_child = alg::Quat::from_mat(
                parent.inverse_orientation() * child_orient,
            );

            let local_child_fwd = local_child * alg::Vec3::fwd();

            // Compute simple and twist rotations
            let simple = alg::Quat::simple(alg::Vec3::fwd(), local_child_fwd);
            let twist = local_child * simple.conjugate();

            // Build cone
            let (lower, right, upper, left) = {
                let x_min = joint.y_limit.min / 90f32.to_radians();
                let x_max = joint.y_limit.max / 90f32.to_radians();
                let y_min = joint.x_limit.min / 90f32.to_radians();
                let y_max = joint.x_limit.max / 90f32.to_radians();

                (
                    alg::Vec3::new(0.0, y_min, 1.0 - y_min).norm(),
                    alg::Vec3::new(x_max, 0.0, 1.0 - x_max).norm(),
                    alg::Vec3::new(0.0, y_max, 1.0 - y_max).norm(),
                    alg::Vec3::new(x_min, 0.0, 1.0 - x_min).norm(),
                )
            };

            // Build planes
            let lower_left = ReachPlane::new(left, lower);
            let lower_right = ReachPlane::new(lower, right);
            let upper_right = ReachPlane::new(right, upper);
            let upper_left = ReachPlane::new(upper, left);

            // Is the rotation inside the cone?
            let inside = lower_left.contains(local_child_fwd)
                && lower_right.contains(local_child_fwd)
                && upper_right.contains(local_child_fwd)
                && upper_left.contains(local_child_fwd);

            // Get linear rotation path
            let ray = (local_child_fwd - alg::Vec3::fwd()).norm();

            // Rebind (limit) simple
            let simple = if !inside {
                // Calculate intersection of ray with cone
                let intersection = {
                    let mut candidates = Vec::with_capacity(2);

                    if lower_left.intersects(ray) {
                        candidates.push(lower_left.intersection(ray));
                    }

                    if lower_right.intersects(ray) {
                        candidates.push(lower_right.intersection(ray));
                    }
                    if upper_right.intersects(ray) {
                        candidates.push(upper_right.intersection(ray));
                    }

                    if upper_left.intersects(ray) {
                        candidates.push(upper_left.intersection(ray));
                    }

                    let mut result = candidates[0];

                    if candidates.len() >= 2 {
                        for i in 1..candidates.len() {
                            let point = candidates[i] + alg::Vec3::fwd();

                            // Solution should be inside all four
                            let inside = lower_left.contains(point)
                                && lower_right.contains(point)
                                && upper_right.contains(point)
                                && upper_left.contains(point);

                            if inside {
                                result = candidates[i];
                            }
                        }
                    }

                    result
                };

                // Calculate rotation midpoint
                let midpoint = (alg::Vec3::fwd() + intersection).norm();

                // Limit rotation
                alg::Quat::simple(alg::Vec3::fwd(), midpoint)

            } else { simple };

            let (axis, angle) = twist.to_axis_angle();
            let sign = if axis.dot(local_child_fwd) > 0.0
                { 1.0 } else { -1.0 };

            // Rebind (limit) twist
            let twist = alg::Quat::axis_angle(
                simple * alg::Vec3::fwd() * sign,
                angle.max(joint.z_limit.min).min(joint.z_limit.max),
            );

            // Calculate correction rotation
            let reverse = local_child.conjugate() * twist * simple;
            let transformation = reverse.pow(JOINT_ANG_RIGID).to_mat();

            let correction = {
                let inverse_child = child_orient.transpose();
                child_orient * transformation * inverse_child
            };

            // Correct child
            let point = child.start();
            child.transform_around(point, correction);

            // Correct parent
            let point = parent.end();
            parent.transform_around(point, correction.transpose());
        }

        // Finalize instances
        for i in 0..self.instances.len() {
            let mut instance = match self.instances[i] {
                Some(ref mut instance) => instance,
                None => continue,
            };

            // Compute average position
            let average = {
                let mut sum = alg::Vec3::zero();

                for particle in &instance.particles {
                    sum = sum + particle.position;
                }

                sum / instance.particles.len() as f32
            };

            // Update instance position
            instance.position = average;

            // Update transform position
            transforms.set_position_i(i, average);
        }
    }

    #[allow(unused_variables)]
    pub fn draw_debug(
        &self,
        entity: entity::Handle,
        debug: &mut debug::Handler,
    ) {
        #[cfg(debug_assertions)] {
            let i = entity.get_index() as usize;
            self.draw_instance_debug(i, debug);
        }
    }

    #[allow(unused_variables)]
    pub fn draw_all_debug(&self, debug: &mut debug::Handler) {
        #[cfg(debug_assertions)] {
            for i in 0..self.instances.len() {
                self.draw_instance_debug(i, debug);
            }
        }
    }

    #[allow(unused_variables)]
    fn draw_instance_debug(&self, index: usize, debug: &mut debug::Handler) {
        #[cfg(debug_assertions)] {
            debug_assert!(index < self.instances.len());
            if let Some(ref instance) = self.instances[index] {
                // Draw instance bindings
                for rod in &instance.rods {
                    let left = instance.particles[rod.left].position;
                    let right = instance.particles[rod.right].position;

                    let lerp = (rod.length - left.dist(right)).abs()
                        / (0.1 * rod.length);

                    debug.add_line(
                        alg::Line::new(left, right),
                        graphics::Color::lerp(
                            graphics::Color::green(),
                            graphics::Color::red(),
                            lerp,
                        )
                    );
                }
            }

            // Draw joint endpoints
            for joint in &self.joints {
                if joint.child == index {
                    let child = self.instances[joint.child]
                        .as_ref().unwrap();

                    let parent = self.instances[joint.parent]
                        .as_ref().unwrap();

                    debug.add_local_axes(
                        child.start(),
                        child.fwd(),
                        child.up(),
                        1.0,
                        1.0,
                    );
                }
            }
        }
    }
}

extern crate nmg_vulkan as nmg;

use nmg::alg;
use nmg::entity;
use nmg::render;
use nmg::graphics;
use nmg::components;
use nmg::components::Component;
use nmg::debug;

/* In debug mode, this demo will render in wireframe, with physics markers.
 * In release mode, it will render the solid deformed mesh.
 */

struct Demo {
    objects: Vec<entity::Handle>,
    mass: f32,
    rigidity: f32,
    mesh: Vec<alg::Vec3>,
}

impl nmg::Start for Demo {
    fn start(
        &mut self,
        entities: &mut entity::Manager,
        components: &mut components::Container,
    ) {
        let object = entities.add();
        components.transforms.register(object);
        #[cfg(not(debug_assertions))] { components.draws.register(object, 0); }
        components.softbodies.register(object);

        // Initial position
        components.transforms.set(
            object,
            alg::Vec3::zero(),
            alg::Quat::id(),
            alg::Vec3::one(),
        );

        // Initial softbody
        components.softbodies.init_instance(
            object,
            self.mass,
            self.rigidity,
            &self.mesh,
            &[
                (0, 1),
                (0, 2),
                (0, 3),
                (0, 4),
                (1, 3),
                (3, 4),
                (4, 2),
                (2, 1),
                (1, 4), // Crosspiece
                (2, 3), // Crosspiece
            ],
            &[],
        );

        // Initial force
        components.softbodies.set_force(
            object,
            alg::Vec3::up() * 4400.,
        );

        /* Add collision planes */

        components.softbodies.add_plane(
            alg::Plane::new(alg::Vec3::new(0., -1., 0.01), 1.),
        );

        components.softbodies.add_plane(
            alg::Plane::new(alg::Vec3::up(), 1.),
        );

        /* Zero gravity */

        components.softbodies.set_gravity_raw(alg::Vec3::zero());

        // Update demo state
        self.objects.push(object);
    }
}

impl nmg::Update for Demo {
    #[allow(unused_variables)]
    fn update(
        &mut self,
        time: f64,
        delta: f64,
        metadata: nmg::Metadata,
        screen_height: u32,
        screen_width: u32,
        entities: &mut entity::Manager,
        components: &mut components::Container,
        debug: &mut debug::Handler,
    ) -> render::SharedUBO {
        let shared_ubo = {
            let view = alg::Mat::look_at_view(
                alg::Vec3::new(-1.0, 0.5, -2.0), // Camera position
                alg::Vec3::new( 0.0, 0.0,  0.0), // Target position
                alg::Vec3::up(),
            );

            let projection = {
                alg::Mat::perspective(
                    60.,
                    screen_width as f32 / screen_height as f32,
                    0.01,
                    4.
                )
            };

            render::SharedUBO::new(view, projection)
        };

        /* Debug data */

        debug.clear_lines();
        debug.add_cross( alg::Vec3::up(), 0.4, graphics::Color::red());
        debug.add_cross(-alg::Vec3::up(), 0.4, graphics::Color::red());
        components.softbodies.draw_debug(self.objects[0], debug);

        shared_ubo
    }
}

impl nmg::FixedUpdate for Demo {
    #[allow(unused_variables)]
    fn fixed_update(
        &mut self,
        time: f64,
        fixed_delta: f32,
        metadata: nmg::Metadata,
        screen_height: u32,
        screen_width: u32,
        entities: &mut entity::Manager,
        components: &mut components::Container,
        debug: &mut debug::Handler,
    ) {
        if metadata.fixed_frame > 0 {
            // Reset forces
            components.softbodies.set_force(
                self.objects[0],
                alg::Vec3::zero(),
            );
        }
    }
}

fn main() {
    let model_data = get_models();

    let mesh = {
        let mut points = Vec::with_capacity(model_data[0].vertices.len());

        for vertex in &model_data[0].vertices {
            points.push(vertex.position);
        }

        points
    };

    let demo = Demo {
        objects: Vec::new(),
        mass: 10.0,
        rigidity: 0.06, // Example value (jiggly)
        mesh: mesh,
    };

    nmg::go(model_data, demo)
}

fn get_models() -> Vec<render::ModelData> {
    let pyramid = render::ModelData::new(
        vec![
            render::Vertex::new_raw( 0.0,  0.5,  0.0, 1., 1., 0.), // Peak
            render::Vertex::new_raw( 0.5, -0.5, -0.5, 1., 0., 0.),
            render::Vertex::new_raw(-0.5, -0.5, -0.5, 1., 0., 1.),
            render::Vertex::new_raw( 0.5, -0.5,  0.5, 1., 1., 0.),
            render::Vertex::new_raw(-0.5, -0.5,  0.5, 1., 1., 1.),
        ], vec![
            0u32, 1u32, 2u32,
            0u32, 3u32, 1u32,
            0u32, 4u32, 3u32,
            0u32, 2u32, 4u32,
            1u32, 2u32, 4u32,
            4u32, 3u32, 1u32,
        ],
    );

    vec![pyramid]
}

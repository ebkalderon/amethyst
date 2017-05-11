//! Displays a multicolored sphere to the user.

extern crate amethyst;
extern crate cgmath;
extern crate genmesh;

use amethyst::prelude::*;
use amethyst::asset_manager::AssetManager;
use amethyst::renderer::{VertexPosNormal, Pipeline};
use cgmath::{Vector3, InnerSpace};
use genmesh::{MapToVertices, Triangulate, Vertices};
use genmesh::generators::SphereUV;

struct Example;

impl State for Example {
    fn on_start(&mut self, engine: &mut Engine, pipe: &mut Pipeline) {
        use amethyst::ecs::Gate;
        use amethyst::ecs::components::{Mesh, Texture};
        use amethyst::ecs::resources::{Camera, Projection, ScreenDimensions};
        use amethyst::renderer::{Layer, PointLight};
        use amethyst::renderer::pass::{Clear, DrawShaded};

        let layer = Layer::new("main",
                               vec![Clear::new([0.0, 0.0, 0.0, 1.0]),
                                    DrawShaded::new("main", "main")]);

        pipe.layers.push(layer);

        {
            let dim = engine.world_mut().read_resource::<ScreenDimensions>().pass();
            let mut camera = engine.world_mut().write_resource::<Camera>().pass();
            camera.proj = Projection::Perspective {
                fov: 60.0,
                aspect_ratio: dim.aspect_ratio,
                near: 0.1,
                far: 100.0,
            };
            camera.eye = [5.0, 0.0, 0.0];
            camera.target = [0.0, 0.0, 0.0];
        }

        let sphere_verts = gen_sphere(32, 32);
        engine.assets.register_asset::<Mesh>();
        engine.assets.register_asset::<Texture>();
        engine.assets.load_asset_from_data::<Mesh, Vec<VertexPosNormal>>("sphere", sphere_verts);
        engine.assets.load_asset_from_data::<Texture, [f32; 4]>("blue", [0.0, 0.0, 1.0, 1.0]);
        engine.assets.load_asset_from_data::<Texture, [f32; 4]>("white", [1.0, 1.0, 1.0, 1.0]);

        let sphere = assets.create_renderable("sphere", "blue", "white", "white", 1.0).unwrap();
        engine.world_mut().create_now()
            .with(sphere)
            .build();

        let light = PointLight {
            center: [2.0, 2.0, 2.0],
            radius: 5.0,
            intensity: 3.0,
            ..Default::default()
        };
        engine.world_mut().create_now()
            .with(light)
            .build();
    }

    fn handle_event(&mut self, _: &mut Engine, event: Event) -> Trans {
        match event {
            Event::Window(e) => match e {
                WindowEvent::KeyboardInput(_, _, Some(Key::Escape), _) |
                WindowEvent::Closed => Trans::Quit,
                _ => Trans::None,
            },
            _ => Trans::None,
        }
    }
}

fn main() {
    let path = format!("{}/examples/02_sphere/resources/config.yml",
                       env!("CARGO_MANIFEST_DIR"));
    let cfg = Config::from_file(path).unwrap();
    let mut game = Application::build(Example, cfg).finish().expect("Fatal error");
    game.run();
}


fn gen_sphere(u: usize, v: usize) -> Vec<VertexPosNormal> {
    let data: Vec<VertexPosNormal> = SphereUV::new(u, v)
        .vertex(|(x, y, z)| {
            VertexPosNormal {
                pos: [x, y, z],
                normal: Vector3::new(x, y, z).normalize().into(),
                tex_coord: [0., 0.],
            }
        })
        .triangulate()
        .vertices()
        .collect();
    data
}

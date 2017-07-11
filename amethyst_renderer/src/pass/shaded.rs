//! Blits a color or depth buffer from one Target onto another.

use cam::Camera;
use cgmath::{Matrix4, One};
use gfx;
use gfx::pso::buffer::{ElemStride, NonInstanced};
use gfx::shade::core::UniformValue;
use gfx::traits::Pod;
use pipe::pass::PassBuilder;
use pipe::{Effect, DepthMode};
use std::any::{Any, TypeId};
use std::mem::{self, transmute};
use vertex::{AttributeNames, Color, Normal, Position, PosNormTex, TextureCoord, VertexFormat};
use light::{DirectionalLight, Light, PointLight};
use scene::Scene;
use std::io::Read;

static VERT_SRC: &'static [u8] = include_bytes!("shaders/vertex/basic.glsl");


//pub static FRAG_SRC: &'static [u8] = include_bytes!("shaders/fragment/pbm.glsl");

/// Draw mesh without lighting
#[derive(Clone, Debug, PartialEq)]
pub struct DrawShaded<V: VertexFormat> {
    named_vertex_attributes: V::NamedAttributes,
    fragment_shader: Vec<u8>,
}

impl<V> AttributeNames for DrawShaded<V>
    where V: VertexFormat
{
    fn name<A: Any>() -> &'static str {
        match TypeId::of::<A>() {
            t if t == TypeId::of::<Position>() => "position",
            t if t == TypeId::of::<Normal>() => "normal",
            t if t == TypeId::of::<TextureCoord>() => "tex_coord",
            _ => "", // Unused attribute
        }
    }
}

impl<V> DrawShaded<V>
    where V: VertexFormat
{
    /// Create instance of `DrawShaded` pass
    pub fn new() -> Self {
        DrawShaded {
            named_vertex_attributes: V::named_attributes::<Self>(),
            fragment_shader: {
                let mut data = Vec::new();
                ::std::fs::File::open("src/pass/shaders/fragment/pbm.glsl").unwrap().read_to_end(&mut data).unwrap();
                data
            }
        }
    }
}

static SAMPLER_NAMES: [&'static str; 7] = [
    "sampler_albedo",
    "sampler_emission",
    "sampler_normal",
    "sampler_metallic",
    "sampler_roughness",
    "sampler_ambient_occlusion",
    "sampler_caveat",
];


fn pad(x: [f32; 3]) -> [f32; 4] {
    [x[0], x[1], x[2], 1.0]
}

impl<'a, V> Into<PassBuilder<'a>> for &'a DrawShaded<V>
    where V: VertexFormat
{
    fn into(self) -> PassBuilder<'a> {
        use gfx::texture::{FilterMethod, WrapMode};

        #[derive(Clone, Copy, Debug)]
        struct VertexArgs {
            proj: [[f32;4]; 4],
            view: [[f32;4]; 4],
            model: [[f32;4]; 4],
        };
        #[derive(Clone, Copy, Debug)]
        struct FragmentArgs {
            point_light_count: i32, 
            directional_light_count: i32,
        };
        #[derive(Clone, Copy, Debug)]
        #[repr(C)]
        struct PointLight {
            position: [f32; 4],
            color: [f32; 4],
            intensity: f32,
            _pad: [f32; 3],
        };
        unsafe impl Pod for PointLight {}
        #[derive(Clone, Copy, Debug)]
        struct DirectionalLight {
            direction: [f32; 3],
            color: [f32; 3],
        };
        unsafe impl Pod for DirectionalLight {}

        let effect = Effect::new_simple_prog(VERT_SRC, &self.fragment_shader)
            .with_raw_vertex_buffer(self.named_vertex_attributes.as_ref(), PosNormTex::size() as ElemStride, 0)
            .with_raw_constant_buffer("VertexArgs", mem::size_of::<VertexArgs>(), 1)
            .with_raw_constant_buffer("FragmentArgs", mem::size_of::<FragmentArgs>(), 1)
            .with_raw_constant_buffer("PointLights", mem::size_of::<PointLight>(), 512)
            .with_raw_constant_buffer("DirectionalLight", mem::size_of::<DirectionalLight>(), 16)
            .with_raw_global("ambient_color")
            .with_raw_global("camera_position")
            .with_sampler(&SAMPLER_NAMES, FilterMethod::Scale, WrapMode::Clamp)
            .with_texture("sampler_roughness")
            .with_texture("sampler_caveat")
            .with_texture("sampler_metallic")
            .with_texture("sampler_emission")
            .with_texture("sampler_ambient_occlusion")
            .with_texture("sampler_albedo")
            .with_texture("sampler_normal")
            .with_output("out_color", None);

        PassBuilder::main(effect, move |ref mut enc, ref out, ref effect, ref scene, ref model| {
            
            let mut data = effect.pso_data.clone();
            {
                let vertex_args = scene.active_camera().map(|cam| VertexArgs {
                    proj: cam.proj.into(),
                    view: Matrix4::look_at(cam.eye, cam.eye + cam.forward, cam.up).into(),
                    model: model.pos.into(),
                }).unwrap_or_else(|| VertexArgs {
                    proj: Matrix4::one().into(),
                    view: Matrix4::one().into(),
                    model: model.pos.into(),
                });
                let vertex_args_buf = effect.const_bufs["VertexArgs"].clone();
                // FIXME: update raw buffer without transmute
                enc.update_constant_buffer::<VertexArgs>(unsafe { transmute(&vertex_args_buf) }, &vertex_args);
                data.const_bufs.push(vertex_args_buf);
            }
            {
                let mut point_lights = Vec::new();
                let mut directional_lights = Vec::new();
                for (i, light) in scene.lights().iter().enumerate() {
                    match *light {
                        Light::Directional(ref light) => directional_lights.push(DirectionalLight {
                            direction: light.direction.into(),
                            color: light.color.into(),
                        }),
                        Light::Point(ref light) => point_lights.push(PointLight {
                            position: pad(light.center.into()),
                            color: pad(light.color.into()),
                            intensity: light.intensity,
                            _pad: [0.0; 3],
                        }),
                        _ => {}
                    }
                }

                let fragment_args = FragmentArgs {
                    point_light_count: point_lights.len() as i32,
                    directional_light_count: directional_lights.len() as i32,
                };

                let fragment_args_buf = effect.const_bufs["FragmentArgs"].clone();
                enc.update_constant_buffer::<FragmentArgs>(unsafe { transmute(&fragment_args_buf) }, &fragment_args);

                let point_lights_buf = effect.const_bufs["PointLights"].clone();
                enc.update_buffer::<PointLight>(unsafe { transmute(&point_lights_buf) }, &point_lights[..], 0);

                let directional_lights_buf = effect.const_bufs["DirectionalLight"].clone();
                enc.update_buffer::<DirectionalLight>(unsafe { transmute(&directional_lights_buf) }, &directional_lights[..], 0);

                data.const_bufs.push(fragment_args_buf);
                data.const_bufs.push(point_lights_buf);
                data.const_bufs.push(directional_lights_buf);
            }
            {
                data.globals.push(UniformValue::F32Vector3([0.005; 3].into()));
                data.globals.push(UniformValue::F32Vector3(scene.active_camera().map(|cam| cam.eye.into()).unwrap_or([0.0; 3])));
            }
            {
                data.samplers.push(effect.samplers["sampler_roughness"].clone());
                data.textures.push(model.material.roughness.view().clone());
                
                data.samplers.push(effect.samplers["sampler_caveat"].clone());
                data.textures.push(model.material.caveat.view().clone());
                
                data.samplers.push(effect.samplers["sampler_metallic"].clone());
                data.textures.push(model.material.metallic.view().clone());
                
                data.samplers.push(effect.samplers["sampler_emission"].clone());
                data.textures.push(model.material.emission.view().clone());
                
                data.samplers.push(effect.samplers["sampler_ambient_occlusion"].clone());
                data.textures.push(model.material.ambient_occlusion.view().clone());

                data.samplers.push(effect.samplers["sampler_albedo"].clone());
                data.textures.push(model.material.albedo.view().clone());
                
                data.samplers.push(effect.samplers["sampler_normal"].clone());
                data.textures.push(model.material.normal.view().clone());
            }

            let (vertex, slice) = model.mesh.geometry();
            data.vertex_bufs.push(vertex.clone());
            data.out_colors.extend(out.color_buf(0).map(|cb| cb.as_output.clone()));
            enc.draw(slice, &effect.pso, &data);
        })
    }
}

//! Blits a color or depth buffer from one Target onto another.

use cam::Camera;
use cgmath::{Matrix4, One};
use gfx;
use gfx::pso::buffer::{ElemStride, NonInstanced};
use pipe::pass::PassBuilder;
use pipe::{Effect, DepthMode};
use std::any::{Any, TypeId};
use std::mem::{self, transmute};
use vertex::{AttributeNames, Color, Normal, Position, PosNormTex, TextureCoord, VertexFormat};

static VERT_SRC: &'static [u8] = b"
    #version 150 core

    layout (std140) uniform VertexArgs {
        uniform mat4 proj;
        uniform mat4 view;
        uniform mat4 model;
    };

    in vec3 position;
    in vec3 normal;
    in vec2 tex_coord;

    out VertexData {
        vec4 position;
        vec3 normal;
        vec2 tex_coord;
    } vertex_data;

    void main() {
        vertex_data.position = model * vec4(position, 1.0);
        vertex_data.normal = mat3(model) * normal;
        vertex_data.tex_coord = tex_coord;
        gl_Position = proj * view * vertex_data.position;
    }
";

static FRAG_SRC: &'static [u8] = b"
    #version 150 core

    uniform sampler2D albedo;

    in VertexData {
        vec4 position;
        vec3 normal;
        vec2 tex_coord;
    } vertex_data;

    out vec4 color;

    void main() {
        color = texture(albedo, vec2(0, 0));
    }
";

/// Draw mesh without lighting
#[derive(Clone, Debug, PartialEq)]
pub struct DrawFlat<V: VertexFormat> {
    named_vertex_attributes: V::NamedAttributes,
}

impl<V> AttributeNames for DrawFlat<V>
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

impl<V> DrawFlat<V>
    where V: VertexFormat
{
    /// Create instance of `DrawFlat` pass
    pub fn new() -> Self {
        DrawFlat {
            named_vertex_attributes: V::named_attributes::<Self>(),
        }
    }
}

impl<'a, V> Into<PassBuilder<'a>> for &'a DrawFlat<V>
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

        let effect = Effect::new_simple_prog(VERT_SRC, FRAG_SRC)
            .with_raw_constant_buffer("VertexArgs", mem::size_of::<VertexArgs>(), 1)
            .with_raw_vertex_buffer(self.named_vertex_attributes.as_ref(), PosNormTex::size() as ElemStride, 0)
            .with_sampler("albedo", FilterMethod::Scale, WrapMode::Clamp)
            .with_texture("albedo")
            .with_output("color", None);

        PassBuilder::main(effect, move |ref mut enc, ref out, ref model, ref effect, ref scene| {
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

            let mut data = effect.pso_data.clone();
            data.const_bufs.push(vertex_args_buf);
            let (vertex, slice) = model.mesh.geometry();
            data.vertex_bufs.push(vertex.clone());
            data.samplers.push(effect.samplers["albedo"].clone());
            data.textures.push(model.material.albedo.view().clone());
            data.out_colors.extend(out.color_buf(0).map(|cb| cb.as_output.clone()));
            enc.draw(slice, &effect.pso, &data);
        })
    }
}

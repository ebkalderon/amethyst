//! Available vertex attributes.

#![allow(missing_docs)]

/// Vertex format with position and RGBA8 color attributes.
gfx_vertex_struct! {
    PosColor {
        position: [f32; 3] = "a_position",
        color: [f32; 4] = "a_color",
    }
}

/// Vertex format with position, normal, and UV texture coordinate attributes.
gfx_vertex_struct! {
    PosNormTex {
        position: [f32; 3] = "a_position",
        normal: [f32; 4] = "a_normal",
        tex_coord: [f32; 2] = "a_tex_coord",
    }
}

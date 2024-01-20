use super::mesh::MeshTex;

pub struct Room{
    pub position:[f32;3],
    pub walls:Vec<Wall>,
    pub height:f32,
    pub moddifiers:Vec<Modifier>,
}

pub struct Wall{
    pub local_pos:[f32;2],
    pub wall_texture:MeshTex,
}

pub enum Modifier{
    Ramp{
        pos:[f32;3],
        dir:f32,
        size:[f32;3],
        floor_texture:MeshTex,
        wall_texture:MeshTex
    }
}
use bevy::{
    color::{LinearRgba, Srgba},
    reflect::{Reflect, Struct},
};
use bytemuck::{Pod, Zeroable};
pub trait ShaderParams:
    Pod + Zeroable + Copy + std::fmt::Debug + Default + Reflect + Struct + PartialEq
{
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect, Pod, Zeroable)]
pub struct NoParams;

impl ShaderParams for NoParams {}

#[repr(C)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Default,
    Reflect,
    bytemuck::Pod,
    bytemuck::Zeroable,
    bevy::ecs::component::Component,
)]
pub struct ColorParams {
    pub color: LinearRgba,
}

impl ShaderParams for ColorParams {}

impl From<bevy::prelude::Color> for ColorParams {
    fn from(value: bevy::prelude::Color) -> Self {
        Self {
            color: value.into(),
        }
    }
}

impl From<Srgba> for ColorParams {
    fn from(value: Srgba) -> Self {
        Self {
            color: value.into(),
        }
    }
}
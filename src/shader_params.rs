use bevy::reflect::{Reflect, Struct};
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
#[derive(Debug, Clone, Copy, PartialEq, Default, Reflect, Pod, Zeroable)]
pub struct LinearRGBA {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

impl std::ops::Mul<f32> for LinearRGBA {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            red: self.red * rhs,
            green: self.green * rhs,
            blue: self.blue * rhs,
            alpha: self.alpha * rhs,
        }
    }
}

impl std::ops::Add for LinearRGBA {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            red: self.red + rhs.red,
            green: self.green + rhs.green,
            blue: self.blue + rhs.blue,
            alpha: self.alpha + rhs.alpha,
        }
    }
}

impl From<bevy::prelude::Color> for LinearRGBA {
    fn from(value: bevy::prelude::Color) -> Self {
        let [red, green, blue, alpha] = value.as_linear_rgba_f32();
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Reflect, Pod, Zeroable)]
pub struct LinearRGB {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

impl From<bevy::prelude::Color> for LinearRGB {
    fn from(value: bevy::prelude::Color) -> Self {
        let [red, green, blue, _alpha] = value.as_linear_rgba_f32();
        Self { red, green, blue }
    }
}

impl std::ops::Mul<f32> for LinearRGB {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            red: self.red * rhs,
            green: self.green * rhs,
            blue: self.blue * rhs,
        }
    }
}

impl std::ops::Add for LinearRGB {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            red: self.red + rhs.red,
            green: self.green + rhs.green,
            blue: self.blue + rhs.blue,
        }
    }
}



#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Reflect, bytemuck::Pod, bytemuck::Zeroable, bevy::ecs::component::Component)]
pub struct ColorParams {
    pub color: LinearRGBA,
}

impl ShaderParams for ColorParams {}

impl From<bevy::prelude::Color> for ColorParams {
    fn from(value: bevy::prelude::Color) -> Self {
        Self {
            color: value.into(),
        }
    }
}
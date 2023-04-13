use bytemuck::Pod;

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

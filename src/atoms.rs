#[derive(Copy, Clone, Debug)]
pub struct Vec2<T>
{
    pub x: T, pub y: T
}

impl<T: std::ops::Neg<Output = T>> std::ops::Neg for Vec2<T>
{
    type Output = Self;
    fn neg(self) -> Self
    {
        Self::new(-self.x, -self.y)
    }
}

impl<T: std::ops::Add<Output = T>> std::ops::Add for Vec2<T>
{
    type Output = Self;
    fn add(self, other: Self) -> Self
    {
        Vec2::new(self.x + other.x, self.y + other.y)
    }
}

impl<T: std::ops::Sub<Output = T>> std::ops::Sub for Vec2<T>
{
    type Output = Self;
    fn sub(self, other: Self) -> Self
    {
        Vec2::new(self.x - other.x, self.y - other.y)
    }
}


impl<T: std::ops::Div<Output = T> + Copy> std::ops::Div<T> for Vec2<T>
{
    type Output = Self;
    fn div(self, v: T) -> Self
    {
        Vec2::new(self.x / v, self.y / v)
    }
}

impl<T: std::ops::Mul<Output = T> + Copy> std::ops::Mul<T> for Vec2<T>
{
    type Output = Self;
    fn mul(self, v: T) -> Self
    {
        Vec2::new(self.x * v, self.y * v)
    }
}

impl<T> Vec2<T>
{
    pub const fn new(x: T, y: T) -> Vec2<T>
    {
        Self {x, y}
    }
}

impl Vec2<f64>
{
    pub fn norm(self) -> Vec2<f64>
    {
        self/(self.x * self.x + self.y * self.y).sqrt()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Rect<T>
{
    pub pos: Vec2<T>,
    pub size: Vec2<T>
}

impl<T: PartialOrd + std::ops::Add<Output = T> + Copy> std::ops::BitAnd<Rect<T>> for self::Rect<T>
{
    type Output = bool;
    fn bitand(self, other: Self) -> bool
    {
        self.pos.x + self.size.x > other.pos.x &&
            self.pos.x < other.pos.x + other.size.x &&
            self.pos.y + self.size.y > other.pos.y &&
            self.pos.y < other.pos.y + other.size.y
    }
}

impl<T> Rect<T>
{
    pub const fn new(x: T, y: T, width: T, height: T) -> Rect<T>
    {
        Self {pos: Vec2::new(x, y), size: Vec2::new(width, height)}
    }
}

impl<T: std::ops::Add<Output = T>> Rect<T>
{
    pub fn at(self, vec: Vec2<T>) -> Self
    {
        Rect { pos: vec + self.pos, size: self.size }
    }
}

pub type Rectf = Rect<f32>;
pub type Vec2f = Vec2<f32>;
pub type Recti = Rect<i32>;
pub type Vec2i = Vec2<i32>;
pub type Id = usize;
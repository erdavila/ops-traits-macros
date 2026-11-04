pub trait Sqr {
    type Output;

    fn sqr(self) -> Self::Output;
}

impl Sqr for i32 {
    type Output = i32;

    fn sqr(self) -> Self::Output {
        self * self
    }
}

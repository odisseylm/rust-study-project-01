

pub trait ValExt<T: Sized + Copy> {
    fn also<F>(self, f: F) -> Self where Self: Sized + Copy, F: FnOnce(Self) {
        f(self);
        self
    }
    #[inline]
    fn is_one_of2(&self, v1: Self, v2: Self) -> bool where Self: Sized + Copy + Eq, T: Eq {
        *self == v1 || *self == v2
    }
    #[inline]
    fn is_one_of3(&self, v1: Self, v2: Self, v3: Self) -> bool where Self: Sized + Copy + Eq, T: Eq {
        *self == v1 || *self == v2 || *self == v3
    }
}
pub trait ValRefExt<T> {
    fn also_ref<F>(&self,f :F) -> &Self where F: FnOnce(&Self) {
        f(self);
        self
    }
    fn also_ref_mut<F>(&mut self,f :F) -> &Self where F: FnOnce(&mut Self) {
        f(self);
        self
    }
    #[inline]
    fn is_ref_one_of2(&self, v1: &Self, v2: &Self) -> bool where Self: Eq, T: Eq {
        self == v1 || self == v2
    }
    #[inline]
    fn is_ref_one_of3(&self, v1: &Self, v2: &Self, v3: &Self) -> bool where Self: Eq, T: Eq {
        self == v1 || self == v2 || self == v3
    }
}

impl<T: Sized + Copy> ValExt<T> for T { }
impl<T> ValRefExt<T> for T { }


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_also() {
        println!("var: {}", true);
        println!("var: {}", true.also(|v| println!("from 'also' (v: {})", v)));
    }
}

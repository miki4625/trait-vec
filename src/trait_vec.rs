use crate::dyn_view_ptr::{DynViewPtr, OffsetDynView};
use std::fmt::Debug;
use std::marker::Unsize;
use std::mem::{align_of, size_of};
use std::ops::Deref;
use std::ptr;
use std::ptr::{DynMetadata, Pointee};
use std::slice::Iter;

pub struct OffsettingIter<'a, T: ?Sized + 'a> {
    ref_to_vec: &'a PolyPtrVec<T>,
    iter: Iter<'a, OffsetDynView<T>>
}

impl<'a, T: ?Sized> OffsettingIter<'a, T> {
    #[inline]
    pub fn new(poly_vec: &'a PolyPtrVec<T>) -> OffsettingIter<'a, T> {
        Self {
            ref_to_vec: poly_vec,
            iter: poly_vec.view.iter(),
        }
    }
}

impl<'a, T: ?Sized + 'a> Iterator for OffsettingIter<'a, T> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|view_offset| unsafe { view_offset.as_view(self.ref_to_vec.buf.as_ptr().to_raw_parts().0).into_inner() })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

pub struct PolyPtrVec<T: ?Sized> {
    view: Vec<OffsetDynView<T>>,
    buf: Vec<u8>,
}

impl<T: ?Sized> PolyPtrVec<T> {
    #[inline]
    fn push_value<U: Unsize<T>>(&mut self, mut value: U) -> OffsetDynView<T> {
        let needed_space = aligned_size_of::<U>();
        let ptr = unsafe {
            let ptr = self.buf.as_ptr();
            let new_ptr = ptr.add(self.buf.len());
            let pointee = &mut value as &mut T as *mut T;
            let info = pointee.to_raw_parts();
            ptr::write(new_ptr as *mut U, value);
            self.buf.set_len(self.buf.len() + needed_space);
            ptr::from_raw_parts::<T>(new_ptr as *const (), info.1).cast_mut()
        };
        let offset = unsafe { self.buf.as_ptr().offset_from(ptr as *const u8) };
        OffsetDynView::<T>::from_ptr(offset, ptr)
    }

    fn relational_size_of<U>() -> usize {
        aligned_size_of::<U>() / aligned_size_of::<u8>()
    }
}

fn aligned_size_of<U>() -> usize {
    size_of::<U>() + align_of::<U>()
}

impl<T: ?Sized> Default for PolyPtrVec<T> {
    fn default() -> Self {
        PolyPtrVec::<T>::new()
    }
}

impl<T: ?Sized> PolyPtrVec<T> {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            view: Vec::new(),
            buf: Vec::new(),
        }
    }

    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            view: Vec::with_capacity(capacity),
            buf: Vec::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn push<U: Unsize<T>>(&mut self, value: U) {
        let size = aligned_size_of::<U>();
        /*        if self.buf.free_capacity() < size {
            self.buf.reserve_for_push(size - self.buf.free_capacity());
        }*/

        let view = self.push_value::<U>(value);
        self.view.push(view);
    }

    #[inline]
    pub fn iter<'a>(&'a self) -> OffsettingIter<'a, T> {
        OffsettingIter::<'a, T>::new(self)
    }
}

impl<T: ?Sized + Debug> PolyPtrVec<T> {
    pub fn coercion<U>(value: U)
    where
        U: AsRef<T> + Debug,
    {
        println!("U: {:?}", value);
        println!("T: {:?}", value.as_ref());
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fmt::Debug;
    use std::mem::size_of;
    use std::ops::{Deref, DerefMut};

    #[derive(Debug)]
    struct Example {
        inner: f64,
    }

    #[derive(Debug)]
    struct Point {
        x: f64,
        y: f64,
        z: f64,
    }

    impl Example {
        pub fn new(value: f64) -> Self {
            Self { inner: value }
        }
    }

    impl Point {
        pub fn new(x: f64, y: f64, z: f64) -> Self {
            Self { x, y, z }
        }
    }

    #[test]
    fn without_resize() {
        let mut vec = PolyPtrVec::<dyn Debug>::with_capacity(1024);
        vec.push(Example::new(std::f64::consts::E));
        vec.push(Example::new(std::f64::consts::PI));
        vec.push(Point::new(0.5, 0.0, 1.7));
        vec.push(Example::new(std::f64::consts::SQRT_2));

        let mut last_addr: Option<usize> = None;
        for (index, view) in vec.iter().enumerate() {
            let pointer = view.deref() as *const dyn Debug;
            let current_addr = pointer.addr();
            let diff = if let Some(addr) = last_addr {
                current_addr - addr
            } else {
                0
            };
            last_addr = Some(current_addr);

            let size_of_example = size_of::<Example>() + align_of::<Example>();
            println!(
                "{}: Value: {:?} | Addr: {} | Addr diff {} | Proper diff {}",
                index, view, current_addr, diff, size_of_example
            );
        }
    }

    #[test]
    fn slice<'a>() {
        let mut vec = PolyPtrVec::<[usize]>::with_capacity(1024);
        vec.push([3; 10]);
        vec.push([10; 15]);

        let result = vec.iter().flat_map(|slice| slice).sum::<usize>();
        //let result = vec.iter().flat_map(|slice| unsafe {slice.inner().as_ref()}).sum::<usize>();
        assert_eq!(180, result);
    }
}

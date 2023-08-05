use crate::dyn_view_ptr::OffsetDynView;
use std::collections::TryReserveError;
use std::marker::Unsize;
use std::mem::{align_of, size_of};
use std::ptr;
use std::slice::Iter;

pub struct OffsettingIter<'a, T: ?Sized + 'a> {
    ref_to_vec: &'a PolyPtrVec<T>,
    iter: Iter<'a, OffsetDynView<T>>,
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
        self.iter.next().map(|view_offset| unsafe {
            view_offset
                .as_view(self.ref_to_vec.buf.as_ptr().to_raw_parts().0)
                .into_inner()
        })
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

    #[inline]
    fn insert_element<U: Unsize<T>>(&mut self, index: usize, element: U) -> OffsetDynView<T> {
        let needed_space = aligned_size_of::<U>();
        let view_len = self.view.len();
        #[cold]
        #[inline(never)]
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("insertion index (is {index}) should be <= len (is {len})");
        }

        let ptr = unsafe {
            let ptr = match self.view.get(index) {
                None => {
                    if index == view_len {
                        self.buf.as_mut_ptr().add(self.buf.len())
                    } else {
                        assert_failed(index, view_len);
                    }
                }
                Some(view) => {
                    let src = self.buf.as_mut_ptr().offset(-view.offset);
                    let dst = src.add(needed_space);
                    let count = (self.buf.len() as isize + view.offset) as usize;

                    ptr::copy(src, dst, count);
                    src
                }
            } as *mut U;
            ptr::write(ptr, element);
            self.buf.set_len(self.buf.len() + needed_space);
            ptr
        };

        let offset = unsafe { self.buf.as_ptr().offset_from(ptr as *const u8) };
        OffsetDynView::<T>::from_ptr(offset, ptr)
    }

    #[track_caller]
    fn remove_element(&mut self, index: usize) -> usize {
        #[cold]
        #[inline(never)]
        #[track_caller]
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("removal index (is {index}) should be < len (is {len})");
        }

        match self.view.get(index) {
            None => assert_failed(index, self.view.len()),
            Some(view) => {
                let next_offset = if let Some(next_view) = self.view.get(index + 1) {
                    next_view.offset
                } else {
                    self.buf.len() as isize
                };

                let size = (next_offset - view.offset) as usize;
                unsafe {
                    let ptr = self.buf.as_mut_ptr().offset(view.offset);
                    ptr::copy(
                        ptr.add(size),
                        ptr,
                        self.buf.len() - view.offset as usize - size,
                    );
                }
                size
            }
        }
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

/// Implementation of vec-like methods for inner raw buffer
impl<T: ?Sized> PolyPtrVec<T> {
    #[inline]
    #[must_use]
    pub fn raw_with_capacity(count: usize, buf_raw_capacity: usize) -> Self {
        Self {
            view: Vec::with_capacity(count),
            buf: Vec::with_capacity(buf_raw_capacity),
        }
    }

    #[inline]
    pub fn raw_capacity(&self) -> usize {
        self.buf.capacity()
    }

    #[inline]
    pub fn raw_reserve(&mut self, additional: usize) {
        self.buf.reserve(additional);
    }

    #[inline]
    pub fn raw_reserve_exact(&mut self, additional: usize) {
        self.buf.reserve_exact(additional)
    }

    #[inline]
    pub fn raw_try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.buf.try_reserve(additional)
    }

    #[inline]
    pub fn raw_try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.buf.try_reserve_exact(additional)
    }

    #[inline]
    pub fn raw_shrink_to(&mut self, min_capacity: usize) {
        self.buf.shrink_to(min_capacity)
    }
}

impl<T: ?Sized> PolyPtrVec<T> {
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.view.len()
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.view.len() == 0
    }

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
    pub fn with_capacity<U: Unsize<T>>(count: usize) -> Self {
        Self {
            view: Vec::with_capacity(count),
            buf: Vec::with_capacity(count * aligned_size_of::<U>()),
        }
    }

    #[inline]
    pub fn capacity<U: Unsize<T>>(&self) -> usize {
        self.buf.capacity() / aligned_size_of::<U>()
    }

    #[inline]
    pub fn reserve<U: Unsize<T>>(&mut self, additional: usize) {
        self.buf.reserve(additional * aligned_size_of::<U>());
    }

    #[inline]
    pub fn reserve_exact<U: Unsize<T>>(&mut self, additional: usize) {
        self.buf.reserve_exact(additional * aligned_size_of::<U>())
    }

    #[inline]
    pub fn try_reserve<U: Unsize<T>>(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.buf.try_reserve(additional * aligned_size_of::<U>())
    }

    #[inline]
    pub fn try_reserve_exact<U: Unsize<T>>(
        &mut self,
        additional: usize,
    ) -> Result<(), TryReserveError> {
        self.buf
            .try_reserve_exact(additional * aligned_size_of::<U>())
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.buf.shrink_to_fit()
    }

    #[inline]
    pub fn shrink_to<U: Unsize<T>>(&mut self, min_capacity: usize) {
        self.buf.shrink_to(min_capacity * aligned_size_of::<U>())
    }

    #[inline]
    pub fn truncate(&mut self, len: usize) {
        self.view.truncate(len);
        match self.view.last() {
            None => self.view.truncate(len),
            Some(view) => self.buf.truncate(view.offset as usize),
        }
    }

    #[inline]
    pub fn push<U: Unsize<T>>(&mut self, value: U) {
        self.buf.reserve(aligned_size_of::<U>());
        let view = self.push_value::<U>(value);
        self.view.push(view);
    }

    #[inline]
    pub fn push_within_capacity<U: Unsize<T>>(&mut self, value: U) -> Result<(), U> {
        if self.buf.len() == self.buf.capacity() {
            return Err(value);
        }
        let view = self.push_value::<U>(value);
        self.view.push(view);
        Ok(())
    }

    #[inline]
    pub fn insert<U: Unsize<T>>(&mut self, index: usize, element: U) {
        self.buf.reserve(aligned_size_of::<U>());
        let view = self.insert_element::<U>(index, element);
        self.view.insert(index, view);
        self.view
            .iter_mut()
            .skip(index + 1)
            .for_each(|view| view.offset += aligned_size_of::<U>() as isize)
    }

    /// Remove is divided into 2 methods (remove and remove_ret)
    /// Because elements have different types, they can have diffrent size. and
    /// Returning diffrent size values from functions isn't stable for now
    #[track_caller]
    pub fn remove(&mut self, index: usize) {
        let freed_space = self.remove_element(index);
        self.view.remove(index);
        self.view
            .iter_mut()
            .skip(index)
            .for_each(|view| view.offset -= freed_space as isize)
    }

    #[inline]
    pub fn iter<'a>(&'a self) -> OffsettingIter<'a, T> {
        OffsettingIter::<'a, T>::new(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fmt::Debug;
    use std::mem::size_of;

    #[allow(dead_code)]
    #[derive(Debug)]
    struct Example {
        inner: f64,
    }

    #[allow(dead_code)]
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
    fn push_without_resize() {
        let mut vec = PolyPtrVec::<dyn Debug>::with_capacity::<Point>(4);
        vec.push(Example::new(std::f64::consts::E));
        vec.push(Example::new(std::f64::consts::PI));
        vec.push(Point::new(0.5, 0.0, 1.7));
        vec.push(Example::new(std::f64::consts::SQRT_2));

        let mut last_addr: Option<usize> = None;
        for (index, view) in vec.iter().enumerate() {
            let pointer = view as *const dyn Debug;
            let current_addr = pointer.addr();
            let diff = if let Some(addr) = last_addr {
                current_addr - addr
            } else {
                0
            };

            let size_of_example = size_of::<Example>() + align_of::<Example>();
            println!(
                "{}: Value: {:?} | Addr: {} | Addr diff {} | Proper diff {}",
                index, view, current_addr, diff, size_of_example
            );

            last_addr = Some(current_addr);
        }
    }

    #[test]
    fn push_with_resize() {
        let mut vec = PolyPtrVec::<dyn Debug>::new();
        vec.push(Example::new(std::f64::consts::E));
        vec.push(Example::new(std::f64::consts::PI));
        vec.push(Point::new(0.5, 0.0, 1.7));
        vec.push(Example::new(std::f64::consts::SQRT_2));
        vec.push(Point::new(0.25, 0.5, 1.5));
        vec.push(Point::new(0.5, 0.5, 1.35));

        let mut last_addr: Option<usize> = None;
        for (index, view) in vec.iter().enumerate() {
            let pointer = view as *const dyn Debug;
            let current_addr = pointer.addr();
            let diff = if let Some(addr) = last_addr {
                current_addr - addr
            } else {
                0
            };

            let size_of_example = size_of::<Example>() + align_of::<Example>();
            println!(
                "{}: Value: {:?} | Addr: {} | Addr diff {} | Proper diff {}",
                index, view, current_addr, diff, size_of_example
            );

            last_addr = Some(current_addr);
        }
    }

    #[test]
    fn push_within_capacity() {
        let mut vec = PolyPtrVec::<dyn Debug>::with_capacity::<Example>(3);
        assert!(vec
            .push_within_capacity(Example::new(std::f64::consts::E))
            .is_ok());
        assert!(vec
            .push_within_capacity(Example::new(std::f64::consts::PI))
            .is_ok());
        assert!(vec
            .push_within_capacity(Example::new(std::f64::consts::SQRT_2))
            .is_ok());
        assert!(vec
            .push_within_capacity(Example::new(std::f64::consts::E))
            .is_err());
        assert!(vec
            .push_within_capacity(Example::new(std::f64::consts::E))
            .is_err());
    }

    #[test]
    fn slice() {
        let mut vec = PolyPtrVec::<[usize]>::with_capacity::<[usize; 25]>(1);
        vec.push([3; 10]);
        vec.push([10; 15]);

        let result = vec.iter().flatten().sum::<usize>();
        assert_eq!(180, result);
    }
}

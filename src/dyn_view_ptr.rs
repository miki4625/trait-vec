use std::marker::Unsize;
use std::ops::{CoerceUnsized, Deref, DerefMut, DispatchFromDyn};
use std::ptr::{NonNull, Pointee};
use std::{fmt, ptr};

pub struct OffsetDynView<T: ?Sized> {
    pub(crate) offset: isize,
    ptr_metadata: <T as Pointee>::Metadata,
}

impl<T: ?Sized> Clone for OffsetDynView<T> {
    fn clone(&self) -> Self {
        Self {
            offset: self.offset,
            ptr_metadata: self.ptr_metadata,
        }
    }
}

impl<'a, T: ?Sized + 'a> OffsetDynView<T> {
    unsafe fn offset_ptr(&self, valid_ptr: *const ()) -> *const () {
        (valid_ptr as *const u8).offset(-self.offset) as *const ()
    }

    unsafe fn offset_ptr_mut(&self, valid_ptr: *mut ()) -> *mut () {
        (valid_ptr as *mut u8).offset(-self.offset) as *mut ()
    }

    pub fn from_ptr(offset: isize, invalid_ptr: *const T) -> Self {
        Self {
            offset,
            ptr_metadata: invalid_ptr.to_raw_parts().1,
        }
    }

    #[inline]
    pub unsafe fn as_view(&self, valid_ptr: *const ()) -> DynViewPtr<T> {
        let data_ptr = self.offset_ptr(valid_ptr);
        let t_ptr = ptr::from_raw_parts::<T>(data_ptr, self.ptr_metadata) as *const T;
        DynViewPtr::<T>::from_ptr_unchecked(t_ptr)
    }

    #[inline]
    pub unsafe fn as_mut_view(&self, valid_ptr: *mut ()) -> DynViewPtr<T> {
        let data_ptr = self.offset_ptr_mut(valid_ptr);
        let t_ptr = ptr::from_raw_parts_mut::<T>(data_ptr, self.ptr_metadata);
        DynViewPtr::<T>::from_mut_ptr_unchecked(t_ptr)
    }
}

#[repr(transparent)]
pub struct DynViewPtr<T>
where
    T: ?Sized,
{
    pointer: NonNull<T>,
}

impl<'a, T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<DynViewPtr<U>> for DynViewPtr<T> {}

impl<'a, T: ?Sized + Unsize<U>, U: ?Sized> DispatchFromDyn<DynViewPtr<U>> for DynViewPtr<T> {}

impl<'a, T: ?Sized> DynViewPtr<T> {
    #[inline(always)]
    fn as_ref<'b>(&'b self) -> &'a T {
        unsafe { self.pointer.as_ref() }
    }

    #[inline(always)]
    pub fn inner(&self) -> &NonNull<T> {
        &self.pointer
    }

    #[inline]
    fn from_inner(pointer: NonNull<T>) -> Self {
        Self { pointer }
    }
}

impl<'a, T: ?Sized + 'a> DynViewPtr<T> {
    /*    #[inline(always)]
    #[must_use]
    pub fn new(mut x: Box<T>) -> Self {
        let pointee = Box::<T>::leak(x);
        Self {
            pointer: NonNull::new(pointee as *mut T).unwrap(),
            _marker: PhantomData,
        }
    }*/

    #[inline]
    pub unsafe fn from_mut_ptr_unchecked(ptr: *mut T) -> DynViewPtr<T> {
        unsafe { DynViewPtr::<T>::from_inner(NonNull::new_unchecked(ptr)) }
    }

    #[inline]
    pub unsafe fn from_ptr_unchecked(ptr: *const T) -> DynViewPtr<T> {
        unsafe { DynViewPtr::<T>::from_inner(NonNull::new_unchecked(ptr as *mut T)) }
    }

    pub fn from_ptr(ptr: *mut T) -> Option<Self> {
        NonNull::new(ptr).map(|nn| Self::from_inner(nn))
    }

    #[inline]
    pub fn into_inner(self) -> &'a T {
        unsafe { self.pointer.as_ref() }
    }
}

impl<T: fmt::Display + ?Sized> fmt::Display for DynViewPtr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: fmt::Debug + ?Sized> fmt::Debug for DynViewPtr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: ?Sized> fmt::Pointer for DynViewPtr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ptr: *const T = &**self;
        fmt::Pointer::fmt(&ptr, f)
    }
}

impl<'a, T: ?Sized + 'a> Deref for DynViewPtr<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        unsafe { self.pointer.as_ref() }
    }
}

impl<'a, T: ?Sized + 'a> DerefMut for DynViewPtr<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.pointer.as_mut() }
    }
}

#[cfg(test)]
mod test {
    use crate::dyn_view_ptr::DynViewPtr;
    use std::fmt::Debug;

    #[derive(Debug)]
    struct Example {
        inner: f64,
    }

    impl Default for Example {
        fn default() -> Self {
            Self {
                inner: std::f64::consts::E,
            }
        }
    }

    #[test]
    fn dyn_view_from_ptr() {
        let example = Example::default();
        println!("{:?}", example);
        let boxed_example = Box::new(example);
        println!("{:?}", boxed_example);
        let raw_pointee = Box::<Example>::into_raw(boxed_example);
        let dyn_view_example = DynViewPtr::<dyn Debug>::from_ptr(raw_pointee).unwrap();
        unsafe {
            drop(Box::from_raw(raw_pointee));
        }
        println!("{:?}", dyn_view_example);
    }
}

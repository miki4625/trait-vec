
// Unstable features
#![feature(unsize)] //Coercion
#![feature(ptr_metadata)]
#![feature(coerce_unsized)]
#![feature(dispatch_from_dyn)]

//For tests
#![feature(strict_provenance)]

extern crate core;

use std::slice::{Iter, SliceIndex};
use crate::dyn_view_ptr::DynViewPtr;
use crate::trait_vec::PolyPtrVec;

pub mod trait_vec;
pub mod dyn_view_ptr;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

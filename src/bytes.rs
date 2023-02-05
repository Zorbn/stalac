pub fn to_bytes<T: Sized>(slice: &[T]) -> &[u8] {
    unsafe {
        core::slice::from_raw_parts(
            slice.as_ptr() as *const u8,
            core::mem::size_of_val(slice),
        )
    }
}
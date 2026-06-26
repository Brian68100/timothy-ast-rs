pub fn do_if_some<T, F: FnOnce(T)>(val: Option<T>, op: F) {
    if let Some(some) = val {
        op(some);
    }
}

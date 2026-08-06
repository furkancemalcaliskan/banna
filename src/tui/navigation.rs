pub(super) fn vim_go_top(index: &mut usize) {
    *index = 0;
}

pub(super) fn vim_go_bottom(index: &mut usize, len: usize) {
    *index = len.saturating_sub(1);
}

pub(super) fn wrap_next(index: &mut usize, len: usize) {
    if len > 0 {
        *index = (*index + 1) % len;
    }
}

pub(super) fn wrap_prev(index: &mut usize, len: usize) {
    if len > 0 {
        *index = (*index + len - 1) % len;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_navigation_wraps_and_handles_empty_lists() {
        let mut index = 0;
        wrap_prev(&mut index, 3);
        assert_eq!(index, 2);
        wrap_next(&mut index, 3);
        assert_eq!(index, 0);
        vim_go_bottom(&mut index, 0);
        assert_eq!(index, 0);
    }
}

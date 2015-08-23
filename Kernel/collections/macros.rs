macro_rules! linked_node {
    ($node: ty { prev: $prev: ident, next: $next: ident }) => {
            #[inline(always)]
            fn get_prev(&self) -> *mut $node {
                self.$prev
            }

            #[inline(always)]
            fn set_prev(&mut self, node: *mut $node) {
                self.$prev = node;
            }

            #[inline(always)]
            fn get_next(&self) -> *mut $node {
                self.$next
            }

            #[inline(always)]
            fn set_next(&mut self, node: *mut $node) {
                self.$next = node;
            }
    };
}

macro_rules! impl_linked_node {
    ($node: ty { prev: $prev: ident, next: $next: ident }) => {
        impl $crate::collections::linked_list::LinkedNode<$node> for $node {
            linked_node!($node { prev: $prev, next: $next });
        }
    };
}


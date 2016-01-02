macro_rules! linked_node {
    ($linker:ty { prev: $prev:ident, next: $next:ident }) => {
            type Linker = $linker;

            #[inline(always)]
            fn get_prev(&self) -> Option<$linker> {
                self.$prev.clone()
            }

            #[inline(always)]
            fn set_prev(&mut self, node: Option<$linker>) {
                self.$prev = node;
            }

            #[inline(always)]
            fn get_next(&self) -> Option<$linker> {
                self.$next.clone()
            }

            #[inline(always)]
            fn set_next(&mut self, node: Option<$linker>) {
                self.$next = node;
            }
    };
}

macro_rules! impl_linked_node {
    ($linker:ty { prev: $prev:ident, next: $next:ident }) => {
        impl $crate::lists::linked_list::LinkedNode for <$linker as $crate::lists::linked_list::Linker>::Node
        {
            linked_node!($linker { prev: $prev, next: $next });
        }
    };
}


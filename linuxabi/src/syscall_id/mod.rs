// syscall ids

macro_rules! define_syscall_ids {
    ( $( $name: ident = $num: expr ),+ ) => {
        $(
            #[allow(unused)]
            pub const $name: usize = $num;
        )*
    }
}

pub mod x86_64;
pub mod generic;

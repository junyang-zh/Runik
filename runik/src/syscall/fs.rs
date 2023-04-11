//! File and filesystem-related syscalls

const FD_STDOUT: usize = 1;

/// write buf of length `len`  to a file with `fd`
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let bufs = crate::mm::addr_space::kspace_from_user_buffer(buf, len);
            for buf_i in bufs {
                unsafe {
                    let slice = core::slice::from_raw_parts(buf_i.as_ptr(), buf_i.len());
                    let str = core::str::from_utf8(slice).unwrap();
                    print!("{}", str);
                }
            };
            len as isize
        }
        _ => {
            panic!("Illegal fd ({}) in sys_write!", fd);
        }
    }
}

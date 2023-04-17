//! File and filesystem-related syscalls

use crate::sbi::console_getchar;

const FD_STDIN: usize = 0;
const FD_STDOUT: usize = 1;

/// read a byte and place it to the buffer
pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    if len == 0 {
        return -1;
    }
    match fd {
        FD_STDIN => {
            let mut bufs = crate::mm::addr_space::kspace_from_user_buffer(buf, len);
            let mut cnt: usize = 0;
            let mut i: usize = 0;
            let mut buf_st: usize = 0;
            while cnt < len {
                match console_getchar() {
                    0 => { break; },
                    ch => {
                        let ch_u8 = match u8::try_from(ch) {
                            Ok(val) => val,
                            Err(_err) => { return -1 },
                        };
                        bufs[i][cnt - buf_st] = ch_u8;
                        cnt += 1;
                        if cnt - buf_st >= bufs[i].len() {
                            buf_st = cnt;
                            i += 1;
                        }
                    },
                }
            }
            cnt as isize
        }
        _ => {
            panic!("Illegal fd ({}) in sys_read!", fd);
        }
    }
}

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

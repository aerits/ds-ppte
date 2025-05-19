/// ergonomic bindings to draw to framebuffer with cpu
/// not useful

const SCREEN_HEIGHT: usize = 240;
const SCREEN_WIDTH: usize = 400;
const PTR_LEN: usize = SCREEN_HEIGHT * SCREEN_WIDTH * 3;

struct Color {
    r: u8,
    g: u8,
    b: u8,
}

struct FrameBuffer<'a> {
    fb: RawFrameBuffer<'a>,
    arr: [u8; PTR_LEN],
}

impl<'a> FrameBuffer<'a> {
    fn from_raw(fb: RawFrameBuffer<'a>) -> FrameBuffer<'a> {
        return FrameBuffer {
            fb,
            arr: [0; PTR_LEN],
        };
    }
    fn fill_screen(&mut self, color: Color) {
        for i in 0..SCREEN_WIDTH {
            for j in 0..SCREEN_HEIGHT {
                let index = (i * SCREEN_HEIGHT + j) * 3;
                self.arr[index] = color.b;
                self.arr[index + 1] = color.g;
                self.arr[index + 2] = color.r;
            }
        }
    }
    fn draw_circle(&mut self, color: Color, x: isize, y: isize, radius: usize) {
        for i in 0..SCREEN_WIDTH {
            for j in 0..SCREEN_HEIGHT {
                let index = (i * SCREEN_HEIGHT + j) * 3;
                let dist = distance(x, y, j as isize, i as isize) as usize;
                if dist < radius * radius {
                    self.arr[index] = color.b;
                    self.arr[index + 1] = color.g;
                    self.arr[index + 2] = color.r;
                }
            }
        }
    }
}
impl<'a> Drop for FrameBuffer<'a> {
    fn drop(&mut self) {
        unsafe {
            self.fb
                .ptr
                .copy_from_nonoverlapping(self.arr.as_ptr(), self.arr.len());
        }
    }
}

fn distance(x1: isize, y1: isize, x2: isize, y2: isize) -> isize {
    let dx = x1 - x2;
    let dy = y1 - y2;
    return dx * dx + dy * dy;
}

use std::io;

pub struct MessageReader {

}

impl io::Read for MessageReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.len() > 0 {
            buf[0] = 0;
            Ok(1)
        } else {
            Ok(0)
        }
    }
}

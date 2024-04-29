pub mod zmessage;
pub mod vlc;
pub mod bussiness;
pub mod zp2p;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use crate::zmessage::ZMessage;
    use super::*;

    #[test]
    fn send_message() {
        let mut msg = ZMessage::default();
        msg.data = vec![1];
    }
}
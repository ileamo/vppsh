use crate::VppSh;

const VPP_PREFIX: &[u8] = "vpp# ".as_bytes();
const VPP_PREFIX_LEN: usize = VPP_PREFIX.len();

impl VppSh<'_> {
    pub fn collect_history(&mut self, n: usize) {
        for c in &self.response[0..n] {
            if self.vpp_prefix {
                match c {
                    10 | 13 => {
                        if self.curr_command_len > VPP_PREFIX_LEN && self.was_enter {
                            println!(
                                "({})\r",
                                String::from_utf8_lossy(
                                    &self.curr_command[VPP_PREFIX_LEN..self.curr_command_len]
                                )
                                .trim()
                            );
                        }
                        self.curr_command_ptr = 0;
                        self.curr_command_len = 0;
                        self.was_enter = false;
                    }
                    8 => {
                        self.curr_command_ptr -= 1;
                    }
                    0..=31 => {}
                    c => {
                        self.curr_command[self.curr_command_ptr] = *c;
                        self.curr_command_ptr += 1;
                        if self.curr_command_ptr > self.curr_command_len {
                            self.curr_command_len = self.curr_command_ptr;
                        }
                        if self.curr_command_len == VPP_PREFIX_LEN
                            && &self.curr_command[0..VPP_PREFIX_LEN] != VPP_PREFIX
                        {
                            self.vpp_prefix = false;
                            self.curr_command_ptr = 0;
                            self.curr_command_len = 0;
                        }
                    }
                }
            } else if c == &(10 as u8) {
                self.vpp_prefix = true;
            }
        }
    }
}

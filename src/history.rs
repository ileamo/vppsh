const CURR_COMMAND_LEN: usize = 4096;
const VPP_PREFIX: &[u8] = "vpp# ".as_bytes();
const VPP_PREFIX_LEN: usize = VPP_PREFIX.len();

pub struct History {
    pub list: Vec<String>,
    pub conf: Vec<String>,
    curr_command: [u8; CURR_COMMAND_LEN],
    curr_command_ptr: usize,
    curr_command_len: usize,
    vpp_prefix: bool,
    was_enter: bool,
    selected: usize,
    conf_selected: usize,
    active: 
}

impl History {
    pub fn new() -> History {
        History {
            list: Vec::new(),
            conf: Vec::new(),
            curr_command: [0; CURR_COMMAND_LEN],
            curr_command_ptr: 0,
            curr_command_len: 0,
            vpp_prefix: true,
            was_enter: false,
            selected: 0,
            conf_selected: 0,
        }
    }

    pub fn get_selected(&self) -> usize {
        self.selected
    }

    pub fn down(&mut self) {
        if self.list.len() > self.selected + 1 {
            self.selected += 1;
        }
    }

    pub fn up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn reset_curr_comand(&mut self) {
        self.curr_command_ptr = 0;
        self.curr_command_len = 0;
    }

    pub fn was_enter(&mut self) {
        self.was_enter = true;
    }

    pub fn collect_history(&mut self, response: &[u8]) {
        for c in response {
            if self.vpp_prefix {
                match c {
                    10 | 13 => {
                        if self.curr_command_len > VPP_PREFIX_LEN && self.was_enter {
                            let hist = &self.curr_command[VPP_PREFIX_LEN..self.curr_command_len];
                            self.list.push(String::from_utf8_lossy(hist).to_string());
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

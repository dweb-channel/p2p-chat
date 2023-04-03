use idgenerator::{IdGeneratorOptions, IdInstance};

static P2P_CHAT_WORKER_ID: u32 = 666;

pub struct MessageIdGenerator;

impl MessageIdGenerator {
    pub fn init() {
        let options = IdGeneratorOptions::new()
            .worker_id(P2P_CHAT_WORKER_ID)
            .worker_id_bit_len(10)
            .base_time(1680529680348);

        IdInstance::set_options(options).unwrap();
    }

    pub fn next_id() -> i64 {
        IdInstance::next_id()
    }
}

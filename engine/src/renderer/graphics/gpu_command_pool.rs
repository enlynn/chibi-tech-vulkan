
use super::api::*;

pub struct CommandPoolFnTable {

}

pub struct CommandPool {
    pub fns:    CommandPoolFnTable,
    pub handle: VkCommandPool,
}

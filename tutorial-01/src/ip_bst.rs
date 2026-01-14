#[derive(Debug)]
pub struct BSTNode{
    prefix: u32,
    prefix_len: u8,
    next_hop: String,
    left: Option<Box<BSTNode>>,
    right: Option<Box<BSTNode>>,
}

impl BSTNode{
    pub fn new(prefix: u32, prefix_len: u8, next_hop: String) -> Self{
        BSTNode{
            prefix,
            prefix_len,
            next_hop,
            left: None,
            right: None,
        }
    }

    pub fn insert(&mut self, prefix: u32, prefix_len: u8, next_hop: String){
        if prefix < self.prefix{
            match &mut self.left {
                Some(node) => node.insert(prefix, prefix_len, next_hop),
                None => self.left = Some(Box::new(BSTNode::new(prefix, prefix_len, next_hop))),
            }
        }else{
            match &mut self.right{
                Some(node) => node.insert(prefix, prefix_len, next_hop),
                None => self.right = Some(Box::new(BSTNode::new(prefix, prefix_len, next_hop))),
            }
        }
    }

    pub fn matches(&self, ip: u32) -> bool{
        let mask = if self.prefix_len == 0{
            0
        }else{
            !0u32 << (32 - self.prefix_len)
        };

        (ip & mask) == (self.prefix & mask)
    }

    pub fn lookup(&self, ip: u32, best: &mut Option<String>, best_len: &mut i32){
        //check the curr node
        if self.matches(ip) && (self.prefix_len as i32) > *best_len{
            *best_len = self.prefix_len as i32;
            *best = Some(self.next_hop.clone());
        }

        //search both the subtrees
        if let Some(ref left) = self.left{
            left.lookup(ip, best, best_len);
        }
        if let Some(ref right) = self.right{
            right.lookup(ip, best, best_len);
        }
    }
}
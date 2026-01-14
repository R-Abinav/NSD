#[derive(Debug)]
pub struct TrieNode{
    left: Option<Box<TrieNode>>,
    right: Option<Box<TrieNode>>,
    next_hop: Option<String>
}

impl TrieNode{
    pub fn new() -> Self {
        TrieNode {
            left: None,
            right: None,
            next_hop: None,
        }
    }

    pub fn insert(&mut self, prefix: u32, prefix_len: u8, next_hop: String){
        let mut curr = self;

        for i in (32 - prefix_len..32).rev(){
            let bit = (prefix >> i) & 1;

            if bit == 0{
                curr = curr.left.get_or_insert_with(|| Box::new(TrieNode::new()));
            }else{
                curr = curr.right.get_or_insert_with(|| Box::new(TrieNode::new()));
            }
        }

        curr.next_hop = Some(next_hop);
    }

    pub fn lookup(&self, ip: u32) -> Option<String>{
        let mut curr = self;
        let mut result = None;

        for i in (0..32).rev(){
            //update result if curr node is valid prefix man
            if let Some(ref hop) = curr.next_hop {
                result = Some(hop.clone());
            }

            let bit = (ip >> i) & 1;

            curr = if bit == 0{
                match &curr.left{
                    Some(node) => node.as_ref(),
                    None => break,
                }
            }else{
                match &curr.right{
                    Some(node) => node.as_ref(),
                    None => break,
                }
            };
        }

        if let Some(ref hop) = curr.next_hop{
            result = Some(hop.clone());
        }

        return result;
    }
}

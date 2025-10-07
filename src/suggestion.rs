use std::collections::HashMap;

#[derive(Debug)]
pub struct PrefixTrieNode<T> {
    children: HashMap<char, PrefixTrieNode<T>>,
    values: Vec<T>,
}

impl<T> PrefixTrieNode<T> {
    fn new() -> Self {
        PrefixTrieNode {
            children: HashMap::new(),
            values: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct PrefixTrie<T> {
    root: PrefixTrieNode<T>,
    min_prefix_len: usize,
    max_prefix_len: usize,
    max_candidates: usize,
}

impl<T: Clone + PartialEq> PrefixTrie<T> {
    pub fn new(min_prefix_len: usize, max_prefix_len: usize, max_candidates: usize) -> Self {
        PrefixTrie {
            root: PrefixTrieNode::new(),
            min_prefix_len,
            max_prefix_len,
            max_candidates,
        }
    }

    pub fn insert(&mut self, key: &str, value: T) {
        if key.len() < self.min_prefix_len {
            return;
        }

        let lower_key = key.to_lowercase();
        
        for i in self.min_prefix_len..=lower_key.len() {
            let prefix_len = i.min(self.max_prefix_len);
            
            let prefix = &lower_key[..prefix_len];
            self.insert_prefix(prefix, value.clone());
        }
    }

    fn insert_prefix(&mut self, prefix: &str, value: T) {
        let mut current = &mut self.root;
        
        for c in prefix.chars() {
            current = current.children.entry(c).or_insert(PrefixTrieNode::new());
        }
        
        // Only add the value if it's not already in the vector and we haven't reached max_candidates
        if !current.values.contains(&value) && current.values.len() < self.max_candidates {
            current.values.push(value);
        }
    }

    pub fn search(&self, prefix: &str) -> Vec<T> {
        let lower_prefix = prefix.to_lowercase();
        
        let mut current = &self.root;
        
        for c in lower_prefix.chars() {
            match current.children.get(&c) {
                Some(child) => current = child,
                None => return Vec::new(), // No matches found
            }
        }
        
        current.values.to_vec()
    }
}

use crate::models::{Execution, SuggestedItem};

pub type ExecutionPrefixTrie = PrefixTrie<SuggestedItem>;

impl ExecutionPrefixTrie {
    // Build the trie from all execution names in the database
    pub async fn build_from_executions(
        pool: &sqlx::SqlitePool,
        min_prefix_len: usize,
        max_prefix_len: usize,
        max_candidates: usize,
    ) -> Result<Self, sqlx::Error> {
        let mut trie = PrefixTrie::new(min_prefix_len, max_prefix_len, max_candidates);
        
        let executions: Vec<Execution> = sqlx::query_as("SELECT * FROM execution ORDER BY time_created DESC")
            .fetch_all(pool)
            .await?;
        
        for execution in executions {
            let item = SuggestedItem {
                id: execution.id.unwrap_or(0).to_string(),
                name: execution.name.clone(),
            };
            trie.insert(&execution.name, item);
        }

        println!("Enable execution suggest api");
        
        Ok(trie)
    }
}
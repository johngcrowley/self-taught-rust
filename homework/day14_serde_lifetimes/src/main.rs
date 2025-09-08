// Rust Lifetimes Challenge: Document Cache System
//
// Your task is to fix all the lifetime issues in this code without changing
// the overall structure or functionality. The goal is to create a document
// cache that can store references to documents and return them efficiently.
//
// Rules:
// 1. You cannot change the function signatures (except adding lifetime parameters)
// 2. You cannot change the data structures (except adding lifetime parameters)
// 3. You cannot use Rc, Arc, or any smart pointers
// 4. All documents must be stored as references, not owned values
// 5. The cache should be able to return references to stored documents

use std::collections::HashMap;

// Document structure - represents a text document with metadata
#[derive(Debug, Clone)]
pub struct Document {
    pub id: &'a str,
    pub title: &'a str,
    pub content:  &'a str,
    pub author: &'a str,
}

// CHALLENGE 1: Fix the DocumentCache struct
// This cache should store references to documents, not owned documents
pub struct DocumentCache {
    documents: HashMap<String, &Document>, // This won't compile - fix the lifetimes!
    access_count: HashMap<String, u32>,
}

impl DocumentCache {
    // CHALLENGE 2: Fix the constructor
    pub fn new() -> DocumentCache {
        DocumentCache {
            documents: HashMap::new(),
            access_count: HashMap::new(),
        }
    }

    // CHALLENGE 3: Fix the insert method
    // Should store a reference to the document, not take ownership
    pub fn insert(&mut self, doc: &Document) {
        self.documents.insert(doc.id.clone(), doc);
        self.access_count.insert(doc.id.clone(), 0);
    }

    // CHALLENGE 4: Fix the get method
    // Should return a reference to the stored document reference
    pub fn get(&mut self, id: &str) -> Option<&Document> {
        if let Some(doc) = self.documents.get(id) {
            *self.access_count.get_mut(id).unwrap() += 1;
            Some(*doc)
        } else {
            None
        }
    }

    // CHALLENGE 5: Fix the get_most_accessed method
    // Should return the document that has been accessed the most
    pub fn get_most_accessed(&self) -> Option<&Document> {
        let most_accessed_id = self.access_count
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(id, _)| id)?;
        
        self.documents.get(most_accessed_id).map(|doc| *doc)
    }
}

// CHALLENGE 6: Fix the DocumentManager
// This should manage a collection of documents and provide a cache
pub struct DocumentManager {
    documents: Vec<Document>,
    cache: DocumentCache, // This won't work with current lifetimes
}

impl DocumentManager {
    pub fn new() -> DocumentManager {
        DocumentManager {
            documents: Vec::new(),
            cache: DocumentCache::new(),
        }
    }

    // CHALLENGE 7: Fix the add_document method
    pub fn add_document(&mut self, doc: Document) {
        self.documents.push(doc);
        // We want to cache the reference to the document we just added
        let last_doc = self.documents.last().unwrap();
        self.cache.insert(last_doc); // This won't compile due to lifetime issues
    }

    // CHALLENGE 8: Fix the find_document method
    pub fn find_document(&mut self, id: &str) -> Option<&Document> {
        // First try cache
        if let Some(doc) = self.cache.get(id) {
            return Some(doc);
        }
        
        // If not in cache, search documents and add to cache
        for doc in &self.documents {
            if doc.id == id {
                self.cache.insert(doc);
                return Some(doc);
            }
        }
        
        None
    }
}

// CHALLENGE 9: Fix the DocumentProcessor
// This struct should be able to process documents from multiple sources
pub struct DocumentProcessor {
    primary_manager: DocumentManager,
    secondary_docs: Vec<Document>,
    temp_cache: DocumentCache,
}

impl DocumentProcessor {
    pub fn new(primary: DocumentManager) -> DocumentProcessor {
        DocumentProcessor {
            primary_manager: primary,
            secondary_docs: Vec::new(),
            temp_cache: DocumentCache::new(),
        }
    }

    // CHALLENGE 10: Fix the cross_reference method
    // This method should be able to cache documents from both sources
    pub fn cross_reference(&mut self, id: &str) -> Option<&Document> {
        // Try primary manager first
        if let Some(doc) = self.primary_manager.find_document(id) {
            self.temp_cache.insert(doc);
            return Some(doc);
        }
        
        // Try secondary documents
        for doc in &self.secondary_docs {
            if doc.id == id {
                self.temp_cache.insert(doc);
                return Some(doc);
            }
        }
        
        None
    }
}

// Test functions - these should work once you fix the lifetimes
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_cache() {
        let doc = Document {
            id: "1".to_string(),
            title: "Test".to_string(),
            content: "Content".to_string(),
            author: "Author".to_string(),
        };
        
        let mut cache = DocumentCache::new();
        cache.insert(&doc);
        let retrieved = cache.get("1").unwrap();
        assert_eq!(retrieved.title, "Test");
    }

    #[test]
    fn test_document_manager() {
        let mut manager = DocumentManager::new();
        let doc = Document {
            id: "1".to_string(),
            title: "Test".to_string(),
            content: "Content".to_string(),
            author: "Author".to_string(),
        };
        
        manager.add_document(doc);
        let retrieved = manager.find_document("1").unwrap();
        assert_eq!(retrieved.title, "Test");
    }
}

// BONUS CHALLENGE: Can you explain why this particular combination of 
// lifetimes and borrowing is problematic? What fundamental Rust concepts
// are being violated, and what would be better architectural approaches?

fn main() {
    println!("Fix all the lifetime errors to make this compile!");
    println!("Hint: Think about how lifetimes relate to data ownership and borrowing rules.");
}

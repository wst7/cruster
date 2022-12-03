mod serializable;

use std::collections::HashMap;

use crate::siv_ui::ProxyDataForTable;
use super::cruster_proxy::request_response::{HyperRequestWrapper, HyperResponseWrapper};

#[derive(Clone, Debug)]
pub(super) struct RequestResponsePair {
    pub(super) request: Option<HyperRequestWrapper>,
    pub(super) response: Option<HyperResponseWrapper>,
    pub(super) index: usize,
}

pub(super) struct HTTPStorageIterator<'a> {
    object: &'a HTTPStorage,
    counter: usize
}

#[derive(Clone)]
pub(crate) struct HTTPStorage {
    storage: Vec<RequestResponsePair>,
    context_reference: HashMap<usize, usize>,
}

impl Default for HTTPStorage {
    fn default() -> Self {
        HTTPStorage {
            storage: Vec::with_capacity(1000),
            context_reference: HashMap::new(),
        }
    }
}

impl<'a> Iterator for HTTPStorageIterator<'a> {
    type Item = &'a RequestResponsePair;
    fn next(&mut self) -> Option<Self::Item> {
        let tmp = self.object.storage.get(self.counter);
        self.counter += 1;
        tmp
    }
}

impl<'a> IntoIterator for &'a HTTPStorage {
    type Item = &'a RequestResponsePair;
    type IntoIter = HTTPStorageIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        HTTPStorageIterator {
            object: self,
            counter: 0
        }
    }
}

// impl Iterator for HTTPStorage {
//     type Item = RequestResponsePair;
//     fn next(&mut self) -> Option<Self::Item> {
//         if self.iter_counter < self.storage.len() {
//             Some(&self.storage[self.iter_counter])
//         }
//         else {
//             self.iter_counter = 0;
//             None
//         }
//     }
// }

impl HTTPStorage {
    pub(crate) fn put_request(&mut self, request: HyperRequestWrapper, addr: usize) -> ProxyDataForTable {
        let index = self.storage.len();

        let table_record = ProxyDataForTable {
            id: index,
            hostname: request.get_host(),
            path: request.get_request_path(),
            method: request.method.clone(),
            status_code: String::default(),
            response_length: 0,
        };

        self.storage.push(
            RequestResponsePair {
                request: Some(request),
                response: None,
                index,
            }
        );

        self.context_reference.insert(addr, index);
        return table_record;
    }

    pub(crate) fn put_response(&mut self, response: HyperResponseWrapper, addr: &usize) -> Option<usize> {
        let mut index_found = None;

        if let Some(index) = self.context_reference.get(addr) {
            self.storage[index.to_owned()].response = Some(response);
            index_found = Some(self.storage[index.to_owned()].index);
        }
        
        return index_found;
    }

    pub(crate) fn get(&self, idx: usize) -> &RequestResponsePair {
        &self.storage[idx]
    }

    pub(crate) fn len(&self) -> usize {
        return self.storage.len().clone();
    }
}

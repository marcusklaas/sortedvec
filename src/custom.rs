
macro_rules! def_sorted_vec {
    // TODO: take visibility modifier
    ( $name:ident, $val:ty, $key:ty, $keygen:expr ) => {
        pub struct $name {
            inner: Vec<$val>,
        }

        impl $name {
            pub fn find<'t>(&'t self, key: &$key) -> Option<&'t $val> {
                self.inner
                    .binary_search_by(|probe| ($keygen)(probe).cmp(key))
                    .ok()
                    .and_then(|idx| self.inner.get(idx))
            }

            pub fn contains(&self, key: &$key) -> bool {
                self.inner
                    .binary_search_by(|probe| ($keygen)(probe).cmp(key))
                    .is_ok()
            }

            pub fn insert(&mut self, val: $val) {
                let ref key = ($keygen)(&val);
                let search = self
                    .inner
                    .binary_search_by(|probe| ($keygen)(probe).cmp(key));
                let idx = match search {
                    Ok(i) | Err(i) => i,
                };
                self.inner.insert(idx, val);
            }

            pub fn from_vec(mut vec: Vec<$val>) -> Self {
                vec.sort_unstable_by(|ref a, ref b| {
                    let lhs = ($keygen)(a);
                    let rhs = ($keygen)(b);
                    lhs.cmp(&rhs)
                });

                Self { inner: vec }
            }
        }

        impl std::iter::FromIterator<$val> for $name {
            fn from_iter<I: std::iter::IntoIterator<Item=$val>>(iter: I) -> Self {
                let inner = Vec::from_iter(iter);
                Self::from_vec(inner)
            }
        }

        // we can even impl Serialize and Deserialize for this type!
    }
}

def_sorted_vec! { TestVec, i32, i32, |x| x }

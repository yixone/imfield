use std::{collections::HashMap, hash::Hash};

pub trait Coalescable<T>: Clone {
    type Key: Clone + Hash + Eq;

    fn p_key(&self) -> &Self::Key;
    fn f_key(t: &T) -> &Self::Key;
}

pub fn unit_models<T, K>(l: Vec<T>, r: Vec<K>) -> Vec<(K, T)>
where
    T: Coalescable<K>,
{
    let idxs = l
        .into_iter()
        .map(|l| (l.p_key().to_owned(), l))
        .collect::<HashMap<_, _>>();

    r.into_iter()
        .filter_map(|r| {
            let entry = idxs.get(T::f_key(&r))?;
            Some((r, entry.clone()))
        })
        .collect()
}

pub fn unit_models_group<T, K>(l: Vec<T>, r: Vec<K>) -> Vec<(K, Vec<T>)>
where
    T: Coalescable<K>,
{
    let mut idxs: HashMap<<T as Coalescable<K>>::Key, Vec<T>> = HashMap::with_capacity(l.len());
    for l in l {
        match idxs.get_mut(l.p_key()) {
            Some(i) => {
                i.push(l);
            }
            None => {
                idxs.insert(l.p_key().to_owned(), vec![l]);
            }
        }
    }

    r.into_iter()
        .filter_map(|r| {
            let entry = idxs.get(T::f_key(&r))?;
            Some((r, entry.clone()))
        })
        .collect()
}

macro_rules! coalesced {
    ($p: path [ $( $pi: ident ).+ ] => $f: path [ $( $fi: ident ).+ ] as $t: ty) => {
        impl $crate::utils::coalesced::Coalescable<$f> for $p {
            type Key = $t;

            fn p_key(&self) -> &Self::Key {
                &self.$( $pi ).+
            }

            fn f_key(t: &$f) -> &Self::Key {
                &t.$( $fi ).+
            }
        }
    };
}

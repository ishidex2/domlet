use std::collections::btree_set::*;
type Id = usize;
#[derive(Debug)]
pub struct BucketArray<T>
{
    items: Vec<T>,
    free: BTreeSet<usize>
}

impl<T> BucketArray<T>
{
    pub fn new() -> Self
    {
        Self { items: vec![], free: BTreeSet::new() }
    }

    pub fn len(&self) -> usize
    {
        self.items.len()-self.free.len()
    }

    pub fn insert(&mut self, item: T) -> Id 
    {
        if self.free.len() > 0
        {
            let idx = *self.free.iter().next().unwrap();
            self.free.remove(&idx);
            self.items[idx] = item;
            idx
        }
        else 
        {
            self.items.push(item);
            self.items.len()-1
        }
    }

    pub fn get(&self, id: Id) -> Option<&T>
    {
        if self.free.contains(&id)
        {
            None
        }
        else 
        {
            self.items.get(id)
        }
    }

    pub fn has(&self, id: Id) -> bool
    {
        id < self.items.len() && !self.free.contains(&id)
    }
        

    pub fn get_mut(&mut self, id: Id) -> Option<&mut T>
    {
        if self.free.contains(&id)
        {
            None
        }
        else 
        {
            self.items.get_mut(id)
        }
    }

    // This assumes you won't need the item
    pub fn remove(&mut self, id: Id)
    {
        if self.has(id)
        {
            self.free.insert(id);
        }
        else
        {
            panic!("Removed non existent value from bucket_array");
        }
    }

    pub fn iter(&self) -> Iter<'_, T>
    {
        Iter { handle: self, id: 0 }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T>
    {
        IterMut { handle: self, id: 0 }
    }
}


pub struct IterMut<'a, T: 'a>
{
    handle: &'a mut BucketArray<T>,
    id: Id
}

impl<'a, T> Iterator for IterMut<'a, T>
{
    type Item = (Id, &'a mut T);

    fn next(&mut self) -> Option<Self::Item>
    {
        unsafe {
            while self.handle.free.contains(&self.id)
            {
                self.id += 1;  
            }

            let r = if self.id < self.handle.items.len()
            {
                self.handle.items.as_mut_ptr().offset(self.id as isize)
            }
            else 
            {
                return None;
            };

            let id = self.id;
            self.id += 1;
            Some((id, &mut *r))
        }
    }
}

pub struct Iter<'a, T>
{
    handle: &'a BucketArray<T>,
    id: Id
}

impl<'a, T> Iterator for Iter<'a, T>
{
    type Item = (Id, &'a T);

    fn next(&mut self) -> Option<(Id, &'a T)>
    {
        while self.handle.free.contains(&self.id)
        {
            self.id += 1;  
        }
        let r = self.handle.get(self.id);
        let id = self.id;
        self.id += 1;
        if let Some(x) = r 
        {
            Some((id, x))
        }
        else 
        {
            return None;
        }
    }
}

#[test]
fn test()
{
    let mut b = BucketArray::<usize>::new();
    b.insert(1);
    b.insert(4000);
    b.insert(5445);
    b.insert(342);
    let i1 = b.insert(343);
    let i2 = b.insert(23);
    b.remove(0);
    b.remove(4);
    assert!(b.has(i2));
    assert!(b.has(4345) == false);
    assert_eq!(b.get(i1), None);
    assert_eq!(b.get(434), None);
    assert_eq!(b.get(i2), Some(&23));
    *b.get_mut(i2).unwrap() = 0;
    assert_eq!(b.get_mut(23245345), None);
    assert_eq!(b.len(), 4);
    {
        let mut it = b.iter();

        let a1 = it.next().unwrap();
        let a2 = it.next().unwrap();
        let a3 = it.next().unwrap();
        let a4 = it.next().unwrap();
        assert_eq!(*a1.1, 4000);
        assert_eq!(a1.0, 1);
        assert_eq!(*a2.1, 5445);
        assert_eq!(a2.0, 2);
        assert_eq!(*a3.1, 342);
        assert_eq!(a3.0, 3);
        assert_eq!(*a4.1, 0);
        assert_eq!(a4.0, 5);
        assert_eq!(it.next(), None);
    }
    dbg!(&b);
    for (k, v) in b.iter_mut()
    {
        *v = 123;
    }
    b.insert(3);
    assert_eq!(b.len(), 5);
    {
        let mut it = b.iter();
        assert_eq!(*it.next().unwrap().1, 3);
        assert_eq!(*it.next().unwrap().1, 123);
        assert_eq!(*it.next().unwrap().1, 123);
        assert_eq!(*it.next().unwrap().1, 123);
        assert_eq!(*it.next().unwrap().1, 123);
        assert_eq!(it.next(), None);
    }
    dbg!(&b);
}

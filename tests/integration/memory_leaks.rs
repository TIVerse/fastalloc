//! Memory leak detection tests.

use fastalloc::FixedPool;

#[test]
fn test_no_leak_on_normal_usage() {
    let pool = FixedPool::<Vec<u8>>::new(10).unwrap();
    
    for _ in 0..100 {
        let handle = pool.allocate(vec![0u8; 1024]).unwrap();
        drop(handle);
    }
    
    assert_eq!(pool.allocated(), 0);
    assert_eq!(pool.available(), pool.capacity());
}

#[test]
fn test_no_leak_with_vec_of_handles() {
    let pool = FixedPool::<String>::new(50).unwrap();
    
    let mut handles = Vec::new();
    for i in 0..50 {
        handles.push(pool.allocate(format!("String {}", i)).unwrap());
    }
    
    assert_eq!(pool.allocated(), 50);
    
    handles.clear();
    
    assert_eq!(pool.allocated(), 0);
}

#[test]
fn test_no_leak_with_partial_drops() {
    let pool = FixedPool::<Vec<i32>>::new(20).unwrap();
    
    for _ in 0..10 {
        let mut handles = Vec::new();
        for i in 0..20 {
            handles.push(pool.allocate(vec![i; 10]).unwrap());
        }
        
        // Drop half
        handles.drain(0..10);
        assert_eq!(pool.allocated(), 10);
        
        // Drop rest
        handles.clear();
        assert_eq!(pool.allocated(), 0);
    }
}

#[test]
fn test_nested_allocation() {
    let pool = FixedPool::<Vec<Vec<u8>>>::new(10).unwrap();
    
    {
        let mut handle = pool.allocate(Vec::new()).unwrap();
        for _ in 0..10 {
            handle.push(vec![0u8; 100]);
        }
        
        assert_eq!(handle.len(), 10);
    }
    
    assert_eq!(pool.allocated(), 0);
}

#[test]
fn test_drop_in_different_scopes() {
    let pool = FixedPool::<String>::new(5).unwrap();
    
    {
        let h1 = pool.allocate(String::from("outer")).unwrap();
        {
            let h2 = pool.allocate(String::from("inner")).unwrap();
            assert_eq!(pool.allocated(), 2);
        }
        assert_eq!(pool.allocated(), 1);
    }
    assert_eq!(pool.allocated(), 0);
}

#[test]
fn test_leak_detection_with_clone() {
    #[derive(Clone)]
    struct Cloneable {
        data: Vec<u8>,
    }
    
    let pool = FixedPool::<Cloneable>::new(10).unwrap();
    
    {
        let handle = pool.allocate(Cloneable { data: vec![1, 2, 3] }).unwrap();
        let _cloned_data = handle.data.clone();
        // Handle itself is not cloned, only internal data
    }
    
    assert_eq!(pool.allocated(), 0);
}

use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::sync::Mutex;

use crate::Pool;

/// スレッドセーフなオブジェクトプール
pub type ThreadSafePool<T> = Pool<T, Mutex<Vec<T>>>;

// 便利なエイリアス型
pub type ThreadSafeVecPool<T> = ThreadSafePool<Vec<T>>;
pub type ThreadSafeHashMapPool<K, V> = ThreadSafePool<HashMap<K, V>>;
pub type ThreadSafeHashSetPool<T> = ThreadSafePool<HashSet<T>>;
pub type ThreadSafeStringPool = ThreadSafePool<String>;
pub type ThreadSafeVecDequePool<T> = ThreadSafePool<VecDeque<T>>;
pub type ThreadSafeBinaryHeapPool<T> = ThreadSafePool<BinaryHeap<T>>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;

    #[test]
    fn test_threadsafe_vec_pool() {
        let pool: ThreadSafeVecPool<i32> = ThreadSafeVecPool::new();

        {
            let mut vec = pool.get();
            vec.push(1);
            vec.push(2);
            vec.push(3);
            assert_eq!(&*vec, &vec![1, 2, 3]);
        }

        assert_eq!(pool.pool_size().unwrap(), 1);

        {
            let vec = pool.get();
            assert_eq!(vec.len(), 0); // クリアされている
        }
    }

    #[test]
    fn test_multithread_usage() {
        let pool = Arc::new(ThreadSafeVecPool::<i32>::new());
        let counter = Arc::new(AtomicUsize::new(0));

        let handles: Vec<_> = (0..10)
            .map(|i| {
                let pool = Arc::clone(&pool);
                let counter = Arc::clone(&counter);

                thread::spawn(move || {
                    for j in 0..100 {
                        let mut vec = pool.get();
                        vec.push(i * 100 + j);
                        vec.push(i * 100 + j + 1);
                        assert_eq!(vec.len(), 2);
                        counter.fetch_add(1, Ordering::Relaxed);
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(counter.load(Ordering::Relaxed), 1000);
        // マルチスレッドで使用後、いくつかのバッファがプールに残っている
        assert!(pool.pool_size().unwrap() > 0);
    }

    #[test]
    fn test_clone_pool() {
        let pool1: Arc<ThreadSafeVecPool<i32>> = Arc::new(ThreadSafeVecPool::new());
        let pool2 = Arc::clone(&pool1); // 同じプールを参照

        {
            let mut vec = pool1.get();
            vec.push(42);
        }

        assert_eq!(pool1.pool_size().unwrap(), 1);
        assert_eq!(pool2.pool_size().unwrap(), 1); // 同じプールなので同じサイズ

        {
            let vec = pool2.get(); // pool2から取得
            assert_eq!(vec.len(), 0); // クリアされている
        }

        assert_eq!(pool1.pool_size().unwrap(), 1); // pool1も影響を受ける
    }

    #[test]
    fn test_prewarm() {
        let pool: ThreadSafeVecPool<i32> = ThreadSafeVecPool::new();

        assert_eq!(pool.pool_size().unwrap(), 0);

        pool.prewarm(5).unwrap();
        assert_eq!(pool.pool_size().unwrap(), 5);

        // プリウォームされたオブジェクトを使用
        {
            let _vec1 = pool.get();
            let _vec2 = pool.get();
            assert_eq!(pool.pool_size().unwrap(), 3); // 2つ使用中、3つプールに残存
        }

        assert_eq!(pool.pool_size().unwrap(), 5); // 使用終了で元に戻る
    }

    #[test]
    fn test_threadsafe_hashmap_pool() {
        let pool: ThreadSafeHashMapPool<String, i32> = ThreadSafeHashMapPool::new();

        {
            let mut map = pool.get();
            map.insert("key1".to_string(), 42);
            map.insert("key2".to_string(), 84);
            assert_eq!(map.len(), 2);
        }

        assert_eq!(pool.pool_size().unwrap(), 1);

        {
            let map = pool.get();
            assert_eq!(map.len(), 0); // クリアされている
        }
    }

    #[test]
    fn test_concurrent_different_pools() {
        let vec_pool = Arc::new(ThreadSafeVecPool::<i32>::new());
        let map_pool = Arc::new(ThreadSafeHashMapPool::<String, i32>::new());
        let str_pool = Arc::new(ThreadSafeStringPool::new());

        let handles: Vec<_> = (0..5)
            .map(|i| {
                let vec_pool = Arc::clone(&vec_pool);
                let map_pool = Arc::clone(&map_pool);
                let str_pool = Arc::clone(&str_pool);

                thread::spawn(move || {
                    let mut vec = vec_pool.get();
                    let mut map = map_pool.get();
                    let mut s = str_pool.get();

                    vec.push(i);
                    map.insert(format!("key{i}"), i);
                    s.push_str(&format!("thread{i}"));

                    assert_eq!(vec.len(), 1);
                    assert_eq!(map.len(), 1);
                    assert!(!s.is_empty());
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        // 各プールにオブジェクトが返却されている
        assert!(vec_pool.pool_size().unwrap() > 0);
        assert!(map_pool.pool_size().unwrap() > 0);
        assert!(str_pool.pool_size().unwrap() > 0);
    }

    #[test]
    fn test_stress_test() {
        let pool = Arc::new(ThreadSafeVecPool::<usize>::new());
        let iterations = 1000;
        let thread_count = 10;

        let handles: Vec<_> = (0..thread_count)
            .map(|thread_id| {
                let pool = Arc::clone(&pool);

                thread::spawn(move || {
                    for i in 0..iterations {
                        let mut vec = pool.get();

                        // 異なるサイズのデータを追加
                        for j in 0..(i % 10) {
                            vec.push(thread_id * iterations + i * 10 + j);
                        }

                        assert!(vec.len() <= 10);
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        // ストレステスト後もプールが正常に動作
        {
            let mut vec = pool.get();
            vec.push(999);
            assert_eq!(vec[0], 999);
        }
    }
}

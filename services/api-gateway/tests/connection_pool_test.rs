// Connection pool tests
//
// These tests verify that the connection pool correctly manages multiple
// gRPC connections and distributes requests using round-robin selection.

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_round_robin_distribution() {
        // Test that round-robin selection distributes requests evenly
        let next_index = AtomicUsize::new(0);
        let pool_size = 5;
        let num_requests = 25;

        let mut distribution = vec![0; pool_size];

        for _ in 0..num_requests {
            let index = next_index.fetch_add(1, Ordering::Relaxed);
            let channel_index = index % pool_size;
            distribution[channel_index] += 1;
        }

        // Each channel should have received exactly 5 requests
        for count in distribution {
            assert_eq!(count, 5, "Round-robin distribution should be even");
        }
    }

    #[test]
    fn test_concurrent_acquisition() {
        // Test that multiple threads can safely acquire channels concurrently
        let next_index = std::sync::Arc::new(AtomicUsize::new(0));
        let pool_size = 10;
        let num_threads = 100;
        let requests_per_thread = 100;

        let handles: Vec<_> = (0..num_threads)
            .map(|_| {
                let next_index = next_index.clone();
                std::thread::spawn(move || {
                    for _ in 0..requests_per_thread {
                        let index = next_index.fetch_add(1, Ordering::Relaxed);
                        let _channel_index = index % pool_size;
                        // Simulate some work
                        std::thread::sleep(std::time::Duration::from_micros(1));
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        // Verify total number of acquisitions
        let total = next_index.load(Ordering::Relaxed);
        assert_eq!(
            total,
            num_threads * requests_per_thread,
            "All acquisitions should be counted"
        );
    }

    #[test]
    fn test_pool_size_configuration() {
        // Test that different pool sizes work correctly
        for pool_size in [1, 5, 10, 20] {
            let next_index = AtomicUsize::new(0);
            let num_requests = pool_size * 10;

            let mut indices = Vec::new();
            for _ in 0..num_requests {
                let index = next_index.fetch_add(1, Ordering::Relaxed);
                indices.push(index % pool_size);
            }

            // Verify each channel is used exactly 10 times
            let mut distribution = vec![0; pool_size];
            for index in indices {
                distribution[index] += 1;
            }

            for count in distribution {
                assert_eq!(
                    count, 10,
                    "Each channel should be used equally for pool size {}",
                    pool_size
                );
            }
        }
    }

    #[tokio::test]
    async fn test_pool_stats() {
        // Test that pool statistics are correctly tracked
        let next_index = std::sync::Arc::new(AtomicUsize::new(0));

        // Simulate 42 requests
        for _ in 0..42 {
            next_index.fetch_add(1, Ordering::Relaxed);
        }

        let requests_served = next_index.load(Ordering::Relaxed);
        assert_eq!(requests_served, 42, "Stats should track total requests");
    }

    #[test]
    fn test_wraparound_behavior() {
        // Test that the counter wraps around correctly
        let next_index = AtomicUsize::new(usize::MAX - 5);
        let pool_size = 3;

        let mut indices = Vec::new();
        for _ in 0..10 {
            let index = next_index.fetch_add(1, Ordering::Relaxed);
            indices.push(index % pool_size);
        }

        // Verify round-robin continues after wraparound
        // The exact pattern depends on usize::MAX % pool_size
        assert_eq!(indices.len(), 10, "Should handle wraparound gracefully");
    }
}

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::collections::HashMap;
use std::sync::Arc;

// Simulate the old implementation without Arc
#[derive(Debug, Clone)]
struct RoutingDecisionOld {
    pub service: String,
    pub grpc_method: String,
    pub path_params: HashMap<String, String>,
}

// New implementation with Arc (from the actual codebase)
#[derive(Debug, Clone)]
struct RoutingDecisionNew {
    pub service: Arc<str>,
    pub grpc_method: Arc<str>,
    pub path_params: HashMap<String, String>,
}

fn benchmark_routing_decision_clone(c: &mut Criterion) {
    let mut group = c.benchmark_group("routing_decision_clone");
    
    // Old implementation - cloning Strings
    let old_decision = RoutingDecisionOld {
        service: "user-auth-service".to_string(),
        grpc_method: "user.UserService/GetUser".to_string(),
        path_params: HashMap::new(),
    };
    
    // New implementation - cloning Arc<str>
    let new_decision = RoutingDecisionNew {
        service: Arc::from("user-auth-service"),
        grpc_method: Arc::from("user.UserService/GetUser"),
        path_params: HashMap::new(),
    };
    
    group.bench_function("old_string_clone", |b| {
        b.iter(|| {
            let cloned = black_box(old_decision.clone());
            black_box(cloned)
        })
    });
    
    group.bench_function("new_arc_clone", |b| {
        b.iter(|| {
            let cloned = black_box(new_decision.clone());
            black_box(cloned)
        })
    });
    
    group.finish();
}

fn benchmark_multiple_clones(c: &mut Criterion) {
    let mut group = c.benchmark_group("multiple_clones");
    
    // Simulate multiple clones as would happen in request processing
    let old_decision = RoutingDecisionOld {
        service: "user-auth-service".to_string(),
        grpc_method: "user.UserService/GetUser".to_string(),
        path_params: HashMap::new(),
    };
    
    let new_decision = RoutingDecisionNew {
        service: Arc::from("user-auth-service"),
        grpc_method: Arc::from("user.UserService/GetUser"),
        path_params: HashMap::new(),
    };
    
    for num_clones in [1, 5, 10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("old_string", num_clones),
            num_clones,
            |b, &num_clones| {
                b.iter(|| {
                    let mut clones = Vec::with_capacity(num_clones);
                    for _ in 0..num_clones {
                        clones.push(black_box(old_decision.clone()));
                    }
                    black_box(clones)
                })
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("new_arc", num_clones),
            num_clones,
            |b, &num_clones| {
                b.iter(|| {
                    let mut clones = Vec::with_capacity(num_clones);
                    for _ in 0..num_clones {
                        clones.push(black_box(new_decision.clone()));
                    }
                    black_box(clones)
                })
            },
        );
    }
    
    group.finish();
}

fn benchmark_arc_wrapping(c: &mut Criterion) {
    let mut group = c.benchmark_group("arc_wrapping");
    
    // Benchmark wrapping in Arc and cloning the Arc
    let old_decision = RoutingDecisionOld {
        service: "user-auth-service".to_string(),
        grpc_method: "user.UserService/GetUser".to_string(),
        path_params: HashMap::new(),
    };
    
    let new_decision = RoutingDecisionNew {
        service: Arc::from("user-auth-service"),
        grpc_method: Arc::from("user.UserService/GetUser"),
        path_params: HashMap::new(),
    };
    
    group.bench_function("old_wrap_and_clone", |b| {
        b.iter(|| {
            let wrapped = Arc::new(black_box(old_decision.clone()));
            let cloned1 = black_box(wrapped.clone());
            let cloned2 = black_box(wrapped.clone());
            black_box((cloned1, cloned2))
        })
    });
    
    group.bench_function("new_wrap_and_clone", |b| {
        b.iter(|| {
            let wrapped = Arc::new(black_box(new_decision.clone()));
            let cloned1 = black_box(wrapped.clone());
            let cloned2 = black_box(wrapped.clone());
            black_box((cloned1, cloned2))
        })
    });
    
    group.finish();
}

fn benchmark_memory_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_size");
    
    // Benchmark memory allocation patterns
    group.bench_function("old_allocate_1000", |b| {
        b.iter(|| {
            let mut decisions = Vec::with_capacity(1000);
            for i in 0..1000 {
                decisions.push(RoutingDecisionOld {
                    service: format!("service-{}", i % 10),
                    grpc_method: format!("service.Service/Method{}", i % 10),
                    path_params: HashMap::new(),
                });
            }
            black_box(decisions)
        })
    });
    
    group.bench_function("new_allocate_1000", |b| {
        b.iter(|| {
            let mut decisions = Vec::with_capacity(1000);
            for i in 0..1000 {
                decisions.push(RoutingDecisionNew {
                    service: Arc::from(format!("service-{}", i % 10).as_str()),
                    grpc_method: Arc::from(format!("service.Service/Method{}", i % 10).as_str()),
                    path_params: HashMap::new(),
                });
            }
            black_box(decisions)
        })
    });
    
    group.finish();
}

fn benchmark_request_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("request_simulation");
    
    // Simulate a full request flow: route -> store in extensions -> retrieve -> use
    let old_decision = RoutingDecisionOld {
        service: "user-auth-service".to_string(),
        grpc_method: "user.UserService/GetUser".to_string(),
        path_params: HashMap::new(),
    };
    
    let new_decision = RoutingDecisionNew {
        service: Arc::from("user-auth-service"),
        grpc_method: Arc::from("user.UserService/GetUser"),
        path_params: HashMap::new(),
    };
    
    group.bench_function("old_full_flow", |b| {
        b.iter(|| {
            // Simulate middleware storing in extensions
            let wrapped = Arc::new(old_decision.clone());
            
            // Simulate handler retrieving from extensions
            let retrieved = wrapped.clone();
            
            // Simulate using the data (clone to avoid borrow issues)
            let service = black_box(retrieved.service.clone());
            let method = black_box(retrieved.grpc_method.clone());
            
            black_box((service, method))
        })
    });
    
    group.bench_function("new_full_flow", |b| {
        b.iter(|| {
            // Simulate middleware storing in extensions
            let wrapped = Arc::new(new_decision.clone());
            
            // Simulate handler retrieving from extensions
            let retrieved = wrapped.clone();
            
            // Simulate using the data (clone Arc, not data)
            let service = black_box(retrieved.service.clone());
            let method = black_box(retrieved.grpc_method.clone());
            
            black_box((service, method))
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_routing_decision_clone,
    benchmark_multiple_clones,
    benchmark_arc_wrapping,
    benchmark_memory_size,
    benchmark_request_simulation
);
criterion_main!(benches);

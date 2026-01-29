use ot_core::TransformService;

#[test]
fn test_registry_coverage() {
    let service = TransformService::new();
    let size = service.registry_size();
    
    println!("Total registered transforms: {}", size);
    
    // We should have many transforms registered
    assert!(size > 50, "Expected at least 50 transforms registered, got {}", size);
}

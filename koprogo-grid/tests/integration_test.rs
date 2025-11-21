use koprogo_grid::core::{Node, Task, TaskType};

#[test]
fn test_node_creation() {
    let node = Node::new(
        "TestNode".to_string(),
        4,
        true,
        "Brussels".to_string(),
    );

    assert!(node.is_ok());
    let node = node.unwrap();
    assert_eq!(node.name, "TestNode");
    assert_eq!(node.cpu_cores, 4);
    assert!(node.has_solar);
}

#[test]
fn test_task_creation() {
    let task = Task::new(
        TaskType::MlTrain,
        "s3://bucket/data.csv".to_string(),
        60,
    );

    assert!(task.is_ok());
    let task = task.unwrap();
    assert_eq!(task.task_type, TaskType::MlTrain);
}

#[test]
fn test_eco_score_calculation() {
    let mut node = Node::new(
        "TestNode".to_string(),
        4,
        true,
        "Brussels".to_string(),
    )
    .unwrap();

    // 80% idle CPU, 500W solar
    node.update_eco_score(20.0, 500.0);

    // Expected: (0.8 * 0.5) + (0.5 * 0.5) = 0.65
    assert!((node.eco_score - 0.65).abs() < 0.01);
}

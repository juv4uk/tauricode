use swarm_cli::adapters;
use swarm_cli::minijson;

#[test]
fn gh_issue_array_maps_labels_state_and_ids() {
    let raw = r#"[
      {"number":7,"title":"add SLP1 bridge","state":"open",
       "labels":[{"name":"sanskrit"},{"name":"rust"}],"body":"bridge notes"},
      {"number":8,"title":"typo","state":"closed","labels":[]}
    ]"#;
    let tasks = adapters::tasks_from_gh_json(raw, None, 3.0).unwrap();
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0].id, "GH-7");
    assert!(tasks[0].capabilities.contains(&"rust".to_string()));
    assert!(!tasks[0].done);
    assert_eq!(tasks[0].description.as_deref(), Some("add SLP1 bridge\n\nbridge notes"));
    assert!(tasks[1].done);
}

#[test]
fn md_frontmatter_note_becomes_task() {
    let note = "---\ntitle: Bridge unblock\ntags: [docs, sanskrit]\npriority: 4\ndepends-on: [PANINI-A]\ndone: false\n---\n\nSome body line about the work.\n";
    let t = adapters::task_from_md_note(note).unwrap();
    assert_eq!(t.id, "BRIDGE-UNBLOCK");
    assert_eq!(t.priority, 4.0);
    assert!(t.capabilities.contains(&"docs".to_string()));
    assert_eq!(t.depends_on, vec!["PANINI-A".to_string()]);
    assert_eq!(t.description.as_deref(), Some("Some body line about the work."));
    assert!(!t.done);
}

#[test]
fn explicit_task_id_beats_title_slug() {
    let note = "---\nid: MY-ID\ntitle: Something else\n---\n";
    assert_eq!(adapters::task_from_md_note(note).unwrap().id, "MY-ID");
}

#[test]
fn md_export_then_reimport_round_trips_core_fields() {
    use swarm_cli::tasks_file::ParsedTask;
    let t = ParsedTask {
        id: "ROUND-TRIP".into(),
        priority: 5.5,
        capabilities: vec!["a".into(), "b".into()],
        depends_on: vec![],
        done: true,
        description: Some("desc".into()),
        origin: Some("cml".into()),
    };
    let md = adapters::tasks_to_md_notes(std::slice::from_ref(&t));
    let back = adapters::task_from_md_note(&md).unwrap();
    assert_eq!(back.id, t.id);
    assert_eq!(back.priority, t.priority);
    assert!(back.capabilities == t.capabilities && back.done && back.origin == t.origin);
}

#[test]
fn json_tasks_import() {
    let raw = r#"{"tasks":[{"id":"X","priority":2,"capabilities":["r"],"depends_on":["Y"],"done":true,"origin":"cml"}]}"#;
    let tasks = adapters::tasks_from_json(raw).unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].id, "X");
    assert!(tasks[0].done);
    assert_eq!(tasks[0].origin.as_deref(), Some("cml"));
}

#[test]
fn json_to_out_is_faithful() {
    let j = minijson::parse(r#"{"a":[1,"b",null,true]}"#).unwrap();
    let o = adapters::json_to_out(&j);
    let rendered = swarm_cli::out::to_json(&o);
    assert!(rendered.contains("\"a\":[1,\"b\",null,true]"));
}

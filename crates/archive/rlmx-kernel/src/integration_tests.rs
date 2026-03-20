//! Integration tests for cross-subsystem interactions within the kernel.
//!
//! These tests exercise the kernel's major subsystems (scheduler, memory, graph,
//! process manager, capability manager, proof engine) together through the
//! `dispatch()` function and direct API calls, verifying the contracts
//! documented in ADR-001 through ADR-010 and DDD-002 through DDD-004.

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use tokio::sync::Mutex;

    use crate::capability::CapabilityManager;
    use crate::graph::Graph;
    use crate::memory::{cosine_similarity, text_to_embedding, MemoryRegion, EMBED_DIM};
    use crate::process::ProcessManager;
    use crate::proof::ProofEngine;
    use crate::scheduler::{Scheduler, SchedulerConfig, Strategy};
    use crate::syscall::{dispatch, KernelContext, Syscall};
    use crate::types::{
        Capability, KernelMessage, MinCutAlgorithm, ProofRequest, SearchFilters,
        SegmentMetadata, SyscallPermission, SyscallResult,
    };
    use chrono::Duration;
    use uuid::Uuid;

    // -----------------------------------------------------------------------
    // Helper: build a fully wired KernelContext
    // -----------------------------------------------------------------------

    fn build_kernel_context() -> KernelContext {
        KernelContext {
            memory: Arc::new(Mutex::new(MemoryRegion::new("integration-test"))),
            graph: Arc::new(Mutex::new(Graph::new())),
            process_manager: Arc::new(Mutex::new(ProcessManager::new())),
            proof_engine: Arc::new(Mutex::new(ProofEngine::new())),
            capability_manager: Arc::new(Mutex::new(CapabilityManager::new())),
            caller_pid: None,
            event_bus: None,
        }
    }

    fn make_meta(source: &str) -> SegmentMetadata {
        SegmentMetadata {
            source: source.into(),
            plugin: None,
            segment_type: "text".into(),
            extra: Default::default(),
        }
    }

    // =======================================================================
    // Scheduler tests
    // =======================================================================

    /// Verify that `Strategy::Auto` resolves to concrete strategies via
    /// the heuristic router. ADR-003 documents that Auto should never be
    /// returned as-is from `resolve_strategy`.
    #[test]
    fn test_scheduler_heuristic_strategies() {
        let scheduler = Scheduler::default();

        // Short question -> Rlm
        let s = scheduler.resolve_strategy("What is Rust?", None);
        assert!(
            matches!(s, Strategy::Rlm),
            "Short question should resolve to Rlm, got {:?}",
            s
        );

        // Long code block -> Trm
        let code_query = format!("Please review this code:\n```\n{}\n```", "x".repeat(600));
        let s = scheduler.resolve_strategy(&code_query, None);
        assert!(
            matches!(s, Strategy::Trm(_)),
            "Code query should resolve to Trm, got {:?}",
            s
        );

        // Medium-length statement -> Hybrid
        let medium = "Explain the trade-offs between microservices and monolithic architecture in detail.";
        let s = scheduler.resolve_strategy(medium, None);
        assert!(
            matches!(s, Strategy::Hybrid { .. }),
            "Medium statement should resolve to Hybrid, got {:?}",
            s
        );
    }

    /// Verify that explicit hints override auto-selection.
    #[test]
    fn test_scheduler_explicit_hint_overrides_auto() {
        let scheduler = Scheduler::default();
        let hint = Strategy::Trm("custom-model".into());
        let s = scheduler.resolve_strategy("short question?", Some(&hint));
        match s {
            Strategy::Trm(model) => assert_eq!(model, "custom-model"),
            other => panic!("Expected Trm with custom-model, got {:?}", other),
        }
    }

    /// Verify recursion depth checking.
    #[test]
    fn test_scheduler_recursion_depth_limit() {
        let config = SchedulerConfig {
            max_recursion_depth: 5,
            ..Default::default()
        };
        let scheduler = Scheduler::new(config);

        assert!(scheduler.check_recursion_depth(0));
        assert!(scheduler.check_recursion_depth(4));
        assert!(!scheduler.check_recursion_depth(5));
        assert!(!scheduler.check_recursion_depth(100));
    }

    /// ADR-001 proposes a `Strategy::Swarm` variant for cross-zone
    /// scatter-gather. Since the scheduler does not yet define this variant,
    /// this test is gated behind a feature flag so it does not break the
    /// build. When the Swarm variant lands, remove the feature gate.
    #[cfg(feature = "integration-tests")]
    #[test]
    fn test_scheduler_with_swarm_strategy() {
        let scheduler = Scheduler::default();
        // Once Strategy::Swarm exists, test that:
        // 1. Providing Swarm as a hint returns Swarm (not auto-resolved).
        // 2. Auto does not select Swarm without a router.
        let swarm = Strategy::Swarm {
            scatter_zones: vec![],
            gather_strategy: crate::scheduler::GatherStrategy::First,
            timeout_ms: 10_000,
        };
        let s = scheduler.resolve_strategy("complex multi-step query", Some(&swarm));
        assert!(matches!(s, Strategy::Swarm { .. }));
    }

    // =======================================================================
    // Memory subsystem tests
    // =======================================================================

    /// Insert a segment via the embedding function and search for it,
    /// verifying a full roundtrip through the memory subsystem.
    #[tokio::test]
    async fn test_memory_insert_search_roundtrip() {
        let ctx = build_kernel_context();

        let content = "The kernel provides capability-secured syscall primitives";
        let embedding = text_to_embedding(content);
        assert_eq!(embedding.len(), EMBED_DIM);

        // Insert via dispatch
        let insert_result = dispatch(
            &Syscall::VecInsert {
                embedding: embedding.clone(),
                content: content.into(),
                metadata: make_meta("test-source"),
            },
            &ctx,
        )
        .await
        .unwrap();

        let segment_id = match insert_result {
            SyscallResult::VecInserted { segment_id } => segment_id,
            other => panic!("Expected VecInserted, got {:?}", other),
        };
        assert!(!segment_id.is_nil());

        // Search with the same embedding should return the inserted segment
        let search_result = dispatch(
            &Syscall::VecSearch {
                query: embedding.clone(),
                k: 5,
                filters: SearchFilters::default(),
            },
            &ctx,
        )
        .await
        .unwrap();

        match search_result {
            SyscallResult::VecSearchResults { results } => {
                assert!(!results.is_empty(), "Search should return at least one result");
                assert_eq!(results[0].segment_id, segment_id);
                assert!(
                    (results[0].score - 1.0).abs() < 1e-6,
                    "Self-search should have score ~1.0, got {}",
                    results[0].score
                );
                assert_eq!(results[0].content, content);
            }
            other => panic!("Expected VecSearchResults, got {:?}", other),
        }
    }

    /// Insert multiple segments and verify that search ranks them by
    /// cosine similarity correctly.
    #[tokio::test]
    async fn test_memory_search_ranking() {
        let ctx = build_kernel_context();

        let texts = [
            "vector search with cosine similarity",
            "graph algorithms and min-cut",
            "vector similarity and embedding search",
        ];

        for text in &texts {
            let emb = text_to_embedding(text);
            dispatch(
                &Syscall::VecInsert {
                    embedding: emb,
                    content: text.to_string(),
                    metadata: make_meta("ranking-test"),
                },
                &ctx,
            )
            .await
            .unwrap();
        }

        // Query about vectors should rank vector-related content higher
        let query_emb = text_to_embedding("vector search");
        let result = dispatch(
            &Syscall::VecSearch {
                query: query_emb,
                k: 3,
                filters: SearchFilters::default(),
            },
            &ctx,
        )
        .await
        .unwrap();

        match result {
            SyscallResult::VecSearchResults { results } => {
                assert_eq!(results.len(), 3);
                // The graph-only content should rank last
                assert!(
                    results[2].content.contains("graph"),
                    "Graph content should rank lowest for 'vector search' query, got: {}",
                    results[2].content,
                );
            }
            other => panic!("Expected VecSearchResults, got {:?}", other),
        }
    }

    /// Delete a segment and verify it no longer appears in search results.
    #[tokio::test]
    async fn test_memory_delete_removes_from_search() {
        let ctx = build_kernel_context();

        let emb = text_to_embedding("ephemeral data");
        let insert_result = dispatch(
            &Syscall::VecInsert {
                embedding: emb.clone(),
                content: "ephemeral data".into(),
                metadata: make_meta("delete-test"),
            },
            &ctx,
        )
        .await
        .unwrap();

        let segment_id = match insert_result {
            SyscallResult::VecInserted { segment_id } => segment_id,
            _ => unreachable!(),
        };

        // Delete
        let del_result = dispatch(&Syscall::VecDelete { segment_id }, &ctx)
            .await
            .unwrap();
        assert!(matches!(del_result, SyscallResult::VecDeleted { success: true }));

        // Search should return empty
        let search = dispatch(
            &Syscall::VecSearch {
                query: emb,
                k: 5,
                filters: SearchFilters::default(),
            },
            &ctx,
        )
        .await
        .unwrap();

        match search {
            SyscallResult::VecSearchResults { results } => {
                assert!(results.is_empty(), "Deleted segment should not appear in search");
            }
            _ => unreachable!(),
        }
    }

    // =======================================================================
    // Graph subsystem tests
    // =======================================================================

    /// Insert nodes and edges via the Graph API, then query via the
    /// GraphQuery syscall using Cypher syntax.
    #[tokio::test]
    async fn test_graph_insert_query() {
        let ctx = build_kernel_context();

        // Build graph directly (no syscall for node/edge insertion)
        {
            let mut graph = ctx.graph.lock().await;
            let agent = graph.insert_node("Agent");
            let task = graph.insert_node("Task");
            graph.insert_edge(agent, task, "ASSIGNED_TO", 1.0).unwrap();
        }

        // Query via dispatch
        let result = dispatch(
            &Syscall::GraphQuery {
                cypher: "MATCH (a:Agent)-[r:ASSIGNED_TO]->(t:Task) RETURN a,t".into(),
            },
            &ctx,
        )
        .await
        .unwrap();

        match result {
            SyscallResult::GraphQueryResult { rows } => {
                assert_eq!(rows.len(), 1, "Should find one Agent->Task edge");
                assert_eq!(rows[0]["a"]["type"], "Agent");
                assert_eq!(rows[0]["t"]["type"], "Task");
            }
            other => panic!("Expected GraphQueryResult, got {:?}", other),
        }
    }

    /// Build a graph and run Stoer-Wagner min-cut via the GraphCut syscall.
    #[tokio::test]
    async fn test_graph_cut_via_dispatch() {
        let ctx = build_kernel_context();

        {
            let mut graph = ctx.graph.lock().await;
            let a = graph.insert_node("N");
            let b = graph.insert_node("N");
            let c = graph.insert_node("N");
            let d = graph.insert_node("N");
            // Two clusters connected by a light edge
            graph.insert_edge(a, b, "HEAVY", 10.0).unwrap();
            graph.insert_edge(c, d, "HEAVY", 10.0).unwrap();
            graph.insert_edge(b, c, "LIGHT", 1.0).unwrap();
        }

        let result = dispatch(
            &Syscall::GraphCut {
                graph_id: Uuid::new_v4(), // not used by current impl
                algorithm: MinCutAlgorithm::StoerWagner,
            },
            &ctx,
        )
        .await
        .unwrap();

        match result {
            SyscallResult::GraphCutResult {
                cut_weight,
                partitions,
            } => {
                assert!(
                    (cut_weight - 1.0).abs() < 1e-9,
                    "Min-cut weight should be 1.0, got {}",
                    cut_weight
                );
                assert_eq!(partitions.len(), 2);
            }
            other => panic!("Expected GraphCutResult, got {:?}", other),
        }
    }

    /// Test graph diffusion via the GraphDiffuse syscall.
    #[tokio::test]
    async fn test_graph_diffuse_via_dispatch() {
        let ctx = build_kernel_context();

        let (_node_ids, _sorted_ids) = {
            let mut graph = ctx.graph.lock().await;
            let a = graph.insert_node("A");
            let b = graph.insert_node("B");
            graph.insert_edge(a, b, "E", 1.0).unwrap();
            let mut sorted = vec![a, b];
            sorted.sort();
            ((a, b), sorted)
        };

        // Set signal: 1.0 on first sorted node, 0.0 on second
        let signal = vec![1.0_f32, 0.0_f32];

        let result = dispatch(
            &Syscall::GraphDiffuse {
                graph_id: Uuid::new_v4(),
                signal,
                steps: 1,
            },
            &ctx,
        )
        .await
        .unwrap();

        match result {
            SyscallResult::GraphDiffused { output_signal } => {
                assert_eq!(output_signal.len(), 2);
                let sum: f64 = output_signal.iter().sum();
                assert!(
                    (sum - 1.0).abs() < 1e-6,
                    "Diffusion should preserve signal sum (~1.0), got {}",
                    sum
                );
            }
            other => panic!("Expected GraphDiffused, got {:?}", other),
        }
    }

    // =======================================================================
    // Capability token tests
    // =======================================================================

    /// Test hierarchical token derivation: parent creates child with subset
    /// permissions, child cannot exceed parent. (DDD-002 invariant 1)
    #[test]
    fn test_capability_token_derivation() {
        let mut mgr = CapabilityManager::new();

        // Root token with broad permissions
        let root_owner = Uuid::new_v4();
        let root_token = mgr.create_token(
            root_owner,
            vec![
                SyscallPermission::VecInsert,
                SyscallPermission::VecSearch,
                SyscallPermission::GraphQuery,
                SyscallPermission::ProcessFork,
                SyscallPermission::ProcessSend,
                SyscallPermission::ProcessRecv,
            ],
            "coordinator".into(),
            Duration::hours(2),
        );

        // Derive child with subset permissions (simulating ADR-005 Worker agent)
        let worker_owner = Uuid::new_v4();
        let worker_token = mgr
            .derive_child_token(
                &root_token.id,
                worker_owner,
                vec![
                    SyscallPermission::VecInsert,
                    SyscallPermission::VecSearch,
                    SyscallPermission::GraphQuery,
                    SyscallPermission::ProcessSend,
                    SyscallPermission::ProcessRecv,
                ],
                "worker".into(),
                Duration::hours(1),
            )
            .unwrap();

        // Worker token should allow VecSearch
        assert!(mgr
            .validate(&worker_token.id, &worker_owner, &SyscallPermission::VecSearch)
            .is_ok());

        // Worker token should NOT allow ProcessFork (not in child permissions)
        assert!(mgr
            .validate(&worker_token.id, &worker_owner, &SyscallPermission::ProcessFork)
            .is_err());

        // Attempting to derive a grandchild with permissions exceeding the worker
        // should fail (hierarchical narrowing rule)
        let grandchild_owner = Uuid::new_v4();
        let result = mgr.derive_child_token(
            &worker_token.id,
            grandchild_owner,
            vec![SyscallPermission::ProcessFork], // worker doesn't have this
            "grandchild".into(),
            Duration::hours(1),
        );
        assert!(result.is_err(), "Grandchild should not exceed parent permissions");
    }

    /// Test that a token with SyscallPermission::All can derive children
    /// with any subset of permissions.
    #[test]
    fn test_capability_all_permission_derives_any_subset() {
        let mut mgr = CapabilityManager::new();
        let root_owner = Uuid::new_v4();
        let root_token = mgr.create_token(
            root_owner,
            vec![SyscallPermission::All],
            "system".into(),
            Duration::hours(4),
        );

        let child_owner = Uuid::new_v4();
        let child = mgr
            .derive_child_token(
                &root_token.id,
                child_owner,
                vec![SyscallPermission::StateMutate, SyscallPermission::VecDelete],
                "special".into(),
                Duration::hours(1),
            )
            .unwrap();

        assert!(mgr
            .validate(&child.id, &child_owner, &SyscallPermission::StateMutate)
            .is_ok());
        assert!(mgr
            .validate(&child.id, &child_owner, &SyscallPermission::VecDelete)
            .is_ok());
        // Child should not have permissions outside its explicit set
        assert!(mgr
            .validate(&child.id, &child_owner, &SyscallPermission::VecInsert)
            .is_err());
    }

    /// Test that revoking a parent prevents deriving new children.
    #[test]
    fn test_capability_revoked_parent_blocks_derivation() {
        let mut mgr = CapabilityManager::new();
        let parent_owner = Uuid::new_v4();
        let parent = mgr.create_token(
            parent_owner,
            vec![SyscallPermission::All],
            "parent".into(),
            Duration::hours(2),
        );

        mgr.revoke(&parent.id);

        let child_owner = Uuid::new_v4();
        let result = mgr.derive_child_token(
            &parent.id,
            child_owner,
            vec![SyscallPermission::VecSearch],
            "child".into(),
            Duration::hours(1),
        );
        assert!(result.is_err());
    }

    // =======================================================================
    // Proof engine / witness chain tests
    // =======================================================================

    /// Test that the proof engine appends witnesses and maintains chain
    /// integrity across multiple state mutations. (DDD-002 invariant 3)
    #[test]
    fn test_proof_engine_witness_chain() {
        let mut engine = ProofEngine::new();

        let req = ProofRequest {
            confidence_threshold: 0.7,
            require_evidence: false,
        };

        // First mutation
        let proof1 = engine
            .validate("create-agent", "initial spawn", vec![], 0.9, &req)
            .unwrap();
        assert!(proof1.valid);
        assert_eq!(engine.chain.len(), 1);

        // Second mutation
        let proof2 = engine
            .validate(
                "assign-task",
                "coordinator dispatches work",
                vec!["task-spec-001".into()],
                0.85,
                &req,
            )
            .unwrap();
        assert!(proof2.valid);
        assert_eq!(engine.chain.len(), 2);

        // Third mutation (below threshold => invalid proof but still recorded)
        let proof3 = engine
            .validate("risky-mutation", "uncertain reasoning", vec![], 0.5, &req)
            .unwrap();
        assert!(!proof3.valid);
        assert_eq!(engine.chain.len(), 3);

        // Verify the entire chain is intact
        assert!(
            engine.chain.verify_integrity(),
            "Witness chain should maintain hash-linked integrity"
        );

        // Verify each witness can be retrieved
        let w1 = engine.chain.get(&proof1.witness_id).unwrap();
        assert!(w1.prev_hash.is_empty(), "Genesis witness should have empty prev_hash");

        let w2 = engine.chain.get(&proof2.witness_id).unwrap();
        assert_eq!(
            w2.prev_hash, w1.content_hash,
            "Second witness prev_hash should equal first witness content_hash"
        );

        let w3 = engine.chain.get(&proof3.witness_id).unwrap();
        assert_eq!(w3.prev_hash, w2.content_hash);
    }

    /// Test that evidence requirement is enforced.
    #[test]
    fn test_proof_engine_requires_evidence_when_configured() {
        let mut engine = ProofEngine::new();

        let req = ProofRequest {
            confidence_threshold: 0.5,
            require_evidence: true,
        };

        // Should fail without evidence
        let result = engine.validate("action", "reasoning", vec![], 0.9, &req);
        assert!(result.is_err());

        // Should succeed with evidence
        let result = engine.validate(
            "action",
            "reasoning",
            vec!["evidence-ref".into()],
            0.9,
            &req,
        );
        assert!(result.is_ok());
    }

    // =======================================================================
    // Process manager tests
    // =======================================================================

    /// Test process forking via dispatch, verifying that the ProcessFork
    /// syscall creates a new process with a properly scoped capability token.
    #[tokio::test]
    async fn test_process_fork_via_dispatch() {
        let ctx = build_kernel_context();

        let result = dispatch(
            &Syscall::ProcessFork {
                capabilities: vec![Capability {
                    name: "worker-caps".into(),
                    permissions: vec![
                        SyscallPermission::VecInsert,
                        SyscallPermission::VecSearch,
                        SyscallPermission::ProcessSend,
                        SyscallPermission::ProcessRecv,
                    ],
                }],
                memory_scope: "worker-scope".into(),
                task: "analyze-data".into(),
            },
            &ctx,
        )
        .await
        .unwrap();

        let child_id = match result {
            SyscallResult::ProcessForked { child_id } => child_id,
            other => panic!("Expected ProcessForked, got {:?}", other),
        };

        // Verify the process exists in the process manager
        let pm = ctx.process_manager.lock().await;
        let process = pm.get(&child_id).expect("Forked process should exist");
        assert_eq!(process.task, "analyze-data");
        assert_eq!(process.memory_scope, "worker-scope");
    }

    /// Test inter-process message passing via dispatch.
    #[tokio::test]
    async fn test_process_send_recv_via_dispatch() {
        let ctx = build_kernel_context();

        // Fork a process
        let result = dispatch(
            &Syscall::ProcessFork {
                capabilities: vec![Capability {
                    name: "msg-test".into(),
                    permissions: vec![SyscallPermission::ProcessSend, SyscallPermission::ProcessRecv],
                }],
                memory_scope: "msg-scope".into(),
                task: "message-test".into(),
            },
            &ctx,
        )
        .await
        .unwrap();

        let child_id = match result {
            SyscallResult::ProcessForked { child_id } => child_id,
            _ => unreachable!(),
        };

        // Send a message to the child
        let msg = KernelMessage::new(
            Uuid::new_v4(),
            child_id,
            serde_json::json!({"instruction": "process data"}),
        );

        let send_result = dispatch(
            &Syscall::ProcessSend {
                target: child_id,
                message: msg,
            },
            &ctx,
        )
        .await
        .unwrap();

        assert!(matches!(
            send_result,
            SyscallResult::MessageSent { delivered: true }
        ));

        // Receive the message (requires caller_pid)
        let ctx_with_pid = KernelContext {
            memory: ctx.memory.clone(),
            graph: ctx.graph.clone(),
            process_manager: ctx.process_manager.clone(),
            proof_engine: ctx.proof_engine.clone(),
            capability_manager: ctx.capability_manager.clone(),
            caller_pid: Some(child_id),
            event_bus: None,
        };

        let recv_result = dispatch(
            &Syscall::ProcessRecv {
                timeout: Some(std::time::Duration::from_secs(1)),
            },
            &ctx_with_pid,
        )
        .await
        .unwrap();

        match recv_result {
            SyscallResult::MessageReceived { message } => {
                let msg = message.expect("Should receive the sent message");
                assert_eq!(msg.payload["instruction"], "process data");
            }
            other => panic!("Expected MessageReceived, got {:?}", other),
        }
    }

    // =======================================================================
    // StateMutate syscall tests
    // =======================================================================

    /// Test the StateMutate syscall end-to-end through dispatch.
    #[tokio::test]
    async fn test_state_mutate_via_dispatch() {
        let ctx = build_kernel_context();

        let result = dispatch(
            &Syscall::StateMutate {
                action: "update-config".into(),
                params: serde_json::json!({
                    "confidence": 0.9,
                    "reasoning": "configuration change approved",
                    "evidence": ["approval-ticket-123"]
                }),
                proof: ProofRequest {
                    confidence_threshold: 0.8,
                    require_evidence: true,
                },
            },
            &ctx,
        )
        .await
        .unwrap();

        match result {
            SyscallResult::StateMutated {
                witness_id,
                success,
            } => {
                assert!(success, "Mutation should succeed with confidence above threshold");
                assert!(!witness_id.is_nil());

                // Verify the witness was recorded
                let engine = ctx.proof_engine.lock().await;
                let witness = engine.chain.get(&witness_id).unwrap();
                assert!(!witness.content_hash.is_empty());
            }
            other => panic!("Expected StateMutated, got {:?}", other),
        }
    }

    // =======================================================================
    // Attention and Halt syscall tests
    // =======================================================================

    #[tokio::test]
    async fn test_attention_select_mechanisms() {
        let ctx = build_kernel_context();

        // Sparse attention
        let result = dispatch(
            &Syscall::AttentionSelect {
                operation_type: "sparse".into(),
                context_size: 2048,
            },
            &ctx,
        )
        .await
        .unwrap();

        match result {
            SyscallResult::AttentionSelected {
                selected_indices,
                mechanism,
            } => {
                assert_eq!(mechanism, "sparse_topk");
                assert!(!selected_indices.is_empty());
                assert!(selected_indices.len() <= 1024);
            }
            other => panic!("Expected AttentionSelected, got {:?}", other),
        }

        // Sliding window attention
        let result = dispatch(
            &Syscall::AttentionSelect {
                operation_type: "sliding_window".into(),
                context_size: 2048,
            },
            &ctx,
        )
        .await
        .unwrap();

        match result {
            SyscallResult::AttentionSelected {
                selected_indices,
                mechanism,
            } => {
                assert_eq!(mechanism, "sliding_window");
                // Sliding window selects the last N positions
                assert!(selected_indices.len() <= 512);
                if let Some(&last) = selected_indices.last() {
                    assert_eq!(last, 2047, "Sliding window should include the last position");
                }
            }
            other => panic!("Expected AttentionSelected, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_halt_check_decisions() {
        let ctx = build_kernel_context();

        // Should halt at high confidence
        let result = dispatch(
            &Syscall::HaltCheck {
                confidence: 0.98,
                iteration: 3,
                max_iterations: 10,
            },
            &ctx,
        )
        .await
        .unwrap();

        match result {
            SyscallResult::HaltDecision {
                should_halt,
                reason,
            } => {
                assert!(should_halt);
                assert!(reason.contains("confidence"));
            }
            other => panic!("Expected HaltDecision, got {:?}", other),
        }

        // Should halt at max iterations
        let result = dispatch(
            &Syscall::HaltCheck {
                confidence: 0.5,
                iteration: 10,
                max_iterations: 10,
            },
            &ctx,
        )
        .await
        .unwrap();

        match result {
            SyscallResult::HaltDecision {
                should_halt,
                reason,
            } => {
                assert!(should_halt);
                assert!(reason.contains("max iterations"));
            }
            _ => unreachable!(),
        }

        // Should NOT halt when both conditions are unmet
        let result = dispatch(
            &Syscall::HaltCheck {
                confidence: 0.7,
                iteration: 3,
                max_iterations: 10,
            },
            &ctx,
        )
        .await
        .unwrap();

        match result {
            SyscallResult::HaltDecision {
                should_halt,
                reason,
            } => {
                assert!(!should_halt);
                assert!(reason.contains("continue"));
            }
            _ => unreachable!(),
        }
    }

    // =======================================================================
    // Cross-subsystem integration tests
    // =======================================================================

    /// End-to-end: fork a process, insert vectors into memory, query graph,
    /// and validate a state mutation -- all through the dispatch function.
    /// Verifies that all subsystems cooperate through KernelContext.
    #[tokio::test]
    async fn test_cross_subsystem_memory_graph_process() {
        let ctx = build_kernel_context();

        // 1. Fork a worker process
        let fork_result = dispatch(
            &Syscall::ProcessFork {
                capabilities: vec![Capability {
                    name: "full".into(),
                    permissions: vec![SyscallPermission::All],
                }],
                memory_scope: "integration".into(),
                task: "cross-subsystem-test".into(),
            },
            &ctx,
        )
        .await
        .unwrap();

        let worker_pid = match fork_result {
            SyscallResult::ProcessForked { child_id } => child_id,
            _ => unreachable!(),
        };

        // 2. Insert knowledge into memory
        let emb = text_to_embedding("agent coordination protocol");
        dispatch(
            &Syscall::VecInsert {
                embedding: emb,
                content: "agent coordination protocol".into(),
                metadata: make_meta("integration"),
            },
            &ctx,
        )
        .await
        .unwrap();

        // 3. Build a graph representing agent relationships
        {
            let mut graph = ctx.graph.lock().await;
            let coordinator = graph.insert_node("Coordinator");
            let worker = graph.insert_node("Worker");
            graph
                .insert_edge(coordinator, worker, "MANAGES", 1.0)
                .unwrap();
        }

        // 4. Query the graph
        let query_result = dispatch(
            &Syscall::GraphQuery {
                cypher: "MATCH (c:Coordinator)-[r:MANAGES]->(w:Worker) RETURN c,w".into(),
            },
            &ctx,
        )
        .await
        .unwrap();

        match &query_result {
            SyscallResult::GraphQueryResult { rows } => {
                assert_eq!(rows.len(), 1, "Should find the Coordinator->Worker edge");
            }
            other => panic!("Expected GraphQueryResult, got {:?}", other),
        }

        // 5. Search memory for related knowledge
        let search_emb = text_to_embedding("coordination");
        let search_result = dispatch(
            &Syscall::VecSearch {
                query: search_emb,
                k: 1,
                filters: SearchFilters::default(),
            },
            &ctx,
        )
        .await
        .unwrap();

        match search_result {
            SyscallResult::VecSearchResults { results } => {
                assert_eq!(results.len(), 1);
                assert!(results[0].content.contains("coordination"));
            }
            _ => unreachable!(),
        }

        // 6. Record a state mutation with proof
        let mutate_result = dispatch(
            &Syscall::StateMutate {
                action: "task-complete".into(),
                params: serde_json::json!({
                    "confidence": 0.95,
                    "reasoning": "worker completed task successfully"
                }),
                proof: ProofRequest::default(),
            },
            &ctx,
        )
        .await
        .unwrap();

        match mutate_result {
            SyscallResult::StateMutated {
                success,
                witness_id,
            } => {
                assert!(success);
                let engine = ctx.proof_engine.lock().await;
                assert!(engine.chain.verify_integrity());
                assert!(engine.chain.get(&witness_id).is_some());
            }
            _ => unreachable!(),
        }

        // 7. Send a completion message to the worker
        let msg = KernelMessage::new(
            Uuid::new_v4(),
            worker_pid,
            serde_json::json!({"status": "completed"}),
        );
        let send_result = dispatch(
            &Syscall::ProcessSend {
                target: worker_pid,
                message: msg,
            },
            &ctx,
        )
        .await
        .unwrap();

        assert!(matches!(
            send_result,
            SyscallResult::MessageSent { delivered: true }
        ));
    }

    /// Test that the text_to_embedding function produces reproducible
    /// embeddings (critical for deterministic search behavior).
    #[test]
    fn test_embedding_reproducibility() {
        let text = "capability-secured syscall primitives";
        let e1 = text_to_embedding(text);
        let e2 = text_to_embedding(text);
        assert_eq!(e1, e2, "Same text should produce identical embeddings");

        // Different text should produce different embeddings
        let e3 = text_to_embedding("graph algorithms");
        assert_ne!(e1, e3);

        // Embeddings should be unit-normalized
        let norm: f32 = e1.iter().map(|v| v * v).sum::<f32>().sqrt();
        assert!(
            (norm - 1.0).abs() < 1e-5,
            "Embedding should be L2-normalized, got norm {}",
            norm
        );
    }

    /// Test cosine similarity properties used by the memory subsystem.
    #[test]
    fn test_cosine_similarity_properties() {
        let a = text_to_embedding("hello world");
        let b = text_to_embedding("hello world");

        // Self-similarity
        let sim = cosine_similarity(&a, &b);
        assert!(
            (sim - 1.0).abs() < 1e-6,
            "Identical embeddings should have similarity 1.0, got {}",
            sim
        );

        // Similar texts should have high similarity
        let c = text_to_embedding("hello earth");
        let sim_ac = cosine_similarity(&a, &c);
        assert!(sim_ac > 0.5, "Similar texts should have sim > 0.5, got {}", sim_ac);

        // Very different texts should have lower similarity
        let d = text_to_embedding("zzzzzzzzzzzzzzzzzz");
        let sim_ad = cosine_similarity(&a, &d);
        assert!(
            sim_ad < sim_ac,
            "Different text should have lower sim ({}) than similar text ({})",
            sim_ad,
            sim_ac
        );
    }

    // =======================================================================
    // Feature-gated tests for router module (ADR-003)
    // =======================================================================

    /// These tests will exercise the TinyDancerRouter once the router module
    /// is implemented. Gated behind `integration-tests` feature to avoid
    /// breaking the build while the module is being developed.
    #[cfg(feature = "integration-tests")]
    mod router_tests {
        use super::*;

        #[test]
        fn test_router_feature_extraction() {
            // Will test TinyDancerRouter::extract_features() once router.rs exists
            todo!("Implement when crates/rlmx-kernel/src/router.rs is created");
        }

        #[test]
        fn test_router_inference_returns_valid_strategy() {
            // Will test that router inference produces one of the 5 strategies
            todo!("Implement when crates/rlmx-kernel/src/router.rs is created");
        }

        #[test]
        fn test_scheduler_with_neural_router() {
            // Will test Scheduler::resolve_strategy with a TinyDancerRouter
            todo!("Implement when router is wired into scheduler");
        }
    }

    /// Feature-gated test for Strategy::Swarm process spawning with agent_type.
    /// ADR-005 defines 12 agent types; the kernel's ProcessFork does not yet
    /// include an agent_type field. This test will validate it when added.
    #[cfg(feature = "integration-tests")]
    #[tokio::test]
    async fn test_process_manager_agent_spawn() {
        // When ProcessFork gains an agent_type field (per ADR-005),
        // verify that:
        // 1. The agent_type is stored on the Process struct
        // 2. The capability token permissions match the ADR-005 permission matrix
        // 3. A Worker agent cannot perform StateMutate
        todo!("Implement when agent_type field is added to ProcessFork");
    }
}

use std::cmp::Reverse;
use std::collections::HashSet;

use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, Order, QueryFilter, QueryOrder,
    Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use entity::{object, test_case, test_execution};

use crate::error::CoreError;

const VALID_TEST_TYPES: &[&str] = &["manual", "automated", "exploratory"];
const VALID_PRIORITIES: &[&str] = &["critical", "high", "medium", "low"];
const VALID_CASE_STATUSES: &[&str] = &["draft", "ready", "deprecated"];
const VALID_EXEC_STATUSES: &[&str] = &["passed", "failed", "blocked", "skipped", "not_run"];

// --- DTOs ---

#[derive(Debug, Deserialize)]
pub struct CreateTestCaseInput {
    pub name: String,
    pub description: Option<String>,
    pub preconditions: Option<String>,
    pub expected_result: Option<String>,
    pub test_type: Option<String>,
    pub priority: Option<String>,
    pub status: Option<String>,
    pub requirement_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTestCaseInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub preconditions: Option<String>,
    pub expected_result: Option<String>,
    pub test_type: Option<String>,
    pub priority: Option<String>,
    pub status: Option<String>,
    pub requirement_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTestExecutionInput {
    pub status: Option<String>,
    pub executor: Option<String>,
    pub executed_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub duration_ms: Option<i64>,
    pub evidence: Option<String>,
    pub environment: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTestExecutionInput {
    pub status: Option<String>,
    pub executor: Option<String>,
    pub executed_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub duration_ms: Option<i64>,
    pub evidence: Option<String>,
    pub environment: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TestStatusCounts {
    pub passed: u64,
    pub failed: u64,
    pub blocked: u64,
    pub skipped: u64,
    pub not_run: u64,
}

#[derive(Debug, Serialize)]
pub struct TestCoverageResponse {
    pub total_requirements: u64,
    pub requirements_with_tests: u64,
    pub requirements_with_passing_tests: u64,
    pub test_coverage_pct: f64,
    pub pass_coverage_pct: f64,
    pub total_test_cases: u64,
    pub by_status: TestStatusCounts,
}

#[derive(Debug, Serialize)]
pub struct TestCaseStatusCounts {
    pub draft: u64,
    pub ready: u64,
    pub deprecated: u64,
}

#[derive(Debug, Serialize)]
pub struct TestPriorityCounts {
    pub critical: u64,
    pub high: u64,
    pub medium: u64,
    pub low: u64,
}

#[derive(Debug, Serialize)]
pub struct TestDashboardSummary {
    pub total_test_cases: u64,
    pub by_test_status: TestCaseStatusCounts,
    pub by_priority: TestPriorityCounts,
    pub recent_executions: Vec<test_execution::Model>,
    pub coverage: TestCoverageResponse,
}

// --- Service ---

pub struct TestService;

impl TestService {
    // --- Test Case CRUD ---

    pub async fn create_test_case(
        db: &impl ConnectionTrait,
        module_id: Uuid,
        input: CreateTestCaseInput,
    ) -> Result<test_case::Model, CoreError> {
        let test_type = input.test_type.as_deref().unwrap_or("manual");
        if !VALID_TEST_TYPES.contains(&test_type) {
            return Err(CoreError::BadRequest(format!(
                "invalid test_type '{test_type}', must be one of: {}",
                VALID_TEST_TYPES.join(", ")
            )));
        }

        let priority = input.priority.as_deref().unwrap_or("medium");
        if !VALID_PRIORITIES.contains(&priority) {
            return Err(CoreError::BadRequest(format!(
                "invalid priority '{priority}', must be one of: {}",
                VALID_PRIORITIES.join(", ")
            )));
        }

        let status = input.status.as_deref().unwrap_or("draft");
        if !VALID_CASE_STATUSES.contains(&status) {
            return Err(CoreError::BadRequest(format!(
                "invalid status '{status}', must be one of: {}",
                VALID_CASE_STATUSES.join(", ")
            )));
        }

        let now = chrono::Utc::now().fixed_offset();
        let id = Uuid::now_v7();

        let req_ids = serde_json::to_value(input.requirement_ids.unwrap_or_default())
            .map_err(|e| CoreError::Internal(format!("json error: {e}")))?;

        let model = test_case::ActiveModel {
            id: Set(id),
            module_id: Set(module_id),
            name: Set(input.name),
            description: Set(input.description),
            preconditions: Set(input.preconditions),
            expected_result: Set(input.expected_result),
            test_type: Set(test_type.to_owned()),
            priority: Set(priority.to_owned()),
            status: Set(status.to_owned()),
            requirement_ids: Set(req_ids),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = model.insert(db).await?;
        Ok(result)
    }

    pub async fn list_test_cases(
        db: &impl ConnectionTrait,
        module_id: Uuid,
    ) -> Result<Vec<test_case::Model>, CoreError> {
        let items = test_case::Entity::find()
            .filter(test_case::Column::ModuleId.eq(module_id))
            .order_by(test_case::Column::CreatedAt, Order::Desc)
            .all(db)
            .await?;
        Ok(items)
    }

    pub async fn get_test_case(
        db: &impl ConnectionTrait,
        id: Uuid,
    ) -> Result<test_case::Model, CoreError> {
        test_case::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("test case {id} not found")))
    }

    pub async fn update_test_case(
        db: &impl ConnectionTrait,
        id: Uuid,
        input: UpdateTestCaseInput,
    ) -> Result<test_case::Model, CoreError> {
        let existing = test_case::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("test case {id} not found")))?;

        let mut active: test_case::ActiveModel = existing.into();

        if let Some(name) = input.name {
            active.name = Set(name);
        }
        if let Some(description) = input.description {
            active.description = Set(Some(description));
        }
        if let Some(preconditions) = input.preconditions {
            active.preconditions = Set(Some(preconditions));
        }
        if let Some(expected_result) = input.expected_result {
            active.expected_result = Set(Some(expected_result));
        }
        if let Some(test_type) = input.test_type {
            if !VALID_TEST_TYPES.contains(&test_type.as_str()) {
                return Err(CoreError::BadRequest(format!(
                    "invalid test_type '{test_type}'"
                )));
            }
            active.test_type = Set(test_type);
        }
        if let Some(priority) = input.priority {
            if !VALID_PRIORITIES.contains(&priority.as_str()) {
                return Err(CoreError::BadRequest(format!(
                    "invalid priority '{priority}'"
                )));
            }
            active.priority = Set(priority);
        }
        if let Some(status) = input.status {
            if !VALID_CASE_STATUSES.contains(&status.as_str()) {
                return Err(CoreError::BadRequest(format!(
                    "invalid status '{status}'"
                )));
            }
            active.status = Set(status);
        }
        if let Some(requirement_ids) = input.requirement_ids {
            let req_ids = serde_json::to_value(requirement_ids)
                .map_err(|e| CoreError::Internal(format!("json error: {e}")))?;
            active.requirement_ids = Set(req_ids);
        }

        active.updated_at = Set(chrono::Utc::now().fixed_offset());

        let result = active.update(db).await?;
        Ok(result)
    }

    pub async fn delete_test_case(db: &impl ConnectionTrait, id: Uuid) -> Result<(), CoreError> {
        let result = test_case::Entity::delete_by_id(id).exec(db).await?;
        if result.rows_affected == 0 {
            return Err(CoreError::NotFound(format!("test case {id} not found")));
        }
        Ok(())
    }

    // --- Test Execution CRUD ---

    pub async fn create_test_execution(
        db: &impl ConnectionTrait,
        test_case_id: Uuid,
        input: CreateTestExecutionInput,
    ) -> Result<test_execution::Model, CoreError> {
        // Verify the test case exists
        let _ = Self::get_test_case(db, test_case_id).await?;

        let status = input.status.as_deref().unwrap_or("not_run");
        if !VALID_EXEC_STATUSES.contains(&status) {
            return Err(CoreError::BadRequest(format!(
                "invalid execution status '{status}', must be one of: {}",
                VALID_EXEC_STATUSES.join(", ")
            )));
        }

        let now = chrono::Utc::now().fixed_offset();
        let id = Uuid::now_v7();

        let model = test_execution::ActiveModel {
            id: Set(id),
            test_case_id: Set(test_case_id),
            status: Set(status.to_owned()),
            executor: Set(input.executor),
            executed_at: Set(input.executed_at),
            duration_ms: Set(input.duration_ms),
            evidence: Set(input.evidence),
            environment: Set(input.environment),
            created_at: Set(now),
        };

        let result = model.insert(db).await?;
        Ok(result)
    }

    pub async fn list_test_executions(
        db: &impl ConnectionTrait,
        test_case_id: Uuid,
    ) -> Result<Vec<test_execution::Model>, CoreError> {
        let items = test_execution::Entity::find()
            .filter(test_execution::Column::TestCaseId.eq(test_case_id))
            .order_by(test_execution::Column::CreatedAt, Order::Desc)
            .all(db)
            .await?;
        Ok(items)
    }

    pub async fn get_test_execution(
        db: &impl ConnectionTrait,
        id: Uuid,
    ) -> Result<test_execution::Model, CoreError> {
        test_execution::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("test execution {id} not found")))
    }

    pub async fn update_test_execution(
        db: &impl ConnectionTrait,
        id: Uuid,
        input: UpdateTestExecutionInput,
    ) -> Result<test_execution::Model, CoreError> {
        let existing = test_execution::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("test execution {id} not found")))?;

        let mut active: test_execution::ActiveModel = existing.into();

        if let Some(status) = input.status {
            if !VALID_EXEC_STATUSES.contains(&status.as_str()) {
                return Err(CoreError::BadRequest(format!(
                    "invalid execution status '{status}'"
                )));
            }
            active.status = Set(status);
        }
        if let Some(executor) = input.executor {
            active.executor = Set(Some(executor));
        }
        if let Some(executed_at) = input.executed_at {
            active.executed_at = Set(Some(executed_at));
        }
        if let Some(duration_ms) = input.duration_ms {
            active.duration_ms = Set(Some(duration_ms));
        }
        if let Some(evidence) = input.evidence {
            active.evidence = Set(Some(evidence));
        }
        if let Some(environment) = input.environment {
            active.environment = Set(Some(environment));
        }

        let result = active.update(db).await?;
        Ok(result)
    }

    pub async fn delete_test_execution(
        db: &impl ConnectionTrait,
        id: Uuid,
    ) -> Result<(), CoreError> {
        let result = test_execution::Entity::delete_by_id(id).exec(db).await?;
        if result.rows_affected == 0 {
            return Err(CoreError::NotFound(format!(
                "test execution {id} not found"
            )));
        }
        Ok(())
    }

    // --- Coverage ---

    pub async fn coverage(
        db: &impl ConnectionTrait,
        module_id: Uuid,
    ) -> Result<TestCoverageResponse, CoreError> {
        // Count non-deleted objects in module
        let all_objects = object::Entity::find()
            .filter(object::Column::ModuleId.eq(module_id))
            .filter(object::Column::DeletedAt.is_null())
            .all(db)
            .await?;
        let total_requirements = all_objects.len() as u64;

        // Get all test cases for this module
        let test_cases = test_case::Entity::find()
            .filter(test_case::Column::ModuleId.eq(module_id))
            .all(db)
            .await?;
        let total_test_cases = test_cases.len() as u64;

        // Collect all unique requirement_ids across test cases
        let mut requirements_with_tests_set: HashSet<String> = HashSet::new();
        let mut requirements_with_passing_set: HashSet<String> = HashSet::new();

        let mut status_counts = TestStatusCounts {
            passed: 0,
            failed: 0,
            blocked: 0,
            skipped: 0,
            not_run: 0,
        };

        for tc in &test_cases {
            let req_ids: Vec<String> =
                serde_json::from_value(tc.requirement_ids.clone()).unwrap_or_default();
            for rid in &req_ids {
                let _ = requirements_with_tests_set.insert(rid.clone());
            }

            // Get latest execution for this test case
            let latest_exec = test_execution::Entity::find()
                .filter(test_execution::Column::TestCaseId.eq(tc.id))
                .order_by(test_execution::Column::CreatedAt, Order::Desc)
                .one(db)
                .await?;

            let exec_status = latest_exec
                .as_ref()
                .map_or("not_run", |e| e.status.as_str());

            match exec_status {
                "passed" => {
                    status_counts.passed += 1;
                    for rid in &req_ids {
                        let _ = requirements_with_passing_set.insert(rid.clone());
                    }
                }
                "failed" => status_counts.failed += 1,
                "blocked" => status_counts.blocked += 1,
                "skipped" => status_counts.skipped += 1,
                _ => status_counts.not_run += 1,
            }
        }

        let requirements_with_tests = requirements_with_tests_set.len() as u64;
        let requirements_with_passing_tests = requirements_with_passing_set.len() as u64;

        #[allow(clippy::cast_precision_loss)]
        let test_coverage_pct = if total_requirements > 0 {
            (requirements_with_tests as f64 / total_requirements as f64) * 100.0
        } else {
            0.0
        };

        #[allow(clippy::cast_precision_loss)]
        let pass_coverage_pct = if total_requirements > 0 {
            (requirements_with_passing_tests as f64 / total_requirements as f64) * 100.0
        } else {
            0.0
        };

        Ok(TestCoverageResponse {
            total_requirements,
            requirements_with_tests,
            requirements_with_passing_tests,
            test_coverage_pct,
            pass_coverage_pct,
            total_test_cases,
            by_status: status_counts,
        })
    }

    // --- Dashboard ---

    pub async fn dashboard(
        db: &impl ConnectionTrait,
        module_id: Uuid,
    ) -> Result<TestDashboardSummary, CoreError> {
        let test_cases = test_case::Entity::find()
            .filter(test_case::Column::ModuleId.eq(module_id))
            .all(db)
            .await?;

        let total_test_cases = test_cases.len() as u64;

        let mut by_test_status = TestCaseStatusCounts {
            draft: 0,
            ready: 0,
            deprecated: 0,
        };

        let mut by_priority = TestPriorityCounts {
            critical: 0,
            high: 0,
            medium: 0,
            low: 0,
        };

        for tc in &test_cases {
            match tc.status.as_str() {
                "draft" => by_test_status.draft += 1,
                "ready" => by_test_status.ready += 1,
                "deprecated" => by_test_status.deprecated += 1,
                _ => {}
            }
            match tc.priority.as_str() {
                "critical" => by_priority.critical += 1,
                "high" => by_priority.high += 1,
                "medium" => by_priority.medium += 1,
                "low" => by_priority.low += 1,
                _ => {}
            }
        }

        // Get recent executions (last 20)
        // Collect all test case IDs for this module, then query executions
        let tc_ids: Vec<Uuid> = test_cases.iter().map(|tc| tc.id).collect();
        let recent_executions = if tc_ids.is_empty() {
            Vec::new()
        } else {
            // Build a map of tc_ids to get all executions in the module
            let mut all_execs: Vec<test_execution::Model> = Vec::new();
            for tc_id in &tc_ids {
                let execs = test_execution::Entity::find()
                    .filter(test_execution::Column::TestCaseId.eq(*tc_id))
                    .order_by(test_execution::Column::CreatedAt, Order::Desc)
                    .all(db)
                    .await?;
                all_execs.extend(execs);
            }
            all_execs.sort_by_key(|e| Reverse(e.created_at));
            all_execs.truncate(20);
            all_execs
        };

        let coverage = Self::coverage(db, module_id).await?;

        Ok(TestDashboardSummary {
            total_test_cases,
            by_test_status,
            by_priority,
            recent_executions,
            coverage,
        })
    }
}

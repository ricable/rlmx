//! Budget optimization via LP solver integration.
//!
//! When the `ruvnet-phase3` feature is enabled, delegates to `ruvector-solver`
//! for hardware-accelerated linear programming. Otherwise, provides a greedy
//! heuristic fallback that works without external dependencies.

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum SolverError {
    #[error("infeasible: total minimum allocations ({0:.2}) exceed budget ({1:.2})")]
    Infeasible(f64, f64),
    #[error("no allocations provided")]
    EmptyAllocations,
    #[error("solver failed: {0}")]
    SolverFailed(String),
}

pub type Result<T> = std::result::Result<T, SolverError>;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A single budget constraint for one life domain or category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConstraint {
    /// Human-readable label (e.g. "groceries", "entertainment").
    pub label: String,
    /// Priority weight (higher = more important). Used by the greedy fallback.
    pub priority: f64,
    /// Minimum allocation (hard floor).
    pub min_allocation: f64,
    /// Maximum allocation (hard ceiling).
    pub max_allocation: f64,
    /// Expected return per unit allocated (utility coefficient).
    pub utility_per_unit: f64,
}

/// Result of a single allocation decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAllocation {
    pub label: String,
    pub allocated: f64,
    pub utility: f64,
}

/// Full optimization result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub allocations: Vec<BudgetAllocation>,
    pub total_utility: f64,
    pub total_allocated: f64,
    pub budget_remaining: f64,
    /// Whether the result was computed by the LP solver or the greedy fallback.
    pub method: String,
}

// ---------------------------------------------------------------------------
// Budget Optimizer
// ---------------------------------------------------------------------------

/// Allocates a fixed budget across life domains to maximize total utility,
/// respecting per-domain min/max constraints.
#[derive(Debug, Clone)]
pub struct BudgetOptimizer {
    pub constraints: Vec<BudgetConstraint>,
    pub total_budget: f64,
}

impl BudgetOptimizer {
    /// Create a new optimizer with the given budget and constraints.
    pub fn new(total_budget: f64, constraints: Vec<BudgetConstraint>) -> Self {
        Self {
            constraints,
            total_budget,
        }
    }

    /// Run the optimization and return the allocation result.
    ///
    /// When `ruvnet-phase3` is enabled, delegates to ruvector-solver's LP
    /// engine. Otherwise falls back to a greedy priority-weighted heuristic.
    pub fn optimize(&self) -> Result<OptimizationResult> {
        if self.constraints.is_empty() {
            return Err(SolverError::EmptyAllocations);
        }

        let min_total: f64 = self.constraints.iter().map(|c| c.min_allocation).sum();
        if min_total > self.total_budget {
            return Err(SolverError::Infeasible(min_total, self.total_budget));
        }

        #[cfg(feature = "ruvnet-phase3")]
        {
            self.optimize_lp()
        }

        #[cfg(not(feature = "ruvnet-phase3"))]
        {
            self.optimize_greedy()
        }
    }

    /// Greedy heuristic: assign minimums, then distribute remaining budget
    /// proportional to priority * utility_per_unit, capped at max.
    fn optimize_greedy(&self) -> Result<OptimizationResult> {
        let mut allocations: Vec<f64> = self.constraints.iter().map(|c| c.min_allocation).collect();
        let mut remaining = self.total_budget - allocations.iter().sum::<f64>();

        // Score = priority * utility_per_unit. Distribute in rounds.
        let scores: Vec<f64> = self
            .constraints
            .iter()
            .map(|c| c.priority * c.utility_per_unit)
            .collect();
        let total_score: f64 = scores.iter().sum();

        if total_score > 0.0 {
            // Sort indices by score descending for greedy assignment.
            let mut indices: Vec<usize> = (0..self.constraints.len()).collect();
            indices.sort_by(|&a, &b| {
                scores[b]
                    .partial_cmp(&scores[a])
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            for &i in &indices {
                if remaining <= 0.0 {
                    break;
                }
                let headroom = self.constraints[i].max_allocation - allocations[i];
                let share = (scores[i] / total_score) * remaining;
                let grant = share.min(headroom).max(0.0);
                allocations[i] += grant;
                remaining -= grant;
            }
        }

        let result_allocs: Vec<BudgetAllocation> = self
            .constraints
            .iter()
            .zip(allocations.iter())
            .map(|(c, &a)| BudgetAllocation {
                label: c.label.clone(),
                allocated: a,
                utility: a * c.utility_per_unit,
            })
            .collect();

        let total_utility: f64 = result_allocs.iter().map(|a| a.utility).sum();
        let total_allocated: f64 = result_allocs.iter().map(|a| a.allocated).sum();

        Ok(OptimizationResult {
            allocations: result_allocs,
            total_utility,
            total_allocated,
            budget_remaining: self.total_budget - total_allocated,
            method: "greedy".to_string(),
        })
    }

    /// LP-based optimization using ruvector-solver.
    #[cfg(feature = "ruvnet-phase3")]
    fn optimize_lp(&self) -> Result<OptimizationResult> {
        use ruvector_solver::LpSolver;

        // Build the LP problem: maximize sum(utility_per_unit_i * x_i)
        // subject to: sum(x_i) <= total_budget, min_i <= x_i <= max_i
        let solver = LpSolver::new();
        let n = self.constraints.len();

        let objective: Vec<f64> = self
            .constraints
            .iter()
            .map(|c| c.utility_per_unit)
            .collect();
        let lower: Vec<f64> = self.constraints.iter().map(|c| c.min_allocation).collect();
        let upper: Vec<f64> = self.constraints.iter().map(|c| c.max_allocation).collect();

        let solution = solver
            .maximize(&objective, &lower, &upper, self.total_budget)
            .map_err(|e| SolverError::SolverFailed(format!("{e}")))?;

        let result_allocs: Vec<BudgetAllocation> = self
            .constraints
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let alloc = solution.values.get(i).copied().unwrap_or(c.min_allocation);
                BudgetAllocation {
                    label: c.label.clone(),
                    allocated: alloc,
                    utility: alloc * c.utility_per_unit,
                }
            })
            .collect();

        let total_utility: f64 = result_allocs.iter().map(|a| a.utility).sum();
        let total_allocated: f64 = result_allocs.iter().map(|a| a.allocated).sum();

        Ok(OptimizationResult {
            allocations: result_allocs,
            total_utility,
            total_allocated,
            budget_remaining: self.total_budget - total_allocated,
            method: "lp-ruvector".to_string(),
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_constraints() -> Vec<BudgetConstraint> {
        vec![
            BudgetConstraint {
                label: "groceries".into(),
                priority: 3.0,
                min_allocation: 200.0,
                max_allocation: 800.0,
                utility_per_unit: 1.5,
            },
            BudgetConstraint {
                label: "entertainment".into(),
                priority: 1.0,
                min_allocation: 50.0,
                max_allocation: 400.0,
                utility_per_unit: 1.0,
            },
            BudgetConstraint {
                label: "savings".into(),
                priority: 2.0,
                min_allocation: 100.0,
                max_allocation: 1000.0,
                utility_per_unit: 2.0,
            },
        ]
    }

    #[test]
    fn test_greedy_optimizer_respects_minimums() {
        let optimizer = BudgetOptimizer::new(1000.0, sample_constraints());
        let result = optimizer.optimize().unwrap();

        assert_eq!(result.method, "greedy");
        for alloc in &result.allocations {
            let constraint = optimizer
                .constraints
                .iter()
                .find(|c| c.label == alloc.label)
                .unwrap();
            assert!(
                alloc.allocated >= constraint.min_allocation,
                "{}: allocated {} < min {}",
                alloc.label,
                alloc.allocated,
                constraint.min_allocation
            );
            assert!(
                alloc.allocated <= constraint.max_allocation + 0.01,
                "{}: allocated {} > max {}",
                alloc.label,
                alloc.allocated,
                constraint.max_allocation
            );
        }
    }

    #[test]
    fn test_optimizer_total_within_budget() {
        let optimizer = BudgetOptimizer::new(1000.0, sample_constraints());
        let result = optimizer.optimize().unwrap();

        assert!(
            result.total_allocated <= 1000.0 + 0.01,
            "total allocated {} exceeds budget 1000",
            result.total_allocated
        );
        assert!(result.budget_remaining >= -0.01);
    }

    #[test]
    fn test_infeasible_budget() {
        let constraints = vec![
            BudgetConstraint {
                label: "a".into(),
                priority: 1.0,
                min_allocation: 600.0,
                max_allocation: 1000.0,
                utility_per_unit: 1.0,
            },
            BudgetConstraint {
                label: "b".into(),
                priority: 1.0,
                min_allocation: 500.0,
                max_allocation: 1000.0,
                utility_per_unit: 1.0,
            },
        ];
        let optimizer = BudgetOptimizer::new(1000.0, constraints);
        let err = optimizer.optimize().unwrap_err();
        assert!(matches!(err, SolverError::Infeasible(_, _)));
    }

    #[test]
    fn test_empty_constraints() {
        let optimizer = BudgetOptimizer::new(1000.0, vec![]);
        let err = optimizer.optimize().unwrap_err();
        assert!(matches!(err, SolverError::EmptyAllocations));
    }

    #[test]
    fn test_higher_priority_gets_more() {
        let constraints = vec![
            BudgetConstraint {
                label: "high".into(),
                priority: 10.0,
                min_allocation: 0.0,
                max_allocation: 500.0,
                utility_per_unit: 2.0,
            },
            BudgetConstraint {
                label: "low".into(),
                priority: 1.0,
                min_allocation: 0.0,
                max_allocation: 500.0,
                utility_per_unit: 1.0,
            },
        ];
        let optimizer = BudgetOptimizer::new(500.0, constraints);
        let result = optimizer.optimize().unwrap();

        let high = result
            .allocations
            .iter()
            .find(|a| a.label == "high")
            .unwrap();
        let low = result
            .allocations
            .iter()
            .find(|a| a.label == "low")
            .unwrap();
        assert!(
            high.allocated > low.allocated,
            "high priority ({}) should get more than low ({})",
            high.allocated,
            low.allocated
        );
    }
}

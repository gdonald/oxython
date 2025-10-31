//! Type definitions for the compiler module.
//!
//! This module contains all the supporting types used throughout the compiler,
//! including assignment kinds, variable targets, function scopes, and more.

use crate::object::{Type, UpvalueDescriptor};
use std::collections::{HashMap, HashSet};

/// Represents the kind of assignment operation being performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AssignmentKind {
    /// Simple assignment: `x = value`
    Simple,
    /// Add-assign: `x += value`
    AddAssign,
    /// Multiply-assign: `x *= value`
    MultiplyAssign,
}

/// Represents segments of an f-string literal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum FStringSegment {
    /// A literal string segment
    Literal(String),
    /// An identifier to be interpolated
    Identifier(String),
}

/// Represents the ending token for list/generator comprehensions.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ComprehensionEnd {
    /// Ends with `]` (list comprehension)
    RBracket,
    /// Ends with `)` (generator expression)
    RParen,
}

/// Represents where a variable is stored (local, upvalue, or global).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum VariableTarget {
    /// Local variable at the given slot index
    Local(usize),
    /// Upvalue (captured variable) at the given index
    Upvalue(usize),
    /// Global variable (name stored in constants)
    Global,
}

/// Context for tracking loop state (for break statements).
#[derive(Debug)]
pub(super) struct LoopContext {
    /// Positions in bytecode where break jumps need to be patched
    pub(super) break_jumps: Vec<usize>,
    /// Stack depth to clean up to when breaking
    pub(super) cleanup_depth: usize,
}

impl LoopContext {
    /// Creates a new loop context with the specified cleanup depth.
    pub(super) fn new(cleanup_depth: usize) -> Self {
        LoopContext {
            break_jumps: Vec::new(),
            cleanup_depth,
        }
    }
}

/// Information about a token's position in the source.
#[derive(Debug, Clone, Copy)]
pub(super) struct TokenInfo {
    /// Indentation level of the token
    pub(super) indent: usize,
    /// Starting position in the source
    pub(super) start: usize,
}

/// Represents a function parameter with optional type annotation and default value.
#[derive(Debug, Clone)]
pub(super) struct Parameter {
    /// Parameter name
    pub(super) name: String,
    /// Optional type annotation
    pub(super) type_annotation: Option<Type>,
    /// Optional default value (as a compiled Object)
    pub(super) default_value: Option<crate::object::Object>,
}

impl Parameter {
    /// Creates a new parameter with the given name and optional type annotation.
    pub(super) fn new(name: String, type_annotation: Option<Type>) -> Self {
        Parameter {
            name,
            type_annotation,
            default_value: None,
        }
    }

    /// Creates a new parameter with a default value.
    pub(super) fn new_with_default(
        name: String,
        type_annotation: Option<Type>,
        default_value: crate::object::Object,
    ) -> Self {
        Parameter {
            name,
            type_annotation,
            default_value: Some(default_value),
        }
    }
}

/// Represents a local variable with optional type annotation.
#[derive(Debug, Clone)]
pub(super) struct Local {
    /// Variable name
    pub(super) name: String,
    /// Optional type annotation
    #[allow(dead_code)]
    pub(super) type_annotation: Option<Type>,
}

impl Local {
    /// Creates a new local variable with the given name and optional type annotation.
    pub(super) fn new(name: String, type_annotation: Option<Type>) -> Self {
        Local {
            name,
            type_annotation,
        }
    }
}

/// Represents a function's scope, tracking parameters, locals, and upvalues.
#[derive(Debug)]
pub(super) struct FunctionScope {
    /// Function parameters
    pub(super) parameters: Vec<Parameter>,
    /// Local variables declared in this scope
    pub(super) locals: Vec<Local>,
    /// Upvalues (captured variables from outer scopes)
    pub(super) upvalues: Vec<UpvalueDescriptor>,
    /// Map from variable name to upvalue index for fast lookup
    pub(super) upvalue_map: HashMap<String, usize>,
    /// Set of variables declared as nonlocal
    pub(super) nonlocals: HashSet<String>,
}

impl FunctionScope {
    /// Creates a new function scope with the given parameter names (no type annotations).
    #[allow(dead_code)]
    pub(super) fn new(parameters: Vec<String>) -> Self {
        let params = parameters
            .into_iter()
            .map(|name| Parameter::new(name, None))
            .collect();
        FunctionScope {
            parameters: params,
            locals: Vec::new(),
            upvalues: Vec::new(),
            upvalue_map: HashMap::new(),
            nonlocals: HashSet::new(),
        }
    }

    /// Creates a new function scope with typed parameters.
    pub(super) fn new_with_params(parameters: Vec<Parameter>) -> Self {
        FunctionScope {
            parameters,
            locals: Vec::new(),
            upvalues: Vec::new(),
            upvalue_map: HashMap::new(),
            nonlocals: HashSet::new(),
        }
    }

    /// Resolves a variable name to its slot index in this scope.
    /// Returns None if the variable is not found in parameters or locals.
    pub(super) fn resolve(&self, name: &str) -> Option<usize> {
        if let Some(idx) = self.parameters.iter().position(|param| param.name == name) {
            return Some(idx + 1);
        }

        if let Some(idx) = self.locals.iter().position(|local| local.name == name) {
            return Some(self.parameters.len() + 1 + idx);
        }

        None
    }

    /// Declares a variable in this scope without type annotation.
    /// Returns the slot index and whether it was newly created (true) or already existed (false).
    pub(super) fn declare(&mut self, name: String) -> (usize, bool) {
        if let Some(idx) = self.parameters.iter().position(|param| param.name == name) {
            return (idx + 1, false);
        }

        if let Some(idx) = self.locals.iter().position(|local| local.name == name) {
            return (self.parameters.len() + 1 + idx, false);
        }

        self.locals.push(Local::new(name, None));
        let idx = self.locals.len() - 1;
        (self.parameters.len() + 1 + idx, true)
    }

    /// Declares a variable in this scope with an optional type annotation.
    /// Returns the slot index and whether it was newly created (true) or already existed (false).
    pub(super) fn declare_with_type(
        &mut self,
        name: String,
        type_annotation: Option<Type>,
    ) -> (usize, bool) {
        if let Some(idx) = self.parameters.iter().position(|param| param.name == name) {
            return (idx + 1, false);
        }

        if let Some(idx) = self.locals.iter().position(|local| local.name == name) {
            return (self.parameters.len() + 1 + idx, false);
        }

        self.locals.push(Local::new(name, type_annotation));
        let idx = self.locals.len() - 1;
        (self.parameters.len() + 1 + idx, true)
    }

    /// Adds an upvalue to this scope, returning its index.
    /// If the upvalue already exists, returns the existing index.
    pub(super) fn add_upvalue(&mut self, name: String, is_local: bool, index: usize) -> usize {
        if let Some(existing) = self.upvalue_map.get(&name) {
            return *existing;
        }

        let upvalue_index = self.upvalues.len();
        self.upvalues.push(UpvalueDescriptor { is_local, index });
        self.upvalue_map.insert(name, upvalue_index);
        upvalue_index
    }

    /// Resolves a variable name to its upvalue index in this scope.
    /// Returns None if no upvalue with that name exists.
    pub(super) fn resolve_upvalue(&self, name: &str) -> Option<usize> {
        self.upvalue_map.get(name).copied()
    }
}

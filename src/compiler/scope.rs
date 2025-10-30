//! Scope and variable resolution utilities for the compiler.
//!
//! This module contains functions for managing variable scopes,
//! resolving variable names to their storage locations (local, upvalue, or global),
//! and handling type annotations.

use crate::object::Type;
use crate::token::Token;

use super::types::VariableTarget;

impl super::Compiler<'_> {
    /// Parses a type annotation from the token stream.
    /// Returns None if the next token is not a valid type identifier.
    pub(super) fn parse_type_annotation(&mut self) -> Option<Type> {
        match self.lexer.next() {
            Some(Ok(Token::Identifier(name))) => match name.as_str() {
                "int" => Some(Type::Int),
                "float" => Some(Type::Float),
                "str" => Some(Type::Str),
                "bool" => Some(Type::Bool),
                "list" => Some(Type::List),
                "dict" => Some(Type::Dict),
                "tuple" => Some(Type::Tuple),
                _ => Some(Type::Class(name)),
            },
            _ => None,
        }
    }

    /// Resolves a variable name in the current function's local scope.
    /// Returns the local slot index if found, None otherwise.
    pub(super) fn resolve_local(&self, name: &str) -> Option<usize> {
        self.function_scopes
            .last()
            .and_then(|scope| scope.resolve(name))
    }

    /// Resolves a variable to its target location (local, upvalue, or global).
    /// Checks locals first, then upvalues, then falls back to global.
    pub(super) fn resolve_variable(&mut self, name: &str) -> VariableTarget {
        if let Some(local) = self.resolve_local(name) {
            VariableTarget::Local(local)
        } else if let Some(upvalue) = self.resolve_upvalue(name) {
            VariableTarget::Upvalue(upvalue)
        } else {
            VariableTarget::Global
        }
    }

    /// Resolves a variable as an upvalue (captured from an enclosing scope).
    /// Returns the upvalue index if found, None otherwise.
    pub(super) fn resolve_upvalue(&mut self, name: &str) -> Option<usize> {
        if self.function_scopes.len() < 2 {
            return None;
        }

        let current_index = self.function_scopes.len() - 1;

        if let Some(existing) = self.function_scopes[current_index].resolve_upvalue(name) {
            return Some(existing);
        }

        self.resolve_upvalue_recursive(current_index, name)
    }

    /// Recursively resolves an upvalue by searching through parent scopes.
    /// This handles nested closures where a variable may be captured through multiple levels.
    pub(super) fn resolve_upvalue_recursive(
        &mut self,
        scope_index: usize,
        name: &str,
    ) -> Option<usize> {
        if scope_index == 0 {
            return None;
        }

        let parent_index = scope_index - 1;
        let parent_local = {
            let parent_scope = &self.function_scopes[parent_index];
            parent_scope.resolve(name)
        };

        if let Some(local_index) = parent_local {
            let name_owned = name.to_string();
            let scope = self
                .function_scopes
                .get_mut(scope_index)
                .expect("scope should exist");
            return Some(scope.add_upvalue(name_owned, true, local_index));
        }

        let parent_upvalue_index = self.resolve_upvalue_recursive(parent_index, name)?;

        let name_owned = name.to_string();
        let scope = self
            .function_scopes
            .get_mut(scope_index)
            .expect("scope should exist");
        Some(scope.add_upvalue(name_owned, false, parent_upvalue_index))
    }

    /// Declares a local variable in the current scope without a type annotation.
    /// Returns the slot index and whether it was newly created.
    /// Returns None if we're not in a function scope or if we've exceeded the max local count.
    pub(super) fn declare_local(&mut self, name: String) -> Option<(usize, bool)> {
        let scope = self.function_scopes.last_mut()?;

        if scope.parameters.len() + scope.locals.len() + 1 >= u8::MAX as usize {
            self.had_error = true;
            return None;
        }

        Some(scope.declare(name))
    }

    /// Declares a local variable in the current scope with an optional type annotation.
    /// Returns the slot index and whether it was newly created.
    /// Returns None if we're not in a function scope or if we've exceeded the max local count.
    pub(super) fn declare_local_with_type(
        &mut self,
        name: String,
        type_annotation: Option<Type>,
    ) -> Option<(usize, bool)> {
        let scope = self.function_scopes.last_mut()?;

        if scope.parameters.len() + scope.locals.len() + 1 >= u8::MAX as usize {
            self.had_error = true;
            return None;
        }

        Some(scope.declare_with_type(name, type_annotation))
    }
}
